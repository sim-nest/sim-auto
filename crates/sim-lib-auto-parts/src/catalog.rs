//! Modeled EPC and aftermarket catalog directories.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use sim_kernel::{
    Cx, Error, Expr, Object, ObjectCompat, Result, Symbol, Value,
    id::CORE_TABLE_CLASS_ID,
    object::ClassRef,
    table::{Dir, Table},
};

use crate::{PartLine, PartsCatalog};

/// Immutable modeled parts catalog exposed as a SIM directory table.
#[sim_citizen_derive::non_citizen(
    reason = "modeled catalog handle; canonical values are auto/PartLine leaves",
    kind = "handle",
    descriptor = "auto/PartLine"
)]
#[derive(Clone)]
pub struct PartsDir {
    label: String,
    node: Arc<CatalogNode>,
}

#[derive(Clone, Default)]
struct CatalogNode {
    dirs: BTreeMap<String, CatalogNode>,
    parts: BTreeMap<String, PartLine>,
}

impl PartsDir {
    /// Builds a directory from a label and catalog node.
    fn new(label: impl Into<String>, node: CatalogNode) -> Self {
        Self {
            label: label.into(),
            node: Arc::new(node),
        }
    }

    /// Fetches a part leaf by path.
    pub fn get_path(&self, cx: &mut Cx, path: &[String]) -> Result<Value> {
        let Some((leaf, parents)) = path.split_last() else {
            return cx.factory().opaque(Arc::new(self.clone()));
        };
        let dir = self.open_path(parents)?;
        dir.get(cx, Symbol::new(leaf.clone()))
    }

    fn open_path(&self, path: &[String]) -> Result<Self> {
        let mut node = self.node.as_ref();
        let mut label = self.label.clone();
        for segment in path {
            node = node.dirs.get(segment).ok_or_else(|| {
                Error::Eval(format!(
                    "parts catalog path segment {segment} is not a directory"
                ))
            })?;
            label.push('/');
            label.push_str(segment);
        }
        Ok(Self {
            label,
            node: Arc::new(node.clone()),
        })
    }
}

impl Object for PartsDir {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("auto/parts-dir[{}]", self.label))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for PartsDir {
    fn class(&self, cx: &mut Cx) -> Result<ClassRef> {
        let symbol = Symbol::qualified("core", "Table");
        if let Some(value) = cx.registry().class_by_symbol(&symbol) {
            return Ok(value.clone());
        }
        cx.factory().class_stub(CORE_TABLE_CLASS_ID, symbol)
    }

    fn as_expr(&self, cx: &mut Cx) -> Result<Expr> {
        self.as_table_expr(cx)
    }

    fn truth(&self, _cx: &mut Cx) -> Result<bool> {
        Ok(!self.node.dirs.is_empty() || !self.node.parts.is_empty())
    }

    fn as_table_impl(&self) -> Option<&dyn sim_kernel::Table> {
        Some(self)
    }

    fn as_dir(&self) -> Option<&dyn Dir> {
        Some(self)
    }
}

impl sim_kernel::Table for PartsDir {
    fn backend_symbol(&self) -> Symbol {
        Symbol::qualified("auto", "parts-dir")
    }

    fn get(&self, cx: &mut Cx, key: Symbol) -> Result<Value> {
        match self.node.parts.get(key.name.as_ref()) {
            Some(part) => cx.factory().expr(part.to_expr()),
            None => cx.factory().nil(),
        }
    }

    fn set(&self, _cx: &mut Cx, _key: Symbol, _value: Value) -> Result<()> {
        Err(immutable_error())
    }

    fn has(&self, _cx: &mut Cx, key: Symbol) -> Result<bool> {
        Ok(self.node.parts.contains_key(key.name.as_ref())
            || self.node.dirs.contains_key(key.name.as_ref()))
    }

    fn del(&self, _cx: &mut Cx, _key: Symbol) -> Result<Value> {
        Err(immutable_error())
    }

    fn keys(&self, _cx: &mut Cx) -> Result<Vec<Symbol>> {
        let keys = self
            .node
            .dirs
            .keys()
            .chain(self.node.parts.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        Ok(keys.into_iter().map(Symbol::new).collect())
    }

    fn entries(&self, cx: &mut Cx) -> Result<Vec<(Symbol, Value)>> {
        self.node
            .parts
            .iter()
            .map(|(key, part)| Ok((Symbol::new(key.clone()), cx.factory().expr(part.to_expr())?)))
            .collect()
    }

    fn len(&self, _cx: &mut Cx) -> Result<usize> {
        Ok(self.node.parts.len())
    }

    fn clear(&self, _cx: &mut Cx) -> Result<()> {
        Err(immutable_error())
    }
}

impl Dir for PartsDir {
    fn mkdir(&self, _cx: &mut Cx, _name: Symbol) -> Result<Value> {
        Err(immutable_error())
    }

    fn opendir(&self, cx: &mut Cx, name: Symbol) -> Result<Option<Value>> {
        match self.node.dirs.get(name.name.as_ref()) {
            Some(node) => cx
                .factory()
                .opaque(Arc::new(Self::new(
                    format!("{}/{}", self.label, name.name),
                    node.clone(),
                )))
                .map(Some),
            None if self.node.parts.contains_key(name.name.as_ref()) => Err(Error::Eval(format!(
                "parts catalog key {name} is a part, not a directory"
            ))),
            None => Ok(None),
        }
    }

    fn rmdir(&self, _cx: &mut Cx, _name: Symbol) -> Result<Value> {
        Err(immutable_error())
    }

    fn is_dir(&self, _cx: &mut Cx, name: Symbol) -> Result<bool> {
        Ok(self.node.dirs.contains_key(name.name.as_ref()))
    }
}

/// Builds the modeled Mercedes EPC catalog directory.
pub fn modeled_epc_dir() -> PartsDir {
    PartsDir::new("epc-modeled", epc_catalog())
}

/// Builds the modeled aftermarket catalog directory.
pub fn modeled_aftermarket_dir() -> PartsDir {
    PartsDir::new("aftermarket-modeled", aftermarket_catalog())
}

/// Builds a modeled catalog directory by family.
pub fn parts_dir(catalog: PartsCatalog) -> PartsDir {
    match catalog {
        PartsCatalog::EpcModeled => modeled_epc_dir(),
        PartsCatalog::AftermarketModeled => modeled_aftermarket_dir(),
    }
}

/// Returns the part expression at a modeled catalog path.
pub fn catalog_part(cx: &mut Cx, catalog: PartsCatalog, path: &[String]) -> Result<Value> {
    parts_dir(catalog).get_path(cx, path)
}

fn epc_catalog() -> CatalogNode {
    CatalogNode::default()
        .with_dir(
            "engine",
            CatalogNode::default().with_dir(
                "ignition",
                CatalogNode::default()
                    .with_part(
                        "coil-1",
                        PartLine::new(
                            "SIM-COIL-1",
                            Some("A0001500180"),
                            "modeled ignition coil for cylinder 1",
                            1,
                        ),
                    )
                    .with_part(
                        "plug-set",
                        PartLine::new(
                            "SIM-PLUG-SET",
                            Some("A0041591803"),
                            "modeled spark plug service set",
                            1,
                        ),
                    ),
            ),
        )
        .with_dir(
            "brake",
            CatalogNode::default().with_dir(
                "front",
                CatalogNode::default().with_part(
                    "pad-set",
                    PartLine::new(
                        "SIM-PAD-FRONT",
                        Some("A0004208700"),
                        "modeled front brake pad set",
                        1,
                    ),
                ),
            ),
        )
}

fn aftermarket_catalog() -> CatalogNode {
    CatalogNode::default().with_dir(
        "engine",
        CatalogNode::default().with_dir(
            "ignition",
            CatalogNode::default().with_part(
                "coil-1",
                PartLine::new(
                    "MEK-SIM-COIL-1",
                    Some("A0001500180"),
                    "modeled aftermarket ignition coil",
                    1,
                ),
            ),
        ),
    )
}

impl CatalogNode {
    fn with_dir(mut self, key: &str, node: CatalogNode) -> Self {
        self.dirs.insert(key.to_owned(), node);
        self
    }

    fn with_part(mut self, key: &str, part: PartLine) -> Self {
        self.parts.insert(key.to_owned(), part);
        self
    }
}

fn immutable_error() -> Error {
    Error::Eval("modeled parts catalog is immutable".to_owned())
}
