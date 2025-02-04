use core::panic;
use std::{iter, usize};

use octa_force::glam::Vec3;

use crate::cgs_tree::tree::CSGTree;

use super::builder::{
    BaseNodeTemplate, NodeIdentifier, NumberRangeDefinesType, UserNodeTemplate, WFCBuilder,
};

#[derive(Debug, Clone)]
pub enum Node<U: Clone> {
    None, 
    NumberSet {
        vals: Vec<i32>,
        r#type: NumberSetType,
        children: Vec<usize>,
    },

    Pos {
        pos: Vec3,
        on_collapse: fn(&mut WFC<U>, &mut U) -> (Vec3, bool)
    }, 

    User {
        attributes: Vec<usize>,
        on_generate: fn(&mut WFC<U>, &mut U),
    },
}

#[derive(Debug, Clone)]
pub enum NumberSetType {
    None,
    Amount,
}

#[derive(Debug, Clone)]
pub struct WFC<U: Clone> {
    pub nodes: Vec<Node<U>>,
    pub node_identifier: Vec<Option<NodeIdentifier>>,
}

impl<U: Clone> WFC<U> {
    pub(crate) fn new(builder: &WFCBuilder<U>, user_data: &mut U) -> Self {
        let mut wfc = WFC {
            nodes: vec![],
            node_identifier: vec![],
        };

        wfc.build_user_node(builder, 0, user_data);
        wfc
    }

    pub fn show(&mut self, user_data: &mut U) {
        self.show_node(user_data, 0);
    }

    fn show_node(&mut self, user_data: &mut U, index: usize) {
        match &self.nodes[index] {
            Node::NumberSet {
                r#type, children, ..
            } => match r#type {
                NumberSetType::Amount => {
                    let children = children.to_owned();

                    for child in children {
                        self.show_node(user_data, child);
                    }
                }
                _ => {}
            },
            Node::User {
                attributes,
                on_generate,
            } => {
                let attributes = attributes.to_owned();

                on_generate(self, user_data);

                for attribute in attributes {
                    self.show_node(user_data, attribute);
                }
            }
            _ => {}
        }
    }

    pub fn get_children_with_identifier(
        &mut self,
        index: usize,
        identifier: NodeIdentifier,
    ) -> Vec<usize> {
        match &self.nodes[index] {
            Node::None => panic!("get children none should never be none"),
            Node::NumberSet { children, .. }
           | Node::User {
                attributes: children,
                ..
            } => children,
            Node::Pos { pos, on_collapse } => panic!("Pos has no children"),
        }
        .iter()
        .filter(|i| self.node_identifier[**i].unwrap_or(NodeIdentifier::MAX) == identifier)
        .map(|i| *i)
        .collect()
    }
}

impl<U: Clone> WFCBuilder<U> {
    pub fn get_index_of_base_node_identifier(&self, identifier: NodeIdentifier) -> Option<usize> {
        for (i, node) in self.base_nodes.iter().enumerate() {
            match node {
                BaseNodeTemplate::NumberRange { identifier: ident, .. }
                | BaseNodeTemplate::Pos { identifier: ident, .. }
                => {
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
