//! Brand and site selection over open automotive manifest data.

use std::cmp::Ordering;

use sim_kernel::{CapabilityName, Error, Result};

use crate::{AutoLane, BrandCaps, SiteManifest};

/// A requested vehicle make, lanes, and optional capability ceiling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandNeed {
    /// Requested vehicle make, such as `volvo` or `mercedes-benz`.
    pub make: String,
    /// Lanes required for the current bay action.
    pub lanes: Vec<AutoLane>,
    /// Capabilities the selected site must be allowed to hold.
    pub capabilities: Vec<CapabilityName>,
}

impl BrandNeed {
    /// Builds a brand-selection request.
    pub fn new(make: impl Into<String>, lanes: Vec<AutoLane>) -> Self {
        Self {
            make: make.into(),
            lanes,
            capabilities: Vec::new(),
        }
    }

    /// Adds one required capability to the selection request.
    pub fn requiring(mut self, capability: CapabilityName) -> Self {
        self.capabilities.push(capability);
        self
    }
}

/// The selected manifest and derived brand capabilities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandSelection {
    /// Selected site manifest.
    pub manifest: SiteManifest,
    /// Brand capability data derived from the manifest ceiling.
    pub brand_caps: BrandCaps,
    /// True when the manifest directly names the requested make.
    pub exact_make: bool,
    /// Count of extra lanes carried by the selected site beyond the request.
    pub lane_surplus: usize,
}

/// Returns the best manifest for `need`.
pub fn select_brand(manifests: &[SiteManifest], need: &BrandNeed) -> Result<BrandSelection> {
    let mut best: Option<Candidate<'_>> = None;
    let mut denied = Vec::new();
    let mut lane_misses = Vec::new();

    for manifest in manifests {
        let Some(make_match) = make_match(manifest, &need.make) else {
            continue;
        };
        let missing_lanes = missing_lanes(manifest, &need.lanes);
        if !missing_lanes.is_empty() {
            lane_misses.push(format!(
                "{} missing {}",
                manifest.site,
                missing_lanes.join(",")
            ));
            continue;
        }
        let missing_capabilities = missing_capabilities(manifest, &need.capabilities);
        if !missing_capabilities.is_empty() {
            denied.push(format!(
                "{} lacks {}",
                manifest.site,
                missing_capabilities.join(",")
            ));
            continue;
        }

        let candidate = Candidate::new(manifest, make_match, need.lanes.len());
        if best
            .as_ref()
            .map(|current| candidate.cmp(current) == Ordering::Greater)
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    if let Some(candidate) = best {
        return Ok(candidate.into_selection());
    }
    if !denied.is_empty() {
        return Err(Error::Eval(format!(
            "auto brand/select denied by capability ceiling for make {}: {}",
            need.make,
            denied.join("; ")
        )));
    }
    let mut message = format!(
        "no installed auto site matches vehicle make {} and lanes {}",
        need.make,
        lane_names(&need.lanes).join(",")
    );
    if !lane_misses.is_empty() {
        message.push_str("; lane diagnostics: ");
        message.push_str(&lane_misses.join("; "));
    }
    Err(Error::Eval(message))
}

/// Returns the best manifest's brand capabilities for `need`.
pub fn select_brand_caps(manifests: &[SiteManifest], need: &BrandNeed) -> Result<BrandCaps> {
    Ok(select_brand(manifests, need)?.brand_caps)
}

/// Derives brand capability rows for all installed manifests.
pub fn installed_brand_caps(manifests: &[SiteManifest]) -> Vec<BrandCaps> {
    manifests
        .iter()
        .map(|manifest| BrandCaps::new(manifest.brand.clone(), manifest.ceiling.clone()))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MakeMatch {
    Fallback,
    Exact,
}

#[derive(Clone, Copy, Debug)]
struct Candidate<'a> {
    manifest: &'a SiteManifest,
    make_match: MakeMatch,
    lane_surplus: usize,
}

impl Candidate<'_> {
    fn new(manifest: &SiteManifest, make_match: MakeMatch, required_lanes: usize) -> Candidate<'_> {
        Candidate {
            manifest,
            make_match,
            lane_surplus: manifest.lanes.len().saturating_sub(required_lanes),
        }
    }

    fn into_selection(self) -> BrandSelection {
        BrandSelection {
            manifest: self.manifest.clone(),
            brand_caps: BrandCaps::new(self.manifest.brand.clone(), self.manifest.ceiling.clone()),
            exact_make: self.make_match == MakeMatch::Exact,
            lane_surplus: self.lane_surplus,
        }
    }
}

impl PartialEq for Candidate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.make_match == other.make_match
            && self.lane_surplus == other.lane_surplus
            && self.manifest.site == other.manifest.site
    }
}

impl Eq for Candidate<'_> {}

impl PartialOrd for Candidate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.make_match
            .cmp(&other.make_match)
            .then_with(|| other.lane_surplus.cmp(&self.lane_surplus))
            .then_with(|| other.manifest.site.cmp(&self.manifest.site))
    }
}

fn make_match(manifest: &SiteManifest, make: &str) -> Option<MakeMatch> {
    let make = normalize(make);
    if manifest
        .makes
        .iter()
        .any(|candidate| normalize(candidate) == make)
        || normalize(&manifest.brand) == make
    {
        return Some(MakeMatch::Exact);
    }
    manifest
        .makes
        .iter()
        .any(|candidate| candidate == "*" || normalize(candidate) == "multi-brand")
        .then_some(MakeMatch::Fallback)
}

fn missing_lanes(manifest: &SiteManifest, required: &[AutoLane]) -> Vec<String> {
    let lanes = manifest
        .lanes
        .iter()
        .map(|lane| normalize(lane))
        .collect::<Vec<_>>();
    required
        .iter()
        .map(|lane| normalize(&lane.name))
        .filter(|lane| !lanes.contains(lane))
        .collect()
}

fn missing_capabilities(manifest: &SiteManifest, required: &[CapabilityName]) -> Vec<String> {
    required
        .iter()
        .filter(|capability| !manifest.ceiling.iter().any(|item| item == *capability))
        .map(|capability| capability.as_str().to_owned())
        .collect()
}

fn lane_names(lanes: &[AutoLane]) -> Vec<String> {
    lanes.iter().map(|lane| lane.name.clone()).collect()
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use sim_kernel::CapabilityName;

    use super::*;
    use crate::{AUTO_DIAGNOSTICS_READ, AUTO_SERVICE_WRITE};

    #[test]
    fn select_prefers_exact_make_then_narrower_lane_set() {
        let fallback = manifest("bosch", "*", vec!["read", "info", "service"]);
        let exact_broad = manifest("volvo-broad", "volvo", vec!["read", "info", "service"]);
        let exact_narrow = manifest("vida", "volvo", vec!["read", "info"]);
        let need = BrandNeed::new("volvo", lanes(&["read", "info"]));

        let selected = select_brand(&[fallback, exact_broad, exact_narrow], &need).unwrap();

        assert_eq!(selected.manifest.site, "vida");
        assert!(selected.exact_make);
        assert_eq!(selected.lane_surplus, 0);
    }

    #[test]
    fn select_reports_no_match_and_denied_capabilities() {
        let fallback = manifest("bosch", "*", vec!["read", "info"]);
        let no_match = select_brand(
            std::slice::from_ref(&fallback),
            &BrandNeed::new("saab", lanes(&["parts"])),
        );
        assert!(matches!(no_match, Err(Error::Eval(message)) if message.contains("missing parts")));

        let denied = select_brand(
            &[fallback],
            &BrandNeed::new("saab", lanes(&["read"]))
                .requiring(CapabilityName::new(AUTO_SERVICE_WRITE)),
        );
        assert!(
            matches!(denied, Err(Error::Eval(message)) if message.contains("capability ceiling"))
        );
    }

    fn manifest(site: &str, make: &str, lanes: Vec<&str>) -> SiteManifest {
        SiteManifest::new(
            site,
            "vehicle-alpha",
            site,
            lanes.into_iter().map(str::to_owned).collect(),
            vec!["modeled".to_owned()],
            vec!["read/dtc".to_owned()],
        )
        .with_makes(vec![make.to_owned()])
        .with_ceiling(vec![CapabilityName::new(AUTO_DIAGNOSTICS_READ)])
    }

    fn lanes(items: &[&str]) -> Vec<AutoLane> {
        items.iter().map(|item| AutoLane::new(*item)).collect()
    }
}
