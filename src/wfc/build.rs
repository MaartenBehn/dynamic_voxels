use std::{ops::Not, usize};

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::{Mat4, Vec3};

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE};

use super::{
    builder::{
        BaseNodeTemplate, NodeIdentifier, NodeIdentifierNone, NumberRangeDefinesType, WFCBuilder,
    },
    node::{Node, NumberSetType, WFC},
};

impl<U: Clone> WFC<U> {
    pub fn build_user_node_by_identifier(
        &mut self,
        builder: &WFCBuilder<U>,
        identifier: NodeIdentifier, 
        user_data: &mut U
    ) -> (usize, bool) {
        let index = builder
            .get_index_of_user_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be User Node"));

        self.build_user_node(builder, index, user_data)
    }

    pub fn build_user_node(&mut self, builder: &WFCBuilder<U>, index: usize, user_data: &mut U) -> (usize, bool) {
        let node_template = &builder.user_nodes[index];

        let index = self.nodes.len();
        self.nodes.push(Node::None);
        self.node_identifier.push(node_template.identifier);
        self.link_data.push(NodeIdentifierNone);

        let mut children = vec![];
        let mut valid = true;

        let mut new_level_children = vec![];
        for child in node_template.children.iter() {
            let i = builder
                .get_index_of_base_node_identifier(*child)
                .expect(&format!("Identifier {child} must be Base Node"));

            match &builder.base_nodes[i] {
                BaseNodeTemplate::NumberRange { defines, .. } => match defines {
                    NumberRangeDefinesType::Amount { .. } => {
                        new_level_children.push(i);
                        continue;
                    }
                    _ => {}
                },
                _ => {}
            }

            let (child_index, child_valid) = self.build_base_node(builder, i, user_data);
            children.push(child_index);
            valid &= child_valid;
        }

        for i in new_level_children {
            let (child_index, child_valid) = self.build_base_node(builder, i, user_data);
            children.push(child_index);
            valid &= child_valid;
        }

        let node = Node::User {
            attributes: children,
            on_generate: node_template.on_generate,
        };

        self.nodes[index] = node;

        (index, valid)
    }

    pub fn build_base_node_by_identifier(
        &mut self,
        builder: &WFCBuilder<U>,
        identifier: NodeIdentifier,
        user_data: &mut U
    ) -> (usize, bool) {
        let index = builder
            .get_index_of_base_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be Base Node"));

        self.build_base_node(builder, index, user_data)
    }

    pub fn build_base_node(&mut self, builder: &WFCBuilder<U>, index: usize, user_data: &mut U) -> (usize, bool) {
        let node_template = &builder.base_nodes[index];

        match node_template {
            BaseNodeTemplate::NumberRange {
                identifier,
                min,
                max,
                defines,
            } => {
                let index = self.nodes.len();
                self.nodes.push(Node::None);
                self.node_identifier.push(identifier.to_owned());
                self.link_data.push(NodeIdentifierNone);

                let mut vals = vec![];

                for i in *min..*max {
                    vals.push(i);
                }

                let mut children = vec![];
                let mut valid = true;

                let r#type = match *defines {
                    NumberRangeDefinesType::None => NumberSetType::None,
                    NumberRangeDefinesType::Amount { of_node } => {
                        let (new_valid, new_children) =
                            self.build_ammount_level(builder, of_node, &vals, index, user_data);
                        vals = vec![new_children.len() as i32];
                        children = new_children;
                        valid &= new_valid;

                        NumberSetType::Amount
                    }
                };

                let node = Node::NumberSet {
                    vals,
                    children,
                    r#type,
                };

                self.nodes[index] = node;

                (index, valid)
            }
            BaseNodeTemplate::Pos { 
                identifier, 
                on_collapse 
            } => {
                
                let index = self.nodes.len();
                let (pos, valid) = on_collapse(self, user_data);
                self.nodes.push(Node::Pos { 
                    pos, 
                    on_collapse: *on_collapse,
                });

                (index, valid)
            },  
        }
    }

    fn build_ammount_level(
        &mut self,
        builder: &WFCBuilder<U>,
        of_node_identifier: usize,
        vals: &[i32],
        index: usize,
        user_data: &mut U
    ) -> (bool, Vec<usize>) {
        let seed = fastrand::u64(0..1000);

        let perm = Permutation::new(vals.len() as _, seed, DefaultBuildHasher::new());

        let mut valid = true;
        let mut children = vec![];
        for perm_index in perm.iter() {
            let amount = vals[perm_index as usize];
            for i in 0..amount {
                let (child_index, child_valid) =
                    self.build_user_node_by_identifier(builder, of_node_identifier, user_data);

                children.push(child_index);
                valid &= child_valid;
            }

            for child in children.iter() {
                self.build_collapse(*child);
            }

            if valid {
                break;
            } else {
                // Delete all nodes that where created
                self.nodes.truncate(index + 1);
                self.node_identifier.truncate(index + 1);
                self.link_data.truncate(index + 1);

                // Reset values
                valid = true;
                children.clear();
            }
        }

        (valid, children)
    }

    pub fn build_collapse(&mut self, index: usize) -> bool {
        let node = &self.nodes[index];

        match node {
            Node::None => panic!("When collapsing there shouldn't be any None Nodes"),
            Node::NumberSet {
                vals,
                r#type,
                children,
            } => match r#type {
                NumberSetType::None => {},
                NumberSetType::Amount => {
                    assert!(vals.len() == 1, "Vals len of Ammount Nodes should be one when collapsing a part of the tree.")
                }
            },
            Node::Pos { .. } => {}
            
            Node::User { attributes, .. } => {
                for attribute in attributes.to_owned().into_iter() {
                    let valid = self.build_collapse(attribute);

                    if !valid {
                        return false;
                    }
                }
            }
        }

        return true;
    }
}
