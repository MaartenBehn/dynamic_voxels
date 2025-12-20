pub mod dependency_tree;
pub mod nodes;
pub mod value;
pub mod update;
pub mod value_hook_iterator;

use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use nodes::TemplateNode;
use octa_force::glam::Vec3;
use octa_force::log::{self, debug, trace};
use smallvec::{SmallVec, smallvec};
use value::{TemplateValue, ValueIndex, VALUE_INDEX_NODE};
use crate::model::data_types::data_type::ComposeDataType;
use crate::model::data_types::number::NumberTemplate;
use crate::model::data_types::number_space::NumberSpaceTemplate;
use crate::model::data_types::position_set::PositionSetTemplate;
use crate::model::data_types::position_space::PositionSpaceTemplate;
use crate::util::number::Nu;

use crate::util::vector::Ve;

use super::composer::build::BS;

pub type TemplateIndex = usize;
pub type OutputIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const TEMPLATE_INDEX_NONE: TemplateIndex = TemplateIndex::MAX;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct Template<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: Vec<TemplateNode>,
    pub values: Vec<TemplateValue<V2, V3, T, B>>,
    pub max_level: usize,
    pub map_node_id: Vec<(TemplateIndex, ValueIndex)>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> Template<V2, V3, T, B> {
    pub fn enshure_map_size(&mut self, node_id: NodeId) {
        if node_id.0 >= self.map_node_id.len() {
            self.map_node_id.resize(node_id.0 + 1, (TEMPLATE_INDEX_NONE, VALUE_INDEX_NODE));
        }
    }

    pub fn get_template_index_from_node_id(&self, node_id: NodeId) -> Option<TemplateIndex> { 
        if self.map_node_id.len() <= node_id.0 {
            return None;
        }

        if self.map_node_id[node_id.0].0 != TEMPLATE_INDEX_NONE {
            Some(self.map_node_id[node_id.0].0)
        } else {
            None
        }
    }

    pub fn get_value_index_from_node_id(&self, node_id: NodeId) -> Option<ValueIndex> {
        if self.map_node_id.len() <= node_id.0 {
            return None;
        }

        if self.map_node_id[node_id.0].1 != VALUE_INDEX_NODE {
            Some(self.map_node_id[node_id.0].1)
        } else {
            None
        }
    }
}

