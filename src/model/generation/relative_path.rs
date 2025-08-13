
use std::{collections::VecDeque, usize};

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, NodeBuilder}, template::{TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


#[derive(Debug, Clone, Default)]
pub struct RelativePathTree {
    pub starts: Vec<usize>,
    pub steps: Vec<RelativePathStep>,
}

#[derive(Debug, Clone)]
pub struct RelativePathStep {
    pub into_index: TemplateIndex,
    pub children: Vec<usize>,
    pub up: bool, 
    pub leaf: LeafType,
}

#[derive(Debug, Clone)]
pub enum LeafType {
    None,
    Restricts(usize),
    Depends(usize),
    Knows(usize)
}

impl RelativePathTree {
    pub fn get_paths_to_other_dependcies_from_parent<T: ModelGenerationTypes>(
        tree: &TemplateTree<T>, 
        parent_index: TemplateIndex, 
        restricts: &[TemplateIndex], 
        depends: &[TemplateIndex], 
        knows: &[TemplateIndex]
    ) -> RelativePathTree {
        let mut restricts = restricts.iter().copied().enumerate().collect::<Vec<_>>();
        let mut depends = depends.iter().copied().enumerate().collect::<Vec<_>>();
        let mut knows = knows.iter().copied().enumerate().collect::<Vec<_>>();

        let mut open_child_paths: VecDeque<(&TemplateNode<T>, Vec<RelativePathStep>)> = VecDeque::new();
        let mut open_parent_paths: VecDeque<(&TemplateNode<T>, Vec<RelativePathStep>)> = VecDeque::new();
        open_parent_paths.push_back((&tree.nodes[parent_index], vec![]));
        let mut path_tree = RelativePathTree { 
            steps: vec![],
            starts: vec![],
        };

        let mut check_hit = |node: &TemplateNode<T>, path: &Vec<RelativePathStep>| {
            if let Some((i, restrict_index, restrict)) = restricts.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, k))|(i, *j, *k))
            {
                restricts.swap_remove(i);
                
                if parent_index != restrict {
                    let leaf_index = path_tree.copy_path(node, path); 

                    path_tree.steps[leaf_index].leaf = LeafType::Restricts(restrict_index);       
                    return
                }
            }

            if let Some((i, depends_index, depend)) = depends.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, k))|(i, *j, *k))
            {
                depends.swap_remove(i);
                
                if parent_index != depend {
                    let leaf_index = path_tree.copy_path(node, path); 

                    path_tree.steps[leaf_index].leaf = LeafType::Depends(depends_index);   
                    return
                }
            }
            
            if let Some((i, knows_index, know)) = knows.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, k))|(i, *j, *k))
            {
                knows.swap_remove(i);
                 
                if parent_index != know {
                    let leaf_index = path_tree.copy_path(node, path); 

                    path_tree.steps[leaf_index].leaf = LeafType::Knows(knows_index); 
                    return
                }
           } 
        };
             
        loop {
            if let Some((node, path)) = open_child_paths.pop_front() {
                check_hit(node, &path);
                
                for index in node.dependend.iter() {
                    let child = &tree.nodes[*index];

                    let mut child_path = path.clone();
                    child_path.push(RelativePathStep { 
                        into_index: *index,
                        children: vec![],
                        up: false,
                        leaf: LeafType::None,
                    });

                    open_child_paths.push_back((child, child_path));
                }
                
            } else if let Some((node, path)) = open_parent_paths.pop_front() {
                check_hit(node, &path);

                for index in node.dependend.iter() {
                    let child = &tree.nodes[*index];

                    let mut child_path = path.clone();
                    child_path.push(RelativePathStep { 
                        into_index: *index,
                        children: vec![],
                        up: false,
                        leaf: LeafType::None,
                    });

                    open_child_paths.push_back((child, child_path));
                }

                for parent_index in node.depends.iter() {
                    let parent = &tree.nodes[*parent_index];

                    let mut parent_path = path.clone();
                    parent_path.push(RelativePathStep { 
                        into_index: *parent_index,
                        children: vec![],
                        up: true,
                        leaf: LeafType::None,
                    });
                    open_parent_paths.push_back((parent, parent_path));
                }
                
            } else {
                break;
            }
        }

        path_tree
    }

    fn copy_path<T: ModelGenerationTypes>(
        &mut self, 
        node: &TemplateNode<T>, 
        path: &Vec<RelativePathStep>, 
    ) -> usize {
        assert!(!path.is_empty(), "Relative path can not be empty because we ignore the node itself in the dependencies!");

        let (mut insert_index, mut instert_from_step) = if let Some(index) =  self.starts.iter()
            .find(|i| { 
                self.steps[**i].into_index == path[0].into_index 
            }) {

            let mut insert_index = *index;
            let mut path_index = 0;

            loop {
                let path_step = &path[path_index];
                let tree_step = &self.steps[insert_index];

                if let Some(index) = tree_step.children.iter()
                    .find(|i| { 
                        self.steps[**i].into_index == path_step.into_index 
                    }) {

                    insert_index = *index;
                    path_index += 1;
                } else {
                    path_index += 1;
                    break;
                }
            }

            (insert_index, path_index)
        } else {
            let index = self.steps.len();
            self.steps.push(path[0].clone());
            self.starts.push(index);
            (index, 1)
        };
        
        
        while instert_from_step < path.len() {
            
            let new_index = self.steps.len();
            self.steps.push(path[instert_from_step].clone());
            self.steps[insert_index].children.push(new_index);

            insert_index = new_index;
            instert_from_step += 1;
        }

        insert_index
    }

}


