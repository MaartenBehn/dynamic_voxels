use std::{iter, usize};

use crate::model_synthesis::relative_path::RelativePathTree;

use super::{builder::{BuilderAmmount, ModelSynthesisBuilder, NodeTemplateValue, IT}};

pub type TemplateIndex = usize;
pub const TEMPLATE_INDEX_ROOT: TemplateIndex = 0;
pub const AMMOUNT_PATH_INDEX: usize = 0;

#[derive(Debug, Clone)]
pub struct TemplateTree<I: IT> {
    pub nodes: Vec<TemplateNode<I>> 
}

#[derive(Debug, Clone)]
pub struct TemplateNode<I: IT> {
    pub identifier: I,
    pub value: NodeTemplateValue,
    pub ammount: TemplateAmmount,
    pub creates: Vec<NodeCreationInfo>,
    pub depends: Vec<TemplateIndex>,
    pub knows: Vec<TemplateIndex>,
    pub level: usize,
}

#[derive(Debug, Clone)]
pub enum TemplateAmmount{
    Root,
    NPer(usize, TemplateIndex),
    DefinedBy(TemplateIndex),
}

#[derive(Debug, Clone)]
pub struct NodeCreationInfo {
    pub index: TemplateIndex,
    pub dependecy_paths: RelativePathTree,
}

impl<I: IT> TemplateTree<I> {
    pub fn new_from_builder(builder: &ModelSynthesisBuilder<I>) -> TemplateTree<I> {
        let mut nodes = vec![TemplateNode { 
            identifier: I::default(), 
            value: NodeTemplateValue::Groupe {  }, 
            ammount: TemplateAmmount::Root,
            creates: vec![], 
            depends: vec![], 
            knows: vec![], 
            level: 0 
        }];

        // Create the nodes
        for builder_node in builder.nodes.iter() {
            let template_node = TemplateNode {
                identifier: builder_node.identifier,
                value: builder_node.value.to_owned(),
                ammount: TemplateAmmount::from_builder_ammount(builder_node.ammount, builder),
                creates: vec![],
                depends: vec![],
                knows: vec![],
                level: 0,
            };

            nodes.push(template_node);
        }

        // Set depends, knows and creates indecies 
        for (mut template_node_index, builder_node) in builder.nodes.iter().enumerate() {
            template_node_index += 1;

            let template_node = &nodes[template_node_index];
            let ammount_index = match template_node.ammount {
                TemplateAmmount::NPer(_, i) => i,
                TemplateAmmount::DefinedBy(i) => i,
                TemplateAmmount::Root => unreachable!(),
            };
            let mut depends = vec![ammount_index];
                   
            let ammount_idetifier = &nodes[ammount_index].identifier;
            &nodes[ammount_index].creates.push(NodeCreationInfo {
                index: template_node_index,
                dependecy_paths: RelativePathTree::default(),
            });

            
            for i in builder_node.depends.iter() {
                let depends_index = builder.get_node_index_by_identifier(*i) + 1;
                if !depends.contains(&depends_index) {
                    depends.push(depends_index);
                }

                &nodes[depends_index].creates.push(NodeCreationInfo {
                    index: template_node_index,
                    dependecy_paths: RelativePathTree::default(),
                });
            }

            let mut knows = vec![];
            for i in builder_node.knows.iter() {
                let knows_index = builder.get_node_index_by_identifier(*i) + 1;
                if !knows.contains(&knows_index) {
                    knows.push(knows_index);
                }
            }

            nodes[template_node_index].depends = depends;
            nodes[template_node_index].knows = knows;
        }
         
        let mut tree = TemplateTree {
            nodes,
        };

        // Set create paths und levels
        for i in 1..tree.nodes.len() {
            dbg!(&tree.nodes[i]);

            if tree.nodes[i].level == 0 {
                tree.set_level_of_node(i);
            }

            let identifier = tree.nodes[i].identifier;
            for j in 0..tree.nodes[i].creates.len() {
                tree.nodes[i].creates[j].dependecy_paths = RelativePathTree::get_paths_to_other_dependcies_from_parent(
                    &tree, 
                    i,
                    builder.nodes[tree.nodes[i].creates[j].index - 1].depends.iter()
                        .filter(|i| **i != identifier)
                        .map(|i| *i)
                        .collect())
            }
        }

        tree
    } 
    
    fn set_level_of_node(&mut self, index: usize) -> usize {
        let node = &self.nodes[index];

        let mut max_level = 0;
        for index in iter::empty()
            .chain(node.depends.to_owned().iter())
            .chain(node.knows.to_owned().iter()) {

            let mut level = self.nodes[*index].level; 

            if level == 0 {
                level = self.set_level_of_node(*index);
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;

        node_level
    } 
}

impl TemplateAmmount {
    fn from_builder_ammount<I: IT>(value: BuilderAmmount<I>, builder: &ModelSynthesisBuilder<I>) -> Self {
        match value {
            BuilderAmmount::OneGlobal => TemplateAmmount::NPer(1, TEMPLATE_INDEX_ROOT),
            BuilderAmmount::OnePer(i) => TemplateAmmount::NPer(1, builder.get_node_index_by_identifier(i) + 1),
            BuilderAmmount::NPer(n, i) => TemplateAmmount::NPer(n, builder.get_node_index_by_identifier(i) + 1),
            BuilderAmmount::DefinedBy(i) => TemplateAmmount::DefinedBy(builder.get_node_index_by_identifier(i) + 1),
        }
    }
}
