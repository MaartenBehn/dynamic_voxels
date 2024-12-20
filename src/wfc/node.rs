use octa_force::glam::Vec3;

use crate::cgs_tree::tree::CSGTree;

use super::builder::{
    ActionTemplate, BaseNodeTemplate, NodeIdentifier, NumberRangeDefinesType, UserNodeTemplate, WFCBuilder,
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
    },

    User {
        data: U,
        attributes: Vec<usize>,
        on_collapse: Vec<Action>
    },
}

#[derive(Debug, Clone)]
pub enum NumberSetType {
    None,
    Amount, 
    Link,
}

#[derive(Debug, Clone)]
pub enum Action {
    TransformNumberSet {
        index: usize,
        func: fn(Vec<i32>) -> Vec<i32>,
    },
    TransformVolume {
        index: usize,
        func: fn(CSGTree) -> CSGTree,
    },
    TransformVolumeWithPosAttribute {
        volume_index: usize,
        attribute_index: usize,
        func: fn(CSGTree, Vec3) -> CSGTree,
    },
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

    pub fn add_user_node_by_identifier(&mut self, builder: &WFCBuilder<U>, identifier: NodeIdentifier) -> usize {
        let index = builder
            .get_index_of_user_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be User Node"));               
        

        self.add_user_node(builder, index)

    }

    pub fn add_user_node(&mut self, builder: &WFCBuilder<U>, index: usize) -> usize {
        let node_template = &builder.user_nodes[index];

        let index = self.nodes.len();
        self.nodes.push(Node::None);
        self.node_identifier.push(node_template.identifier);

        
        let mut children = vec![];
        for child in node_template.children.iter() {

            let res = self.add_base_node_by_identifier(builder, *child);
            children.push(res);
        }

        let node = Node::User {
            data: node_template.data.to_owned(),
            attributes: children,
            on_collapse: vec![],
        }; 

        self.nodes[index] = node;

        index
    }
    
    pub fn add_base_node_by_identifier(&mut self, builder: &WFCBuilder<U>, identifier: NodeIdentifier) -> usize {
        let index = builder
            .get_index_of_base_node_identifier(identifier)
            .expect(&format!("Identifier {identifier} must be Base Node"));
                        
        self.add_base_node(builder, index)
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

                let index = self.nodes.len();
                self.nodes.push(Node::None);
                self.node_identifier.push(identifier.to_owned());


                let mut vals = vec![];

                for i in *min..*max {
                    vals.push(i);
                }

                let mut children = vec![];

                let r#type = match *defines {
                    NumberRangeDefinesType::None => {
                        NumberSetType::None
                    }
                    NumberRangeDefinesType::Amount { of_node } => {
                        for i in 0..*max {
                            let res = self.add_user_node_by_identifier(builder, of_node);
                            children.push(res);
                        }
                        NumberSetType::Amount
                    }
                    NumberRangeDefinesType::Link { to_node } => {
                        children.push(to_node);
                        NumberSetType::Link
                    }
                };

                let node = Node::NumberSet { vals, children, r#type };

                self.nodes[index] = node;

                index
            }
            BaseNodeTemplate::Volume { identifier, csg,  .. } => {

                let index = self.nodes.len();
                self.nodes.push(Node::None);
                self.node_identifier.push(identifier.to_owned());

                let mut children = vec![]; 

                let node = Node::Volume {
                    csg: csg.to_owned(),
                    children,
                };

                self.nodes[index] = node;
                index
            }
            BaseNodeTemplate::VolumeChild { identifier, parent_identifier } => {
                
                let mut children = vec![]; 

                let node = Node::VolumeChild {
                    parent: *parent_identifier,
                    children,
                };

                let index = self.nodes.len();
                self.nodes.push(node);
                self.node_identifier.push(identifier.to_owned());
                index
            },
        }
    }

    pub fn link_nodes(&mut self, builder: &WFCBuilder<U>) {
        
        for (i, node) in self.nodes.to_owned().iter().enumerate() {
            
            match node {
                Node::NumberSet { r#type, children , .. } => {
                    match r#type {
                        NumberSetType::Link => {
                            let identifier = children[0];

                            match &mut self.nodes[i] {
                                Node::NumberSet { children, .. } => {
                                    children.clear();
                                }
                                _ => unreachable!()
                            }

                            for (test_index, test_identifier) in self.node_identifier.iter().enumerate() {
                                if Some(identifier) == *test_identifier {
                                    let test_node = &self.nodes[test_index];

                                    match test_node {
                                        Node::User { attributes, .. } => {
                                            if attributes.contains(&i) {
                                                continue;
                                            }
                                        },
                                        _ => {}
                                    }


                                    match &mut self.nodes[i] {
                                     Node::NumberSet { children, .. } => {
                                        children.push(test_index);
                                        }
                                        _ => unreachable!()
                                    }
                                } 
                            }
                        },
                        _ => {}
                    }
                },
                Node::VolumeChild { parent, .. } => {
                    for (test_index, test_identifier) in self.node_identifier.iter().enumerate() {
                        if Some(*parent) == *test_identifier {
                            match &mut self.nodes[test_index] {
                                Node::Volume { children, .. } 
                                | Node::VolumeChild { children, .. }=> {
                                    children.push(i);
                                },
                                _ => panic!("Volume Parent Identifier must be Volume"),
                            }
                            
                            // Set parent to the parent node index we found!
                            match &mut self.nodes[i] {
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
