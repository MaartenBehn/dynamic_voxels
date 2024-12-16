use octa_force::glam::Vec3;

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use super::base_node::WFC;
use std::ops::RangeBounds;

pub type NodeIdentifier = usize;

#[derive(Debug, Clone)]
pub struct WFCBuilder<U> {
    pub user_nodes: Vec<UserNodeTemplate<U>>,
    pub base_nodes: Vec<BaseNodeTemplate>,
}

#[derive(Debug, Clone)]
pub struct UserNodeTemplate<U> {
    pub identifier: Option<NodeIdentifier>,
    pub data: U,
    pub children: Vec<NodeIdentifier>,
    pub on_collapse: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum BaseNodeTemplate {
    NumberRange {
        identifier: Option<NodeIdentifier>,
        min: i32,
        max: i32,
        defines: NumberRangeDefinesType,
    },
    Volume {
        identifier: Option<NodeIdentifier>,
        csg: CSGTree,
        defines: VolumeDefinesType,
    },
}

#[derive(Debug, Clone)]
pub struct WFCUserNodeBuilder<U> {
    pub node: UserNodeTemplate<U>,
    pub base_nodes_templates: Vec<BaseNodeTemplate>,
}

#[derive(Debug, Clone)]
pub enum NumberRangeDefinesType {
    None,
    Amount { of_node: NodeIdentifier },
}

#[derive(Debug, Clone)]
pub struct NumberRangeBuilder {
    pub identifier: Option<NodeIdentifier>,
    pub defines: NumberRangeDefinesType,
}

#[derive(Debug, Clone)]
pub enum VolumeDefinesType {
    None,
    Attribute {
        of_node: NodeIdentifier,

        identifier: NodeIdentifier,
    },
}

#[derive(Debug, Clone)]
pub struct VolumeBuilder {
    pub identifier: Option<NodeIdentifier>,
    pub csg: Option<CSGTree>,
    pub defines: VolumeDefinesType,
}

#[derive(Debug, Clone)]
pub enum Action {
    TransformNumberSet {
        identifier: NodeIdentifier,
        func: fn(Vec<i32>) -> Vec<i32>,
    },
    TransformVolume {
        identifier: NodeIdentifier,
        func: fn(CSGTree) -> CSGTree,
    },
    TransformVolumeWithPosAttribute {
        volume_identifier: NodeIdentifier,
        attribute_identifier: NodeIdentifier,
        func: fn(CSGTree, Vec3) -> CSGTree,
    },
}

impl<U: ToOwned<Owned = U>> WFCBuilder<U> {
    pub fn new() -> WFCBuilder<U> {
        WFCBuilder {
            user_nodes: vec![],
            base_nodes: vec![],
        }
    }

    pub fn node(
        mut self,
        data: U,
        build_node: fn(builder: WFCUserNodeBuilder<U>) -> WFCUserNodeBuilder<U>,
    ) -> WFCBuilder<U> {
        let mut builder = WFCUserNodeBuilder::new(data);
        builder = build_node(builder);

        self.user_nodes.push(builder.node);
        self.base_nodes.append(&mut builder.base_nodes_templates);
        self
    }

    pub fn build(&self) -> WFC<U> {
        WFC::new(self)
    }
}

impl<U> WFCUserNodeBuilder<U> {
    fn new(data: U) -> Self {
        WFCUserNodeBuilder {
            node: UserNodeTemplate::new(data),
            base_nodes_templates: vec![],
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.node.identifier = Some(identifier);
        self
    }

    pub fn number_range<R: RangeBounds<i32>>(
        mut self,
        range: R,
        number_set_options: fn(b: NumberRangeBuilder) -> NumberRangeBuilder,
    ) -> Self {
        let start_bound = match range.start_bound() {
            std::ops::Bound::Included(&num) => num,
            std::ops::Bound::Excluded(&num) => num + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end_bound = match range.end_bound() {
            std::ops::Bound::Included(&num) => num + 1,
            std::ops::Bound::Excluded(&num) => num,
            std::ops::Bound::Unbounded => panic!("Range can not be unbounded"),
        };

        /*
                let mut vals = vec![];
                for i in start_bound..end_bound {
                    vals.push(i);
                }
        */

        let mut number_set_builder = NumberRangeBuilder::new();
        number_set_builder = number_set_options(number_set_builder);

        let number_range = BaseNodeTemplate::NumberRange {
            defines: number_set_builder.defines,
            min: start_bound,
            max: end_bound,
            identifier: number_set_builder.identifier,
        };

        self.base_nodes_templates.push(number_range);

        self.node
            .children
            .push(number_set_builder.identifier.unwrap());

        self
    }

    pub fn volume(mut self, volume_options: fn(b: VolumeBuilder) -> VolumeBuilder) -> Self {
        let mut volume_builder = VolumeBuilder::new();
        volume_builder = volume_options(volume_builder);

        let volume = BaseNodeTemplate::Volume {
            identifier: volume_builder.identifier,
            csg: volume_builder.csg.unwrap(),
            defines: volume_builder.defines,
        };

        self.base_nodes_templates.push(volume);

        self.node.children.push(volume_builder.identifier.unwrap());

        self
    }

    pub fn on_collapse_modify_volume_with_pos_attribute(
        mut self,
        volume_identifier: NodeIdentifier,
        attribute_identifier: NodeIdentifier,
        func: fn(CSGTree, Vec3) -> CSGTree,
    ) -> Self {
        self.node
            .on_collapse
            .push(Action::TransformVolumeWithPosAttribute {
                func,
                volume_identifier,
                attribute_identifier,
            });
        self
    }
}

impl<U> UserNodeTemplate<U> {
    fn new(data: U) -> UserNodeTemplate<U> {
        UserNodeTemplate {
            data,
            identifier: None,
            children: vec![],
            on_collapse: vec![],
        }
    }
}

impl NumberRangeBuilder {
    pub fn new() -> Self {
        NumberRangeBuilder {
            defines: NumberRangeDefinesType::None,
            identifier: None,
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn defines(mut self, defines: NumberRangeDefinesType) -> Self {
        self.defines = defines;
        self
    }
}

impl VolumeBuilder {
    pub fn new() -> Self {
        VolumeBuilder {
            identifier: None,
            csg: None,
            defines: VolumeDefinesType::None,
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn tree(mut self, tree: CSGTree) -> Self {
        self.csg = Some(tree);
        self
    }

    pub fn csg_node(self, node: CSGNodeData) -> Self {
        let mut tree = CSGTree::new();
        tree.nodes = vec![CSGNode::new(node)];
        tree.set_all_aabbs(0.0);

        self.tree(tree)
    }

    pub fn defines(mut self, defines: VolumeDefinesType) -> Self {
        self.defines = defines;
        self
    }
}
