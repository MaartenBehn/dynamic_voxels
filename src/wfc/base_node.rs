use std::usize;

use octa_force::glam::Vec3;

use crate::cgs_tree::tree::CSGTree;

use super::builder::{
    Action, BaseNodeTemplate, NodeIdentifier, NumberRangeDefinesType, UserNodeTemplate, WFCBuilder,
};

#[derive(Debug, Clone)]
pub enum Node<U> {
    Number {
        val: i32,
    },
    NumberSet {
        vals: Vec<i32>,
        children: Vec<usize>,
    },

    Pos {
        pos: Vec3,
    },
    Volume {
        csg: CSGTree,
        children: Vec<usize>,
    },

    User {
        data: U,
        children: Vec<usize>,
    },
}

#[derive(Debug)]
pub struct WFC<U> {
    nodes: Vec<Node<U>>,
}

impl<U> WFC<U>
where
    U: ToOwned<Owned = U>,
{
    pub(crate) fn new(builder: &WFCBuilder<U>) -> Self {
        let mut wfc = WFC { nodes: vec![] };

        wfc.add_user_node(builder, 0);

        wfc
    }

    pub fn add_user_node(&mut self, builder: &WFCBuilder<U>, index: usize) -> usize {
        let node_template = &builder.user_nodes[index];

        let mut children = vec![];
        for child in node_template.children.iter() {
            let index = builder
                .get_index_of_base_node_identifier(*child)
                .expect("User Node Identifier must be Base Node");

            let index = self.add_base_node(builder, index);

            children.push(index);
        }

        let node = Node::User {
            data: node_template.data.to_owned(),
            children,
        };

        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    pub fn add_base_node(&mut self, builder: &WFCBuilder<U>, index: usize) -> usize {
        let node_template = &builder.base_nodes[index];
        match node_template {
            BaseNodeTemplate::NumberRange {
                identifier,
                min,
                max,
                defines,
            } => {
                let mut vals = vec![];

                for i in *min..*max {
                    vals.push(i);
                }

                let mut children = vec![];

                match *defines {
                    NumberRangeDefinesType::None => {}
                    NumberRangeDefinesType::Amount { of_node } => {
                        let index = builder
                            .get_index_of_user_node_identifier(of_node)
                            .expect("User Node Identifier must be Base Node");

                        let index = self.add_user_node(builder, index);

                        children.push(index);
                    }
                }

                let node = Node::NumberSet { vals, children };

                let index = self.nodes.len();
                self.nodes.push(node);

                index
            }
            BaseNodeTemplate::Volume { csg, defines, .. } => {
                let mut children = vec![];

                match defines {
                    super::builder::VolumeDefinesType::None => {}
                    super::builder::VolumeDefinesType::Attribute {
                        of_node,
                        identifier,
                    } => {}
                }

                let node = Node::Volume {
                    csg: csg.to_owned(),
                    children,
                };
                let index = self.nodes.len();
                self.nodes.push(node);
                index
            }
        }
    }
}

impl<U> WFCBuilder<U> {
    pub fn get_index_of_base_node_identifier(&self, identifier: NodeIdentifier) -> Option<usize> {
        for (i, node) in self.base_nodes.iter().enumerate() {
            match node {
                BaseNodeTemplate::NumberRange {
                    identifier: ident, ..
                }
                | BaseNodeTemplate::Volume {
                    identifier: ident, ..
                } => {
                    if Some(identifier) == *ident {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    pub fn get_index_of_user_node_identifier(&self, identifier: NodeIdentifier) -> Option<usize> {
        for (i, node) in self.user_nodes.iter().enumerate() {
            if Some(identifier) == node.identifier {
                return Some(i);
            }
        }

        None
    }
}
