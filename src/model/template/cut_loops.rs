use octa_force::log::trace;

use crate::model::template::{Template, dependency_tree::DependencyPath, nodes::TemplateNode};

impl Template {
     pub fn cut_loops(&mut self, index: usize, mut index_seen: Vec<usize>) -> usize {
        index_seen.push(index);

        trace!("Set level of node {}, index_seen: {:?}", index, &index_seen);

        let node: &mut TemplateNode = &mut self.nodes[index];
        
        let mut max_level = 0;
        for (i, depends_index) in node.depends.to_owned().iter().enumerate().rev() {
            trace!("Node {}, depends on {}", index, *depends_index);

            if let Some(_) = index_seen.iter().find(|p| **p == *depends_index) {
                trace!("Loop found from {} to {:?}", index, depends_index);
                
                let value_index = self.nodes[index].value_index;
                for hook in self.iter_hooks(value_index) {
                    hook.loop_cut |= hook.template_index == *depends_index;
                }

                let node = &mut self.nodes[index];
                node.depends.swap_remove(i);
                node.depends_loop.push((*depends_index, DependencyPath::default()));

                continue;
            }

            let mut level = self.nodes[*depends_index].level; 
            if level == 0 {
                level = self.cut_loops(*depends_index, index_seen.to_owned());
            } 

            max_level = max_level.max(level);
        }

        let node_level = max_level + 1;
        self.nodes[index].level = node_level;
        self.max_level = self.max_level.max(node_level);

        node_level
    }
}
