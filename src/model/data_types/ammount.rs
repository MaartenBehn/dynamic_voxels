use std::iter;

use egui_snarl::{NodeId, OutPinId};
use itertools::Itertools;

use crate::{model::composer::{build::BS, dependency_tree::DependencyTree, nodes::{ComposeNode, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex}, ModelComposer}, util::{number::Nu, vector::Ve}};

use super::{data_type::ComposeDataType, number::NumberTemplate};



