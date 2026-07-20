//! Modeled ECU flash backup and restore helpers.

use sim_kernel::{Error, Expr, Result, Symbol};

/// Content-addressed stock-map backup for a modeled ECU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockMapBackup {
    /// ECU label the stock bytes came from.
    pub ecu: String,
    /// Stable content key for the stock bytes.
    pub content_key: String,
    /// Synthetic stock bytes held by the modeled backup.
    pub bytes: Vec<u8>,
}

impl StockMapBackup {
    /// Builds a stock-map backup and derives its content key.
    pub fn new(ecu: impl Into<String>, bytes: Vec<u8>) -> Self {
        let ecu = ecu.into();
        let content_key = stock_content_key(&ecu, &bytes);
        Self {
            ecu,
            content_key,
            bytes,
        }
    }

    /// Returns the reversal artifact required by irreversible flash writes.
    pub fn reversal_artifact(&self) -> Expr {
        Expr::Map(vec![
            entry(
                "kind",
                Expr::Symbol(Symbol::qualified("auto", "StockMapBackup")),
            ),
            string_entry("ecu", &self.ecu),
            string_entry("content-key", &self.content_key),
            entry("bytes", Expr::Bytes(self.bytes.clone())),
        ])
    }
}

/// Modeled flash session that never touches a live VCI or ECU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeledFlashSession {
    ecu: String,
    stock_bytes: Vec<u8>,
    current_bytes: Vec<u8>,
}

impl ModeledFlashSession {
    /// Builds a modeled flash session from synthetic stock bytes.
    pub fn new(ecu: impl Into<String>, stock_bytes: Vec<u8>) -> Self {
        Self {
            ecu: ecu.into(),
            current_bytes: stock_bytes.clone(),
            stock_bytes,
        }
    }

    /// Returns the current modeled ECU bytes.
    pub fn read_ecu(&self) -> &[u8] {
        &self.current_bytes
    }

    /// Stores a stock-map backup for the original modeled ECU bytes.
    pub fn backup_stock(&self) -> StockMapBackup {
        StockMapBackup::new(self.ecu.clone(), self.stock_bytes.clone())
    }

    /// Applies a modeled flash payload after checking the stock backup.
    pub fn flash(&mut self, tuned_bytes: Vec<u8>, backup: &StockMapBackup) -> Result<()> {
        self.validate_backup(backup)?;
        self.current_bytes = tuned_bytes;
        Ok(())
    }

    /// Restores the modeled ECU to the backed-up stock bytes.
    pub fn restore(&mut self, backup: &StockMapBackup) -> Result<Vec<u8>> {
        self.validate_backup(backup)?;
        self.current_bytes = backup.bytes.clone();
        Ok(self.current_bytes.clone())
    }

    fn validate_backup(&self, backup: &StockMapBackup) -> Result<()> {
        if backup.ecu != self.ecu {
            return Err(Error::Eval(format!(
                "stock-map backup for ECU {} cannot restore ECU {}",
                backup.ecu, self.ecu
            )));
        }
        let expected = stock_content_key(&backup.ecu, &backup.bytes);
        if backup.content_key != expected {
            return Err(Error::Eval(
                "stock-map backup content key does not match bytes".to_owned(),
            ));
        }
        if backup.bytes != self.stock_bytes {
            return Err(Error::Eval(
                "stock-map backup bytes do not match modeled stock".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Computes a stable content key for modeled stock-map bytes.
pub fn stock_content_key(ecu: &str, bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in ecu
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0xff))
        .chain(bytes.iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("auto-stock-fnv1a64-{hash:016x}")
}

fn string_entry(name: &str, value: &str) -> (Expr, Expr) {
    entry(name, Expr::String(value.to_owned()))
}

fn entry(name: &str, value: Expr) -> (Expr, Expr) {
    (Expr::Symbol(Symbol::new(name.to_owned())), value)
}
