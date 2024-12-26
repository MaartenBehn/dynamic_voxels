use std::{ops::Not, usize};

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::{Mat4, Vec3};

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE};

use super::{builder::{BaseNodeTemplate, NodeIdentifier, NodeIdentifierNone, NumberRangeDefinesType, WFCBuilder}, node::{Node, NumberSetType, WFC}};



impl<U: Clone> WFC<U> {
    pub fn build_user_node_by_identifier(&mut self, builder: &WFCBuilder<U>, identifier: NodeIdentifier) -> (usize, bool) {
        let index = builder
            .get_index_of_user_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be User Node"));

        self.build_user_node(builder, index)
 
    }

    pub fn build_user_node(&mut self, builder: &WFCBuilder<U>, index: usize) -> (usize, bool) {
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
                BaseNodeTemplate::NumberRange { defines, .. } => {
                    match defines {
                        NumberRangeDefinesType::Amount { .. } => {
                            new_level_children.push(i);
                            continue;
                        },
                        _ => {}
                    }
                },
                _ => {}
            } 


            let (child_index, child_valid) = self.build_base_node(builder, i);
            children.push(child_index);
            valid &= child_valid;
        }

        for i in new_level_children {
            let (child_index, child_valid) = self.build_base_node(builder, i);
            children.push(child_index);
            valid &= child_valid;
        }

        let node = Node::User {
            data: node_template.data.to_owned(),
            attributes: children,
        }; 

        self.nodes[index] = node;

        (index, valid)
    }

    pub fn build_base_node_by_identifier(&mut self, builder: &WFCBuilder<U>, identifier: NodeIdentifier) -> (usize, bool) {
        let index = builder
            .get_index_of_base_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be Base Node"));

        self.build_base_node(builder, index)
    }

    pub fn build_base_node(&mut self, builder: &WFCBuilder<U>, index: usize) -> (usize, bool) {
        let node_template = &builder.base_nodes[index];
        
        match node_template {
            BaseNodeTemplate::NumberRange { identifier, min, max, defines } => {

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
                    NumberRangeDefinesType::None => {
                        NumberSetType::None
                    }
                    NumberRangeDefinesType::Amount { of_node } => {
                        let (new_valid, new_children) = self.build_ammount_level(builder, of_node, &vals, index);
                        vals = vec![new_children.len() as i32];
                        children = new_children;
                        valid &= new_valid;

                        NumberSetType::Amount
                    }
                    NumberRangeDefinesType::Link { to_node } => {
                        self.link_data[index] = to_node;
                        NumberSetType::Link
                    }
                };

                let node = Node::NumberSet { vals, children, r#type };

                self.nodes[index] = node;

                (index, valid)
            },
            BaseNodeTemplate::Volume { identifier, csg } => {
                let index = self.nodes.len();
                self.nodes.push(Node::None);
                self.node_identifier.push(identifier.to_owned());
                self.link_data.push(NodeIdentifierNone);

                let mut children = vec![]; 

                let node = Node::Volume {
                    csg: csg.to_owned(),
                    children,
                };

                self.nodes[index] = node;

                (index, true)
            },
            BaseNodeTemplate::VolumeChild { identifier, parent_identifier, on_collapse } => {
                let mut children = vec![]; 

                let node = Node::VolumeChild {
                    parent: usize::MAX,
                    children,
                    on_collapse: *on_collapse,
                };

                let index = self.nodes.len();
                self.nodes.push(node);
                self.node_identifier.push(identifier.to_owned());
                self.link_data.push(*parent_identifier);
                
                (index, true)
            },
        }
    }

    fn build_ammount_level(&mut self, builder: &WFCBuilder<U>, of_node_identifier: usize, vals: &[i32], index: usize) -> (bool, Vec<usize>) {
        
        let seed = fastrand::u64(0..1000);
                        
        let perm = Permutation::new( vals.len() as _, seed, DefaultBuildHasher::new());
        
        let mut valid = true;
        let mut children = vec![];
        for perm_index in perm.iter() {

            let amount = vals[perm_index as usize];
            for i in 0..amount {
                let (child_index, child_valid) = self.build_user_node_by_identifier(builder, of_node_identifier); 
                
                children.push(child_index);
                valid &= child_valid;
            }

            for i in index..self.nodes.len() {
                valid &= self.link_node(i);
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

    pub fn link_node(&mut self, index: usize) -> bool {
        let node = &self.nodes[index];

        match node {
            Node::NumberSet { r#type, children , vals } => {
                match r#type {
                    NumberSetType::Link => {
                        if !children.is_empty() {
                            return true;
                        }

                        let vals_min = vals[0] as usize;

                        let identifier = self.link_data[index];
                            
                        let mut add_counter = 0;
                        for (test_index, test_identifier) in self.node_identifier.iter().enumerate() {
                            if Some(identifier) == *test_identifier {
                                let test_node = &self.nodes[test_index];

                                match test_node {
                                    Node::User { attributes, .. } => {
                                        if attributes.contains(&index) {
                                            continue;
                                        }
                                    },
                                    _ => {}
                                }


                                match &mut self.nodes[index] {
                                    Node::NumberSet { children, .. } => {
                                        children.push(test_index);
                                        add_counter += 1;
                                    }
                                    _ => unreachable!()
                                }
                            } 
                        }
                        
                        // There are not enought children for a single val.
                        if vals_min > add_counter {
                            return false;
                        }
                        
                        // Delete all vals that are to large for the amount of possible
                        // children.
                        match &mut self.nodes[index] {
                            Node::NumberSet { vals, .. } => {
                                *vals = vals.to_owned()
                                    .into_iter()
                                    .filter(|val| *val as usize <= add_counter)
                                    .collect();
                            },
                            _ => unreachable!()
                        }
                    },
                    _ => {}
                }
            },
            Node::VolumeChild { parent, .. } => {
                if *parent != usize::MAX {
                    return true;
                }
                    
                let identifier = self.link_data[index];
                
                dbg!(identifier);

                for (test_index, test_identifier) in self.node_identifier.iter().enumerate() {
                    if Some(identifier) == *test_identifier {
                        match &mut self.nodes[test_index] {
                            Node::Volume { children, .. } 
                            | Node::VolumeChild { children, .. }=> {
                                children.push(index);
                            },
                            _ => panic!("Volume Parent Identifier must be Volume"),
                        }
                        
                        // Set parent to the parent node index we found!
                        match &mut self.nodes[index] {
                            Node::VolumeChild { parent, .. } => {
                                *parent = test_index;
                            },
                            _ => unreachable!(),
                        }

                        break;
                    }
                }                         
            },
            _ => {}
        }

        true
    }

    pub fn build_collapse(&mut self, index: usize) -> bool {
        let node = &self.nodes[index];

        match node {
            Node::None => panic!("When collapsing there shouldn't be any None Nodes"),
            Node::Number { .. } => {},
            Node::NumberSet { vals, r#type, children } => {
                
                match r#type {
                    NumberSetType::None => panic!("When collapsing the NumberSettype can't be None"),
                    NumberSetType::Amount => {
                        assert!(vals.len() == 1, "Vals len of Ammount Nodes should be one when collapsing a part of the tree.")
                    },
                    NumberSetType::Link => {

                        let selected_val_index = fastrand::usize(0..vals.len());
                        let selected_val = vals[selected_val_index];

                        let seed = fastrand::u64(0..1000);
                        
                        let new_chilrend: Vec<usize> = Permutation::new(
                            children.len() as _, 
                            seed, 
                            DefaultBuildHasher::new())
                            .iter()
                            .take(selected_val as _)
                            .map(|i| {
                                children[i as usize]
                            })
                            .collect();

                        match &mut self.nodes[index] {
                            Node::NumberSet { children, .. } => {
                                *children = new_chilrend;
                            },
                            _ => unreachable!()
                        } 

                        match &mut self.nodes[index] {
                            Node::NumberSet { vals, .. } => {
                                vals.clear();
                                vals.push(selected_val);
                            },
                            _ => unreachable!()
                        }
                    },
                }
            },
            Node::Pos { .. } => {},
            Node::Volume { .. } => {},
            Node::VolumeChild { parent, on_collapse, .. } => {
                let parent_index = *parent;
                let on_collapse = *on_collapse;

                let parent_node = &mut self.nodes[parent_index];                

                let pos = match parent_node {   
                    Node::Volume { csg, .. } => {
                        let pos = csg.find_valid_pos(0.1);

                        if pos.is_none() {
                            return false;
                        }
                        let pos = pos.unwrap();

                        on_collapse(csg, pos);
                                                
                        pos
                    },
                    _ => panic!("VolumeChild parent mus be index of Volume.")
                };

                self.nodes[index] = Node::Pos { pos }; 
            },
            Node::User { attributes, .. } => {
                for attribute in attributes.to_owned().into_iter() {
                    let valid = self.collapse(attribute);
                            
                    if !valid {
                        return false;
                    } 
                }
            },
        }

        return true;
    }
}
