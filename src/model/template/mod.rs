pub mod dependency_tree;
pub mod nodes;
pub mod value;
pub mod value_hook_iterator;
pub mod cut_loops;

use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use nodes::TemplateNode;
use octa_force::glam::Vec3;
use octa_force::log::{self, debug, trace};
use smallvec::{SmallVec, smallvec};
use value::{ValueIndex, VALUE_INDEX_NODE};
use crate::model::data_types::data_type::{ComposeDataType, TemplateValue};
use crate::model::data_types::number::NumberValue;
use crate::model::data_types::number_space::NumberSpaceValue;
use crate::model::data_types::position_set::PositionSetValue;
use crate::model::data_types::position_space::PositionSpaceValue;

pub type TemplateIndex = usize;
pub type OutputIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const TEMPLATE_INDEX_NONE: TemplateIndex = TemplateIndex::MAX;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct Template {
    pub nodes: Vec<TemplateNode>,
    pub values: Vec<TemplateValue>,
    pub max_level: usize,
}

impl Template {
    pub fn empty() -> Self {
        Self {
            nodes: vec![TemplateNode {
                index: 0,
                value_index: 0,
                depends_loop: smallvec![],
                depends: smallvec![],
                dependend: smallvec![],
                level: 1,
                creates: smallvec![],
                created_by: (TEMPLATE_INDEX_NONE, 0),
                dependecy_tree: Default::default(),
                external_input_marker: Default::default(),
            }],
            values: vec![TemplateValue::None],
            max_level: 1,
        }
    }
}

