use octa_force::{anyhow::{anyhow, bail, ensure, Context}, glam::{Vec3, Vec3A}, OctaResult};
use slotmap::{Key, SlotMap};
use octa_force::log::info;

use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};

use super::{collapse::{CollapseChildKey, CollapseNode, CollapseNodeKey, Collapser, NodeDataType}, number_range::NumberRange, pos_set::PositionSet, template::{TemplateNode, TemplateTree}, traits::ModelGenerationTypes};


impl<T: ModelGenerationTypes> Collapser<T> { 
    pub(super) fn get_template_from_node_ref<'a>(&self, node: &CollapseNode<T>, template: &'a TemplateTree<T>) -> &'a TemplateNode<T> {
        &template.nodes[node.template_index]
    }
 
    // -- Number Range ---

    pub(super) fn get_number_values_mut(&mut self, index: CollapseNodeKey) -> &mut Vec<i32> {
        match &mut self.nodes.get_mut(index).expect("Number by index not found").data {
            NodeDataType::NumberRange(d) => &mut d.values,
            _ => panic!("Number by index is not of Type Number")
        }
    }

    pub fn get_number(&self, index: CollapseNodeKey) -> OctaResult<i32> {
        let node = self.nodes.get(index).expect("Number Set by index not found");
        
        match &node.data {
            NodeDataType::NumberRange(d) => Ok(d.value),
            _ => bail!("{:?} is not of Type Number Set", node.identifier)
        }
    }

    // -- Pos Set --
    
    pub fn set_position_set_value(&mut self, index: CollapseNodeKey, pos_set: PositionSet<T>) -> OctaResult<()> {
        let node = &mut self.nodes[index];
        ensure!(matches!(node.data, NodeDataType::PosSet(..)), "{:?} is not a Pos Set", node.identifier);

        node.data = NodeDataType::PosSet(pos_set); 
        Ok(())
    }

    pub(super) fn get_volume_mut(&mut self, index: CollapseNodeKey) -> OctaResult<&mut T::Volume> {
        let node = self.nodes.get_mut(index).expect("Pos Set by index not found");

        match &mut node.data {
            NodeDataType::PosSet(d) => d.get_volume_mut(),
            _ => bail!("{:?} is not of type Pos Set", node.identifier)
        }
    }

    pub(super) fn get_volume2d_mut(&mut self, index: CollapseNodeKey) -> OctaResult<&mut T::Volume2D> {
        let node = self.nodes.get_mut(index).expect("Pos Set by index not found");
        
        match &mut node.data {
            NodeDataType::PosSet(d) => d.get_volume2d_mut(),
            _ => bail!("{:?} is not a Pos Set", node.identifier)
        }
    }
     
    pub fn get_pos(&self, index: CollapseNodeKey, pos_key: CollapseChildKey) -> OctaResult<Vec3A> {
        let node = self.nodes.get(index).expect("Pos Set by index not found");
        
        match &node.data {
            NodeDataType::PosSet(d) => Ok(d.get_pos(pos_key)),
            _ => bail!("{:?} is not a Pos Set", node.identifier)
        }
    }


    // -- Undo Data -- 
    
    pub fn get_undo_data(&self, index: CollapseNodeKey) -> &T::UndoData {
        &self.nodes[index].undo_data
    }

    pub fn get_undo_data_mut(&mut self, index: CollapseNodeKey) -> &mut T::UndoData {
        &mut self.nodes[index].undo_data
    }

    pub fn set_undo_data(&mut self, index: CollapseNodeKey, data: T::UndoData) {
        let node = self.nodes.get_mut(index)
            .expect("Index of node is not valid!");

        node.undo_data = data;
    }

 
    // -- Depends --

    pub(super) fn get_dependend_index(&self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<CollapseNodeKey> {
        let node = self.nodes.get(index).expect("Node by index not found"); 

        Ok(node.depends.iter().find(|(i, _)| *i == identifier)
            .context(format!("{:?} does not depend on {:?}", node.identifier, identifier))?
            .1
        )
    }

    pub fn get_dependend_number(&self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<i32> {
        let index = self.get_dependend_index(index, identifier)
            .context("Trying to get dependend number")?;
        self.get_number(index)
    }

    pub fn get_dependend_pos(&self, index: CollapseNodeKey, identifier: T::Identifier, pos_set_child_idetifier: T::Identifier) -> OctaResult<Vec3A> {
        let i = self.get_dependend_index(index, identifier)?;
        let ci = self.get_dependend_index(index, pos_set_child_idetifier)?;
        let child_key = self.nodes[ci].child_key;

        ensure!(child_key != CollapseChildKey::null(), "{:?} is not a child of a pos set", pos_set_child_idetifier);
        self.get_pos(i, child_key)
    }

    pub fn get_dependend_undo_data(&self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<&T::UndoData> {
        let index = self.get_dependend_index(index, identifier)?;
        Ok(self.get_undo_data(index))
    }

    pub fn get_dependend_undo_data_mut(&mut self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<&mut T::UndoData> {
        let index = self.get_dependend_index(index, identifier)?;
        Ok(self.get_undo_data_mut(index))
    }

    pub fn get_parent_pos(&self, index: CollapseNodeKey) -> OctaResult<Vec3A> {
        let node = &self.nodes[index];
        self.get_pos(node.defined_by, node.child_key)
    }

    // -- Knows --
    
    pub(super) fn get_known_index(&self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<CollapseNodeKey> {
        let node = self.nodes.get(index).expect("Node by index not found"); 

        Ok(node.knows.iter().find(|(i, _)| *i == identifier)
            .context(format!("{:?} does not know {:?}", node.identifier, identifier))?
            .1
        )
    }

    pub fn get_known_number(&self, index: CollapseNodeKey, identifier: T::Identifier) -> OctaResult<i32> {
        let index = self.get_known_index(index, identifier)?;
        self.get_number(index)
    }


    // -- by identifier --

    pub fn get_node_index_by_identifier(&self, identifier: T::Identifier) -> OctaResult<CollapseNodeKey> {
        self.nodes.iter()
            .find(|(key, n)| n.identifier == identifier)
            .map(|(key, _)| key)
            .context(format!("No node for {:?} found", identifier))
    }
 
    pub fn get_position_set_by_identifier_mut(&mut self, identifier: T::Identifier) -> OctaResult<&mut PositionSet<T>> {
        let index = self.get_node_index_by_identifier(identifier)?;
        let node = &mut self.nodes[index];
        let NodeDataType::PosSet(pos_set) = &mut node.data else { bail!("{:?} is not pos set", identifier) };
        Ok(pos_set)
    }
}
