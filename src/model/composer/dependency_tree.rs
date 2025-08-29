
use std::{collections::VecDeque, usize};

use smallvec::SmallVec;

use super::template::{ComposeTemplate, TemplateIndex, TemplateNode};


#[derive(Debug, Clone, Default)]
pub struct DependencyTree {
    pub steps: Vec<DependencyPathStep>,
}

#[derive(Debug, Clone)]
pub struct DependencyPathStep {
    pub into_index: TemplateIndex,
    pub children: Vec<usize>,
    pub up: bool, 
    pub leaf: Option<usize>,
}

impl DependencyTree {
    pub fn new(
        template: &ComposeTemplate, 
        parent_index: TemplateIndex, 
        depends: &[TemplateIndex], 
    ) -> DependencyTree {
        let mut depends = depends.iter().copied().enumerate().collect::<Vec<_>>();

        let mut open_child_paths: VecDeque<(&TemplateNode, Vec<DependencyPathStep>)> = VecDeque::new();
        let mut open_parent_paths: VecDeque<(&TemplateNode, Vec<DependencyPathStep>)> = VecDeque::new();

        open_parent_paths.push_back((
            &template.nodes[parent_index], 
            vec![DependencyPathStep { 
                into_index: parent_index,
                children: vec![],
                up: true,
                leaf: None,
            }]
        ));

        let mut path_tree = DependencyTree { 
            steps: vec![],
        };

        let mut check_hit = |node: &TemplateNode, path: &Vec<DependencyPathStep>| {
            let depends_res = depends.iter()
                .enumerate()
                .find(|(_, (_, i))| *i == node.index)
                .map(|(i, (j, _))|(i, *j));

            if let Some((i, depends_index)) = depends_res {
                let leaf_index = path_tree.copy_path(node, path);
                depends.swap_remove(i);
                path_tree.steps[leaf_index].leaf = Some(depends_index);        
            }
        };
             
        loop {
            if let Some((node, path)) = open_child_paths.pop_front() {
                check_hit(node, &path);
                
                for index in node.dependend.iter() {
                    let child = &template.nodes[*index];

                    let mut child_path = path.clone();
                    child_path.push(DependencyPathStep { 
                        into_index: *index,
                        children: vec![],
                        up: false,
                        leaf: None,
                    });

                    open_child_paths.push_back((child, child_path));
                }
                
            } else if let Some((node, path)) = open_parent_paths.pop_front() {
                check_hit(node, &path);

                for index in node.dependend.iter() {
                    let child = &template.nodes[*index];

                    let mut child_path = path.clone();
                    child_path.push(DependencyPathStep { 
                        into_index: *index,
                        children: vec![],
                        up: false,
                        leaf: None,
                    });

                    open_child_paths.push_back((child, child_path));
                }

                for parent_index in node.depends.iter() {
                    let parent = &template.nodes[*parent_index];

                    let mut parent_path = path.clone();
                    parent_path.push(DependencyPathStep { 
                        into_index: *parent_index,
                        children: vec![],
                        up: true,
                        leaf: None,
                    });
                    open_parent_paths.push_back((parent, parent_path));
                }
                
            } else {
                break;
            }
        }

        path_tree
    }

    fn copy_path(
        &mut self, 
        node: &TemplateNode, 
        path: &Vec<DependencyPathStep>, 
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


