use egui_graphs::Graph;
use petgraph::graph::NodeIndex;

use crate::wfc::node::{Node, WFC};

use super::renderer::WFCRenderer;

impl WFCRenderer {
    pub fn set_wfc<U: Clone>(&mut self, wfc: &WFC<U>) {
         
        let mut g = petgraph::stable_graph::StableGraph::new();

        for node in wfc.nodes.iter() {
            g.add_node(());
        }

        for (i, node) in wfc.nodes.iter().enumerate() {
            match node {
                Node::NumberSet { vals, r#type, children } => {
                    
                    for child in children {
                        g.add_edge(NodeIndex::from(i as u32), NodeIndex::from(*child as u32), ());
                    }

                },
                Node::Volume { csg, children } => {},
                Node::VolumeChild { parent, children } => {},
                Node::User { data, attributes, on_collapse } => {
                    for attribute in attributes {
                        g.add_edge(NodeIndex::from(i as u32), NodeIndex::from(*attribute as u32), ()); 
                    }
                },
                _ => {}
            }
        }

        self.g = Graph::from(&g);
        self.reset();
    }
}
