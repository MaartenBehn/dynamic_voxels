use octa_force::glam::Vec3;

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use std::{ops::RangeBounds, usize};

use super::node::WFC;

pub type NodeIdentifier = usize;
pub const NodeIdentifierNone: NodeIdentifier = NodeIdentifier::MAX;

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
    },
    VolumeChild {
        identifier: Option<NodeIdentifier>,
        parent_identifier: NodeIdentifier,
        on_collapse: fn(&mut CSGTree, Vec3),
    }
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
    Link { to_node: NodeIdentifier },
}

#[derive(Debug, Clone)]
pub struct NumberRangeBuilder {
    pub identifier: Option<NodeIdentifier>,
    pub defines: NumberRangeDefinesType,
}


#[derive(Debug, Clone)]
pub struct VolumeBuilder {
    pub identifier: Option<NodeIdentifier>,
    pub csg: Option<CSGTree>,
}

#[derive(Debug, Clone)]
pub struct PosBuilder {
    pub identifier: Option<NodeIdentifier>,
    pub parent_identifier: Option<usize>,
    pub on_collapse: fn(&mut CSGTree, Vec3),
}

impl<U: Clone> WFCBuilder<U> {
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
        };

        self.base_nodes_templates.push(volume);

        self.node.children.push(volume_builder.identifier.unwrap());

        self
    }
    
    pub fn pos(mut self, pos_options: fn(b: PosBuilder) -> PosBuilder) -> Self {
        let mut pos_builder = PosBuilder::new();
        pos_builder = pos_options(pos_builder);

        let pos = BaseNodeTemplate::VolumeChild {
            identifier: pos_builder.identifier,
            parent_identifier: pos_builder.parent_identifier.unwrap(),
            on_collapse: pos_builder.on_collapse,
        };

        self.base_nodes_templates.push(pos);

        self.node.children.push(pos_builder.identifier.unwrap());

        self
    }
}

impl<U> UserNodeTemplate<U> {
    fn new(data: U) -> UserNodeTemplate<U> {
        UserNodeTemplate {
            data,
            identifier: None,
            children: vec![],
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
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn csg_tree(mut self, csg: CSGTree) -> Self {
        self.csg = Some(csg);
        self
    }

    pub fn csg_node(self, node: CSGNodeData) -> Self {
        let mut csg = CSGTree::new();
        csg.nodes = vec![CSGNode::new(node)];
        csg.set_all_aabbs(0.0);

        self.csg_tree(csg)
    }

}

impl PosBuilder {
    pub fn new() -> Self {
        PosBuilder {
            identifier: None,
            parent_identifier: None,
            on_collapse: |_, _| {},
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn in_volume(mut self, identifier: NodeIdentifier) -> Self {
        self.parent_identifier = Some(identifier);
        self
    }

    pub fn on_collapse(mut self, on_collapse: fn(&mut CSGTree, Vec3)) -> Self {
        self.on_collapse = on_collapse;
        self
    }
}
