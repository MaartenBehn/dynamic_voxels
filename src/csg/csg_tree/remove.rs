use crate::util::math_config::MC;

use super::tree::CSGTreeIndex;

#[derive(Debug, Clone, Default)]
pub struct CSGTreeRemove {
    pub base: CSGTreeIndex,
    pub remove: CSGTreeIndex,
}
