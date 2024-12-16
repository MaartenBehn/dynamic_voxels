use octa_force::{anyhow::bail, OctaResult};

use super::builder::NodeIdentifier;

pub struct IdentifierCounter {
    used_vals: Vec<usize>,
}

impl IdentifierCounter {
    pub fn new() -> Self {
        IdentifierCounter { used_vals: vec![] }
    }

    pub fn use_val(&mut self, val: NodeIdentifier) -> OctaResult<()> {
        if self.used_vals.contains(&val) {
            bail!("Identifier: {val} is already used");
        }

        Ok(())
    }
}
