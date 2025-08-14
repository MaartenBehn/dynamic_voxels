
use std::{collections::VecDeque, usize};

use smallvec::SmallVec;

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{builder::{BuilderNode, ModelSynthesisBuilder, NodeBuilder}, template::{TemplateIndex, TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


#[derive(Debug, Clone, Default)]
pub struct RelativePathTree {
    pub steps: Vec<RelativePathStep>,
}

#[derive(Debug, Clone)]
pub struct RelativePathStep {
    pub into_index: TemplateIndex,
    pub children: Vec<usize>,
    pub up: bool, 
    pub leafs: SmallVec<[LeafType; 3]>,
}

#[derive(Debug, Clone)]
pub enum LeafType {
    Restricts(usize),
    Depends(usize),
    Knows(usize)
}

impl RelativePathTree {
    pub fn get_paths_to_other_dependcies<T: ModelGenerationTypes>(
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

        open_parent_paths.push_back((
            &tree.nodes[parent_index], 
            vec![RelativePathStep { 
                into_index: parent_index,
                children: vec![],
                up: true,
                leafs: SmallVec::new(),
            }]
        ));

        let mut path_tree = RelativePathTree { 
            steps: vec![],
        };

        let mut check_hit = |node: &TemplateNode<T>, path: &Vec<RelativePathStep>| {
            let restricts_res = restricts.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, _))|(i, *j));

            let depends_res = depends.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, _))|(i, *j));

            let knows_res = knows.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, _))|(i, *j));


            if restricts_res.is_some() || depends_res.is_some() || knows_res.is_some() {
                let leaf_index = path_tree.copy_path(node, path);

                if let Some((i, restrict_index)) = restricts_res {
                    restricts.swap_remove(i);
                    path_tree.steps[leaf_index].leafs.push(LeafType::Restricts(restrict_index));       
                }

                if let Some((i, depends_index)) = depends_res {
                    depends.swap_remove(i);
                    path_tree.steps[leaf_index].leafs.push(LeafType::Depends(depends_index));        
                }

                if let Some((i, knows_index)) = knows_res {
                    knows.swap_remove(i);
                    path_tree.steps[leaf_index].leafs.push(LeafType::Knows(knows_index));       
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
                        leafs: SmallVec::new(),
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
                        leafs: SmallVec::new(),
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
                        leafs: SmallVec::new(),
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
        if self.steps.is_empty() {
            for step in path {
                self.steps.push(step.clone());
            } 
            return self.steps.len() - 1;
        } 
        assert!(path[0].into_index == self.steps[0].into_index, "We should allways first go into the parent");
        
        let mut insert_index = 0;
        let mut path_index = 1;

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
                break;
            }
        }

        while path_index < path.len() {

            let new_index = self.steps.len();
            self.steps.push(path[path_index].clone());
            self.steps[insert_index].children.push(new_index);

            insert_index = new_index;
            path_index += 1;
        }

        insert_index
    }

}


