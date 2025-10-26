pub mod dependency_tree;
pub mod nodes;
pub mod value;
pub mod update;

use std::{iter, ops::RangeBounds};

use egui_snarl::{InPinId, NodeId, OutPinId};
use itertools::Itertools;
use nodes::TemplateNode;
use octa_force::glam::Vec3;
use octa_force::log::{self, debug, trace};
use smallvec::{SmallVec, smallvec};
use value::ComposeTemplateValue;
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
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone, Default)]
pub struct ComposeTemplate<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub nodes: Vec<TemplateNode>,
    pub values: Vec<ComposeTemplateValue<V2, V3, T, B>>,
    pub max_level: usize,
}

