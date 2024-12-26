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
                Node::Volume { .. } => {},
                Node::VolumeChild { .. } => {},
                Node::User { data, attributes } => {
                    for attribute in attributes {
                        g.add_edge(NodeIndex::from(i as u32), NodeIndex::from(*attribute as u32), ()); 
                    }
                },
                _ => {}
            }
        }

        self.g = Graph::from(&g);

        for (i, node) in wfc.nodes.iter().enumerate() {
            
            let label = match node {
                Node::None => "None".to_owned(),
                Node::Number { val } => format!("Number: {val}"),
                Node::NumberSet { vals, r#type, .. } => format!("NumberSet: {:?} {vals:?}", r#type),
                Node::Pos { pos } => format!("Pos: [{:0.2}, {:0.2}]", pos.x, pos.y),
                Node::Volume { csg, children } => "Volume".to_owned(),
                Node::VolumeChild { parent, .. } => format!("VolumeChild {parent}"),
                Node::User { .. } => format!("User"),
            }; 

            self.g.node_mut(NodeIndex::new(i)).unwrap().set_label(label);
        }

        self.reset();
    }
}
