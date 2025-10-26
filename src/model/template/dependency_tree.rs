
use std::{collections::VecDeque, usize};

use itertools::Itertools;
use smallvec::{SmallVec, smallvec};

use crate::{model::composer::build::BS, util::{number::Nu, vector::Ve}};

use super::{nodes::TemplateNode, ComposeTemplate, TemplateIndex};

#[derive(Debug, Clone, Default)]
pub struct DependencyPath {
    pub steps: Vec<DependencyPathStep>,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyTree {
    pub steps: Vec<DependencyTreeStep>,
}

#[derive(Debug, Clone)]
pub struct DependencyPathStep {
    pub into_index: TemplateIndex,
    pub up: bool, 
}

#[derive(Debug, Clone)]
pub struct DependencyTreeStep {
    pub into_index: TemplateIndex,
    pub children: Vec<usize>,
    pub up: bool, 
    pub leaf: Option<usize>,
}

pub fn get_dependency_tree_and_loop_paths<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>>(
    template: &ComposeTemplate<V2, V3, T, B>, 
    parent_index: TemplateIndex, 
    depends: &[TemplateIndex],
    dependend: &[TemplateIndex], 
    depends_loop: &[(TemplateIndex, DependencyPath)], 
) -> (DependencyTree, SmallVec<[(TemplateIndex, DependencyPath); 4]>) {
    
    let mut open_child_paths: VecDeque<(&TemplateNode, DependencyPath)> = VecDeque::new();
    let mut open_parent_paths: VecDeque<(&TemplateNode, DependencyPath)> = VecDeque::new();

    for dependend_child in dependend {
        open_child_paths.push_back((
            &template.nodes[*dependend_child], 
            DependencyPath {
                steps: vec![DependencyPathStep { 
                    into_index: *dependend_child,
                    up: false,
                }]
            }
        ));
    }

    open_parent_paths.push_back((
        &template.nodes[parent_index], 
        DependencyPath {
            steps: vec![DependencyPathStep { 
                into_index: parent_index,
                up: true,
            }]
        }
    ));

    let mut depends = depends.iter()
        .copied()
        .enumerate()
        .collect_vec();

    let mut depends_loop = depends_loop.iter()
        .map(|d| d.0)
        .collect_vec();

    let mut path_tree = DependencyTree { 
        steps: vec![],
    };
    let mut loop_paths = smallvec![];
 
    while !depends.is_empty() {
        if let Some((node, path)) = open_child_paths.pop_front() {
            check_hit(node, &path, &mut depends, &mut depends_loop, &mut path_tree, &mut loop_paths);

            for index in node.dependend.iter() {
                let child = &template.nodes[*index];

                let mut child_path = path.clone();
                child_path.steps.push(DependencyPathStep { 
                    into_index: *index,
                    up: false,
                });

                open_child_paths.push_back((child, child_path));
            }

        } else if let Some((node, path)) = open_parent_paths.pop_front() {
            check_hit(node, &path, &mut depends, &mut depends_loop, &mut path_tree, &mut loop_paths);

            for index in node.dependend.iter() {
                let child = &template.nodes[*index];

                let mut child_path = path.clone();
                child_path.steps.push(DependencyPathStep { 
                    into_index: *index,
                    up: false,
                });

                open_child_paths.push_back((child, child_path));
            }

            for parent_index in node.depends.iter() {
                let parent = &template.nodes[*parent_index];

                let mut parent_path = path.clone();
                parent_path.steps.push(DependencyPathStep { 
                    into_index: *parent_index,
                    up: true,
                });
                open_parent_paths.push_back((parent, parent_path));
            }

        } else {
            break;
        }
    }

    (path_tree, loop_paths)
}

pub fn check_hit(
    node: &TemplateNode, 
    path: &DependencyPath,
    depends: &mut Vec<(usize, usize)>,
    depends_loop: &mut Vec<usize>,
    path_tree: &mut DependencyTree,
    loop_paths: &mut SmallVec<[(TemplateIndex, DependencyPath); 4]>,
) {
    let depends_res = depends.iter()
        .enumerate()
        .find(|(_, (_, i))| *i == node.index)
        .map(|(i, (j, _))|(i, *j));

    if let Some((i, depends_index)) = depends_res {
        depends.swap_remove(i);
        path_tree.copy_path(node, path, depends_index);
    }

    let depends_res = depends_loop.iter()
        .enumerate()
        .find(|(_, i)| **i == node.index)
        .map(|(i, j)| (i, *j));

    if let Some((i, depends_index)) = depends_res {
        depends_loop.swap_remove(i);
        loop_paths.push((depends_index, path.clone()));
    }
}

impl DependencyTree {
    fn copy_path(
        &mut self, 
        node: &TemplateNode, 
        path: &DependencyPath,
        depends_index: usize,
    ) {
        assert!(path.steps[0].up, "DependencyTree must allways first go into the parent");

        if self.steps.is_empty() {
            for step in path.steps.iter() {
                self.steps.push(DependencyTreeStep { 
                    into_index: step.into_index, 
                    children: vec![], 
                    up: step.up, 
                    leaf: None, 
                });
            }
            
            let last_index =  self.steps.len() - 1; 
            self.steps[last_index].leaf = Some(depends_index);
            return
        } 
        
        let mut insert_index = 0;
        let mut path_index = 1;

        loop {
            let path_step = &path.steps[path_index];
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

        while path_index < path.steps.len() {

            let new_index = self.steps.len();
            let step = &path.steps[path_index]; 
            self.steps.push(DependencyTreeStep { 
                into_index: step.into_index, 
                children: vec![], 
                up: step.up, 
                leaf: None 
            });
            self.steps[insert_index].children.push(new_index);

            insert_index = new_index;
            path_index += 1;
        }

        self.steps[insert_index].leaf = Some(depends_index);
    }
}
