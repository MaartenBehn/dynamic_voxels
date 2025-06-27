use octa_force::{anyhow::anyhow, glam::Vec3, OctaResult};
use slotmap::SlotMap;
use octa_force::log::info;

use crate::volume::VolumeQureyPosValid;

use super::{builder::{BU, IT}, collapse::{CollapseNodeKey, Collapser, CollapseNode, NodeDataType}, template::{TemplateNode, TemplateTree}};


impl<'a, I: IT, U: BU, V: VolumeQureyPosValid> Collapser<'a, I, U, V> { 
    pub fn has_index(&self, node_index: CollapseNodeKey) -> bool {
        self.nodes.contains_key(node_index)
    }

    pub fn has_index_unpacked(nodes: &SlotMap<CollapseNodeKey, CollapseNode<I, U, V>>, node_index: CollapseNodeKey) -> bool {
        nodes.contains_key(node_index)
    }

    pub fn get_node_ref_from_node_index(&self, node_index: CollapseNodeKey) -> OctaResult<&CollapseNode<I, U, V>> {
        self.nodes.get(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_ref_from_node_index_unpacked(nodes: &SlotMap<CollapseNodeKey, CollapseNode<I, U, V>>, node_index: CollapseNodeKey) -> OctaResult<&CollapseNode<I, U, V>> {
        nodes.get(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_mut_from_node_index(&mut self, node_index: CollapseNodeKey) -> OctaResult<&mut CollapseNode<I, U, V>> {
        self.nodes.get_mut(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_mut_from_node_index_unpacked(nodes: &mut SlotMap<CollapseNodeKey, CollapseNode<I, U, V>>, node_index: CollapseNodeKey) -> OctaResult<&mut CollapseNode<I, U, V>> {
        nodes.get_mut(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_template_from_node_ref(&self, node: &CollapseNode<I, U, V>) -> &'a TemplateNode<I, V> {
        &self.template.nodes[node.template_index]
    }

    pub fn get_template_from_node_ref_unpacked(template: &'a TemplateTree<I, V>, node: &CollapseNode<I, U, V>) -> &'a TemplateNode<I, V> {
        &template.nodes[node.template_index]
    }

    pub fn get_template_from_node_index(&self, node_index: CollapseNodeKey) -> &'a TemplateNode<I, V> {
        &(self.template.nodes[self.nodes[node_index].template_index])
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> i32 {
        match &self.nodes.get(index).expect("Number by index not found").data {
            NodeDataType::Number(d) => d.value,
            _ => panic!("Number by index is not of Type Number")
        }
    }

    pub fn get_pos(&self, index: CollapseNodeKey) -> Vec3 {
        match &self.nodes.get(index).expect("Pos by index not found").data {
            NodeDataType::Pos(d) => d.value,
            _ => panic!("Pos by index is not of Type Pos")
        }
    }


    pub fn get_pos_mut(&mut self, index: CollapseNodeKey) -> &mut Vec3 {
        match &mut self.nodes.get_mut(index).expect("Pos by index not found").data {
            NodeDataType::Pos(d) => &mut d.value,
            _ => panic!("Pos by index is not of Type Pos")
        }
    }

    fn get_dependend_index(&self, index: CollapseNodeKey, identifier: I) -> CollapseNodeKey {
        let depends = &self.nodes.get(index).expect("Node by index not found").depends;
        depends.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no depends {:?}", identifier)).1
    }


    pub fn get_dependend_number(&self, index: CollapseNodeKey, identifier: I) -> i32 {
        let index = self.get_dependend_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_dependend_pos(&self, index: CollapseNodeKey, identifier: I) -> Vec3 {
        let index = self.get_dependend_index(index, identifier);
        self.get_pos(index)
    }

    pub fn get_dependend_pos_mut(&mut self, index: CollapseNodeKey, identifier: I) -> &mut Vec3 {
        let index = self.get_dependend_index(index, identifier);
        self.get_pos_mut(index)
    }


    fn get_known_index(&self, index: CollapseNodeKey, identifier: I) -> CollapseNodeKey {
        let knows = &self.nodes.get(index).expect("Node by index not found").knows;
        knows.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no knows {:?}", identifier)).1
    }

    pub fn get_known_number(&self, index: CollapseNodeKey, identifier: I) -> i32 {
        let index = self.get_known_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_known_pos(&self, index: CollapseNodeKey, identifier: I) -> Vec3 {
        let index = self.get_known_index(index, identifier);
        self.get_pos(index)
    }

    pub fn get_known_pos_mut(&mut self, index: CollapseNodeKey, identifier: I) -> &mut Vec3 {
        let index = self.get_known_index(index, identifier);
        self.get_pos_mut(index)
    }
}
