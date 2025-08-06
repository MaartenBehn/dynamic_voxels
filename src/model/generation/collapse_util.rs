use octa_force::{anyhow::{anyhow, bail}, glam::Vec3, OctaResult};
use slotmap::{Key, SlotMap};
use octa_force::log::info;

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{collapse::{CollapseChildKey, CollapseNode, CollapseNodeKey, Collapser, NodeDataType}, pos_set::PositionSet, template::{TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


impl<T: ModelGenerationTypes> Collapser<T> { 
    pub fn has_index(&self, node_index: CollapseNodeKey) -> bool {
        self.nodes.contains_key(node_index)
    }

    pub fn has_index_unpacked(nodes: &SlotMap<CollapseNodeKey, CollapseNode<T>>, node_index: CollapseNodeKey) -> bool {
        nodes.contains_key(node_index)
    }

    pub fn get_node_ref_from_node_index(&self, node_index: CollapseNodeKey) -> OctaResult<&CollapseNode<T>> {
        self.nodes.get(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_ref_from_node_index_unpacked(nodes: &SlotMap<CollapseNodeKey, CollapseNode<T>>, node_index: CollapseNodeKey) -> OctaResult<&CollapseNode<T>> {
        nodes.get(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_mut_from_node_index(&mut self, node_index: CollapseNodeKey) -> OctaResult<&mut CollapseNode<T>> {
        self.nodes.get_mut(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_node_mut_from_node_index_unpacked(nodes: &mut SlotMap<CollapseNodeKey, CollapseNode<T>>, node_index: CollapseNodeKey) -> OctaResult<&mut CollapseNode<T>> {
        nodes.get_mut(node_index).ok_or(anyhow!("Node index invalid!"))
    }

    pub fn get_template_from_node_ref<'a>(&self, node: &CollapseNode<T>, template: &'a TemplateTree<T>) -> &'a TemplateNode<T> {
        &template.nodes[node.template_index]
    }

    pub fn get_template_from_node_index<'a>(&self, node_index: CollapseNodeKey, template: &'a TemplateTree<T>) -> &'a TemplateNode<T> {
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
 
    fn get_dependend_index(&self, index: CollapseNodeKey, identifier: T::Identifier) -> CollapseNodeKey {
        let depends = &self.nodes.get(index).expect("Node by index not found").depends;
        depends.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no depends {:?}", identifier)).1
    }

    pub fn get_dependend_number(&self, index: CollapseNodeKey, identifier: T::Identifier) -> i32 {
        let index = self.get_dependend_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_dependend_pos(&self, index: CollapseNodeKey, identifier: T::Identifier, pos_set_child_idetifier: T::Identifier) -> Vec3 {
        let i = self.get_dependend_index(index, identifier);
        let ci = self.get_dependend_index(index, pos_set_child_idetifier);
        let child_key = self.nodes[ci].child_key;
        self.get_pos(i, child_key)
    }

    pub fn get_parent_pos(&self, index: CollapseNodeKey) -> Vec3 {
        let node = &self.nodes[index];
        self.get_pos(node.defined_by, node.child_key)
    }

    fn get_known_index(&self, index: CollapseNodeKey, identifier: T::Identifier) -> CollapseNodeKey {
        let knows = &self.nodes.get(index).expect("Node by index not found").knows;
        knows.iter().find(|(i, _)| *i == identifier).expect(&format!("Node has no knows {:?}", identifier)).1
    }

    pub fn get_known_number(&self, index: CollapseNodeKey, identifier: T::Identifier) -> i32 {
        let index = self.get_known_index(index, identifier);
        self.get_number(index)
    }

    pub fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> OctaResult<CollapseNodeKey> {
        self.nodes.iter()
            .find(|(key, n)| n.identifier == identifier)
            .map(|(key, _)| Ok(key))
            .unwrap_or(Err(anyhow!("No node for identifier found")))
    }

    pub fn get_position_set_by_identifier_mut(&mut self, identifier: T::Identifier) -> OctaResult<&mut PositionSet<T>> {
        let index = self.get_node_index_by_identifier(identifier)?;
        let node = &mut self.nodes[index];
        let NodeDataType::PosSet(pos_set) = &mut node.data else { bail!("Node is not pos set") };
        Ok(pos_set)
    }

    pub fn set_position_set_value(&mut self, index: CollapseNodeKey, pos_set: PositionSet<T>) {
        let node = &mut self.nodes[index];
        node.data = NodeDataType::PosSet(pos_set);  
    }
}
