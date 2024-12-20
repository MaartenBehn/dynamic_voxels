use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::{Mat4, Vec3};

use crate::cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE};

use super::node::{Node, NumberSetType, WFC};



impl<U: Clone> WFC<U> {

    pub fn collapse(&mut self, index: usize) -> bool {
        
        let valid = self.collapse_node(index);

        if !valid {
            return false;
        }

        let node = &self.nodes[index];
        match node {
            Node::None => panic!("When collapsing there shouldn't be any None Nodes"),
            Node::Number { .. } => {},
            Node::NumberSet { vals, r#type, children } => {
                match r#type {
                    NumberSetType::None => panic!("When collapsing the NumberSettype can't be None"),
                    NumberSetType::Amount => {
                        
                        let mut num_valid = 0;
                        for (i, child) in children.to_owned().into_iter().enumerate().rev() {
                            let valid = self.collapse(child);

                            if !valid {
                                match &mut self.nodes[index] {
                                    Node::NumberSet { children, .. } => {
                                        children.remove(i);
                                    },
                                    _ => unreachable!()
                                }
                            } else {
                                num_valid += 1;
                            } 
                        }

                        match &mut self.nodes[index] {
                            Node::NumberSet { vals, .. } => {
                                *vals = vals.iter()
                                    .filter_map(|val| {
                                        if *val <= num_valid {
                                            Some(*val)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                if vals.is_empty() {
                                    return false;
                                }
                            },
                            _ => unreachable!()
                        }

                    },
                    NumberSetType::Link => {},
                }
            },
            Node::Pos { .. } => {},
            Node::Volume { .. } => {},
            Node::VolumeChild { .. } => {},
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
    
    pub fn collapse_node(&mut self, index: usize) -> bool {
        let node = &self.nodes[index];

        match node {
            Node::None => panic!("When collapsing there shouldn't be any None Nodes"),
            Node::Number { .. } => {},
            Node::NumberSet { vals, r#type, children } => {
                let selected_val_index = fastrand::usize(0..vals.len());
                let selected_val = vals[selected_val_index];

                match r#type {
                    NumberSetType::None => panic!("When collapsing the NumberSettype can't be None"),
                    NumberSetType::Amount => {
                        
                        match &mut self.nodes[index] {
                            Node::NumberSet { children, .. } => {
                                children.truncate(selected_val as usize);
                        },
                        _ => unreachable!()
                        }
                    },
                    NumberSetType::Link => {

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

                    },
                }

                match &mut self.nodes[index] {
                    Node::NumberSet { vals, .. } => {
                        vals.clear();
                        vals.push(selected_val);
                    },
                    _ => unreachable!()
                }
            },
            Node::Pos { .. } => {},
            Node::Volume { csg, children } => {},
            Node::VolumeChild { parent, .. } => {
                let parent_index = *parent;

                let parent_node = &mut self.nodes[parent_index];                

                let pos = match parent_node {   
                    Node::Volume { csg, .. } => {
                        let pos = csg.find_valid_pos(0.1);

                        if pos.is_none() {
                            return false;
                        }
                        let pos = pos.unwrap();

                        // Later belongs in user func
                        let mut tree = CSGTree::new();
                        tree.nodes.push(CSGNode::new(CSGNodeData::Sphere(
                            Mat4::from_scale_rotation_translation(
                                Vec3::ONE * 0.1,
                                octa_force::glam::Quat::from_euler(octa_force::glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                                pos,
                            ),
                            MATERIAL_NONE,
                        )));
                        csg.append_tree_with_remove(tree);
                        csg.set_all_aabbs(0.0);
                        
                        pos
                    },
                    _ => panic!("VolumeChild parent mus be index of Volume.")
                };

                self.nodes[index] = Node::Pos { pos }; 
            },
            Node::User { .. } => {},
        }

        return true;
    }
}
