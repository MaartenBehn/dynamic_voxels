use egui_snarl::NodeId;
use smallvec::SmallVec;

use crate::{model::composer::build::BS, util::{number::Nu, vector::Ve}};

use super::{dependency_tree::{DependencyPath, DependencyTree}, value::{ComposeTemplateValue, ValueIndex}, TemplateIndex};

#[derive(Debug, Clone)]
pub struct TemplateNode {
    pub index: TemplateIndex,
    pub value_index: ValueIndex,

    pub level: usize,
    pub creates: SmallVec<[Creates; 4]>,

    pub depends: SmallVec<[TemplateIndex; 4]>,
    pub dependecy_tree: DependencyTree,
    pub depends_loop: SmallVec<[(TemplateIndex, DependencyPath); 4]>,
    
    pub dependend: SmallVec<[TemplateIndex; 4]>,
}

#[derive(Debug, Clone)]
pub struct Creates {
    pub to_create: TemplateIndex,
    pub t: CreatesType,
    pub others: SmallVec<[TemplateIndex; 2]>
}

#[derive(Debug, Clone, Copy)]
pub enum CreatesType {
    One,
    Children,
}
