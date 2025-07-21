use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::OctaResult;

use crate::volume::VolumeQureyPosValid;

use super::{builder::{BU, IT}, collapse::{CollapseNodeKey, CollapseOperation, Collapser}, template::TemplateTree};

#[derive(Debug, Clone)]
pub struct NumberRange {
    pub min: i32,
    pub max: i32,
    pub permutation: Permutation<DefaultBuildHasher>,
    pub perm_counter: usize,
    pub value: i32,
}

impl NumberRange {
    pub fn new(min: i32, max: i32) -> Self {
        let seed = fastrand::u64(0..1000);

        Self {
            min,
            max, 
            permutation: Permutation::new((max - min) as _, seed, DefaultBuildHasher::new()),
            perm_counter: 0,
            value: 0,
        }
    }
}

 
