use octa_force::glam::Vec3;

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree};

use std::{marker::PhantomData, ops::RangeBounds, usize};

use super::node::WFC;

pub type NodeIdentifier = usize;
pub const NodeIdentifierNone: NodeIdentifier = NodeIdentifier::MAX;

#[derive(Debug, Clone)]
pub struct WFCBuilder<U: Clone> {
    pub user_nodes: Vec<UserNodeTemplate<U>>,
    pub base_nodes: Vec<BaseNodeTemplate<U>>,
}

#[derive(Debug, Clone)]
pub struct UserNodeTemplate<U: Clone> {
    pub phantom: PhantomData<U>,

    pub identifier: Option<NodeIdentifier>,
    pub children: Vec<NodeIdentifier>,
    pub on_generate: fn(&mut WFC<U>, &mut U),
}

#[derive(Debug, Clone)]
pub enum BaseNodeTemplate<U: Clone> {
    NumberRange {
        identifier: Option<NodeIdentifier>,
        min: i32,
        max: i32,
        defines: NumberRangeDefinesType,
    }, 
    Pos {
        identifier: Option<NodeIdentifier>,
        on_collapse: fn(&mut WFC<U>, &mut U) -> (Vec3, bool),
    },
}

#[derive(Debug, Clone)]
pub struct WFCUserNodeBuilder<U: Clone> {
    pub node: UserNodeTemplate<U>,
    pub base_nodes_templates: Vec<BaseNodeTemplate<U>>,
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
pub struct PosBuilder<U: Clone> {
    pub identifier: Option<NodeIdentifier>,
    pub on_collapse: fn(&mut WFC<U>, &mut U) -> (Vec3, bool),
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
        build_node: fn(builder: WFCUserNodeBuilder<U>) -> WFCUserNodeBuilder<U>,
    ) -> WFCBuilder<U> {
        let mut builder = WFCUserNodeBuilder::new();
        builder = build_node(builder);

        self.user_nodes.push(builder.node);
        self.base_nodes.append(&mut builder.base_nodes_templates);
        self
    }

    pub fn build(&self, user_data: &mut U) -> WFC<U> {
        WFC::new(self, user_data)
    }
}

impl<U: Clone> WFCUserNodeBuilder<U> {
    fn new() -> Self {
        WFCUserNodeBuilder {
            node: UserNodeTemplate::new(),
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

    pub fn pos(
        mut self,
        pos_options: fn(b: PosBuilder<U>) -> PosBuilder<U>,
    ) -> Self {

        let mut pos_builder = PosBuilder::new();
        pos_builder = pos_options(pos_builder);

        let pos = BaseNodeTemplate::Pos {
            identifier: pos_builder.identifier,
            on_collapse: pos_builder.on_collapse, 
        };

        self.base_nodes_templates.push(pos);

        self
    }

    pub fn on_generate(mut self, on_generate: fn(&mut WFC<U>, &mut U)) -> Self {
        self.node.on_generate = on_generate;
        self
    }
}

impl<U: Clone> UserNodeTemplate<U> {
    fn new() -> UserNodeTemplate<U> {
        UserNodeTemplate {
            identifier: None,
            children: vec![],
            on_generate: |_, _| {},
            phantom: PhantomData,
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

impl<U: Clone> PosBuilder<U> {
    pub fn new() -> Self {
        PosBuilder {
            identifier: None,
            on_collapse: |_, _| { panic!("Pos has no collapse implementation!") }
        }
    }

    pub fn identifier(mut self, identifier: NodeIdentifier) -> Self {
        self.identifier = Some(identifier);
        self
    } 

    pub fn on_collapse(mut self, on_collapse: fn(&mut WFC<U>, &mut U) -> (Vec3, bool)) -> Self {
        self.on_collapse = on_collapse;
        self
    }
}
