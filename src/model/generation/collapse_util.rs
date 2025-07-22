use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, OctaResult};
use slotmap::{Key, SlotMap};
use octa_force::log::info;

use crate::volume::VolumeQureyPosValid;

use super::{builder::{BU, IT}, collapse::{CollapseChildKey, CollapseNode, CollapseNodeKey, Collapser, NodeDataType}, pos_set::PositionSet, template::{TemplateNode, TemplateTree}};


impl<I: IT, U: BU, V: VolumeQureyPosValid> Collapser<I, U, V> { 
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

    pub fn get_template_from_node_ref<'a>(&self, node: &CollapseNode<I, U, V>, template: &'a TemplateTree<I, V>) -> &'a TemplateNode<I, V> {
        &template.nodes[node.template_index]
    }

    pub fn get_template_from_node_index<'a>(&self, node_index: CollapseNodeKey, template: &'a TemplateTree<I, V>) -> &'a TemplateNode<I, V> {
        &(template.nodes[self.nodes[node_index].template_index])
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> i32 {
        match &self.nodes.get(index).expect("Number by index not found").data {
            NodeDataType::NumberRange(d) => d.value,
            _ => panic!("Number by index is not of Type Number")
        }
    }

    pub fn get_pos(&self, index: CollapseNodeKey, pos_key: CollapseChildKey) -> Vec3 {
        match &self.nodes.get(index).expect("Pos Set by index not found").data {
            NodeDataType::PosSet(d) => d.get_pos(pos_key),
            _ => panic!("Number by index is not of Type Number")
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
        let i = self.get_dependend_index(index, identifier);
        let child_key = self.nodes[index].child_key;
        self.get_pos(i, child_key)
    }

    fn get_known_index(&self, index: CollapseNodeKey, identifier: I) -> CollapseNodeKey {
        let knows = &self.nodes.get(index).expect("Node by index not found").knows;
        knows.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no knows {:?}", identifier)).1
    }

    pub fn get_known_number(&self, index: CollapseNodeKey, identifier: I) -> i32 {
        let index = self.get_known_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_node_index_by_identifier(&self, identifier: I) -> OctaResult<CollapseNodeKey> {
        self.nodes.iter()
            .find(|(key, n)| n.identfier == identifier)
            .map(|(key, _)| Ok(key))
            .unwrap_or(Err(anyhow!("No node for identifier found")))
    }

    pub fn get_position_set_by_identifier_mut(&mut self, identifier: I) -> OctaResult<&mut PositionSet<V>> {
        let index = self.get_node_index_by_identifier(identifier)?;
        let node = &mut self.nodes[index];
        let NodeDataType::PosSet(pos_set) = &mut node.data else { bail!("Node is not pos set") };
        Ok(pos_set)
    }
}
