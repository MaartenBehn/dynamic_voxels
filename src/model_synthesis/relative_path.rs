use std::usize;

use super::{builder::{BuilderNode, ModelSynthesisBuilder, NodeBuilder, BU, IT}, template::{TemplateIndex, TemplateNode, TemplateTree}};


#[derive(Debug, Clone, Default)]
pub struct RelativePathTree {
    steps: Vec<RelativePathStep>,
}

#[derive(Debug, Clone)]
pub struct RelativePathStep {
    into_index: TemplateIndex,
    children: Vec<usize>,
    up: bool, 
}

impl RelativePathTree {
    pub fn get_paths_to_other_dependcies_from_parent<I: IT>(tree: &TemplateTree<I>, parent_index: TemplateIndex, mut other_dependencies: Vec<I>) -> RelativePathTree {
        let base_steps = vec![RelativePathStep { 
            into_index: parent_index, 
            children: vec![], 
            up: false 
        }];

        let mut open_child_paths: Vec<(&TemplateNode<I>, Vec<RelativePathStep>)> = vec![];
        let mut open_parent_paths: Vec<(&TemplateNode<I>, Vec<RelativePathStep>)> = vec![(&tree.nodes[parent_index], base_steps.clone())];
        let mut path_tree = RelativePathTree { steps: base_steps };


        let mut check_hit = |node: &TemplateNode<I>, path: &Vec<RelativePathStep>| {
            if let Some(i) = other_dependencies.iter().position(|i| *i == node.identifier) {
                other_dependencies.swap_remove(i);    
                
                let mut insert_step = 0;
                let mut insert_index = 0;
                loop {
                    let path_step = &path[insert_step];
                    let tree_step = &path_tree.steps[insert_index];

                    if let Some(index) = tree_step.children.iter().find(|i| { path_tree.steps[**i].into_index == path_step.into_index }) {
                        insert_index = *index;
                        insert_step += 1;
                    } else {
                        insert_step += 1;
                        break;
                    }
                }

                while insert_step < path.len() {
                    
                    let new_index = path_tree.steps.len();
                    path_tree.steps.push(path[insert_step].clone());
                    path_tree.steps[insert_index].children.push(new_index);

                    insert_index = new_index;
                    insert_step += 1;
                }
            }
        };
 
        loop {
            if let Some((node, path)) = open_child_paths.pop() {
                check_hit(node, &path);
                
                for child_info in node.creates.iter() {
                    let child = &tree.nodes[child_info.index];

                    let mut child_path = path.clone();
                    child_path.push(RelativePathStep { 
                        into_index: child_info.index,
                        children: vec![],
                        up: false,
                    });

                    open_child_paths.push((child, child_path));
                }
                
                } else if let Some((node, path)) = open_parent_paths.pop() {
                check_hit(node, &path);

                for child_info in node.creates.iter() {
                    let child = &tree.nodes[child_info.index];

                    let mut child_path = path.clone();
                    child_path.push(RelativePathStep { 
                        into_index: child_info.index,
                        children: vec![],
                        up: false,
                    });

                    open_child_paths.push((child, child_path));
                }

                for parent_index in node.depends.iter() {
                    let parent = &tree.nodes[*parent_index];

                    let mut parent_path = path.clone();
                    parent_path.push(RelativePathStep { 
                        into_index: *parent_index,
                        children: vec![],
                        up: true,
                    });
                    open_parent_paths.push((parent, parent_path));
                }
                
                } else {
                break;
            }
        }
        path_tree
    }
}


