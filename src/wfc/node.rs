use std::{iter, usize};

use octa_force::glam::Vec3;

use crate::cgs_tree::tree::CSGTree;

use super::builder::{
    BaseNodeTemplate, NodeIdentifier, NumberRangeDefinesType, UserNodeTemplate, WFCBuilder,
};

#[derive(Debug, Clone)]
pub enum Node<U: Clone> {
    None,
    Number {
        val: i32,
    },
    NumberSet {
        vals: Vec<i32>,
        r#type: NumberSetType,
        children: Vec<usize>,
    },

    Pos {
        pos: Vec3,
    },
    Volume {
        csg: CSGTree,
        children: Vec<usize>,
    },
    VolumeChild {
        parent: usize,
        children: Vec<usize>,
        on_collapse: fn(&mut CSGTree, Vec3),
    },

    User {
        data: U,
        attributes: Vec<usize>,
        on_show: fn(&mut WFC<U>, usize, &mut CSGTree), 
    },
}

#[derive(Debug, Clone)]
pub enum NumberSetType {
    None,
    Amount, 
    Link,
}

#[derive(Debug, Clone)]
pub struct WFC<U: Clone> {
    pub nodes: Vec<Node<U>>,
    pub node_identifier: Vec<Option<NodeIdentifier>>,
    pub link_data: Vec<NodeIdentifier>,
}

impl<U: Clone> WFC<U>{
    pub(crate) fn new(builder: &WFCBuilder<U>) -> Self {
        let mut wfc = WFC { 
            nodes: vec![],
            node_identifier: vec![],
            link_data: vec![], 
        };

        wfc.build_user_node(builder, 0);
        wfc
    }

    pub fn show(&mut self, csg: &mut CSGTree) {
        self.show_node(csg, 0);
    }

    fn show_node(&mut self, csg: &mut CSGTree, index: usize) {
        match &self.nodes[index] {
            Node::NumberSet { r#type, children, .. } => {
                match r#type {
                    NumberSetType::Amount => {
                        let children = children.to_owned();

                        for child in children {
                            self.show_node(csg, child);
                        }
                    },
                    _ => {}
                }
            },
            Node::User { data, attributes, on_show } => {
                let attributes = attributes.to_owned();

                on_show(self, index, csg);

                for attribute in attributes {
                    self.show_node(csg, attribute);
                }
            },
            _ => {}
        }
    }

    pub fn get_children_with_identifier(&mut self, index: usize, identifier: NodeIdentifier) -> Vec<usize> {
        let empty = vec![];
        match &self.nodes[index] {
            Node::None => panic!("get children none should never be none"),
            Node::Number { .. } 
             | Node::Pos { .. } => &empty,
            Node::NumberSet { children, .. }
             | Node::Volume { children, .. }
             | Node::VolumeChild { children, .. }
             | Node::User { attributes: children, .. } => children,
        }
            .iter()
            .filter(|i| {
                self.node_identifier[**i].unwrap_or(NodeIdentifier::MAX) == identifier
            })
            .map(|i| *i)
            .collect()
    } 
}

impl<U: Clone> WFCBuilder<U> {
    pub fn get_index_of_base_node_identifier(&self, identifier: NodeIdentifier) -> Option<usize> {
        for (i, node) in self.base_nodes.iter().enumerate() {
            match node {
                BaseNodeTemplate::NumberRange {
                    identifier: ident, ..
                }
                | BaseNodeTemplate::Volume {
                    identifier: ident, ..
                }
                | BaseNodeTemplate::VolumeChild {
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
