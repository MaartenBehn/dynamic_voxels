use super::{builder::{Identifier, WFCBuilder}, collapse::{ Node, NumberAttribute, PosAttribute}, volume::PossibleVolume};
use std::fmt::Debug;

pub trait FuncData<U: Clone + Debug, B: Clone + Debug> {

    fn get_nodes(&self) -> &[Node<U>];
    fn get_nodes_mut(&mut self) -> &mut[Node<U>];
    fn get_number_attributes(&self) -> &[NumberAttribute];
    fn get_pos_attributes(&self) -> &[PosAttribute];
    fn get_builder(&self) -> &WFCBuilder<U, B>; 
    
    fn get_node_index_from_identifier(&self, identifer: Identifier) -> Option<usize> {
        for (i, node) in self.get_nodes().iter().enumerate().rev() {
            let template_node = &self.get_builder().nodes[node.template_index];
            if template_node.identifier == Some(identifer) {
                return Some(i);
            }
        }

        None
    }

    fn get_node_with_identifier(&self, identifer: Identifier) -> Option<&Node<U>> {
        for node in self.get_nodes().iter().rev() {
            let template_node = &self.get_builder().nodes[node.template_index];
            if template_node.identifier == Some(identifer) {
                return Some(node);
            }
        }

        None
    }

    fn get_number_attribute_with_identifier(&self, identifer: Identifier, mut skip: usize) -> Option<&NumberAttribute> {
        for attribute in self.get_number_attributes().iter().rev() {
            let template_attribute = &self.get_builder().attributes[attribute.template_index];
            if template_attribute.identifier == identifer {
                if skip > 0 {
                    skip -= 1;
                } else {
                    return Some(attribute);
                }
            }
        }

        None
    }

    fn get_pos_attribute_with_identifier(&self, identifer: Identifier, mut skip: usize) -> Option<&PosAttribute> {
        for attribute in self.get_pos_attributes().iter().rev() {
            let template_attribute = &self.get_builder().attributes[attribute.template_index];
            if template_attribute.identifier == identifer {
                if skip > 0 {
                    skip -= 1;
                } else {
                    return Some(attribute);
                }
            }
        }

        None
    }

    fn get_node_user_data_mut(&mut self, index: usize) -> Option<&mut U> {
        self.get_nodes_mut()[index].user_data.as_mut()
    }

    fn get_node_user_data_mut_with_identifier(&mut self, identifer: Identifier) -> Option<&mut U> {
        let index = self.get_node_index_from_identifier(identifer);
        if index.is_none() {
            return None;
        }

        self.get_node_user_data_mut(index.unwrap())
    }

    fn get_child_number_attribute_with_identifier(&self, index: usize, identifer: Identifier) -> Option<&NumberAttribute> {
        for &attribute_index in self.get_nodes()[index].number_attributes.iter() {
            let attribute = &self.get_number_attributes()[attribute_index];
            let attribute_template = &self.get_builder().attributes[attribute.template_index];

            if attribute_template.identifier == identifer {
                return Some(attribute);
            }
        }

        None
    }

    fn get_child_pos_attribute_with_identifier(&self, index: usize, identifer: Identifier) -> Option<&PosAttribute> {
        for &attribute_index in self.get_nodes()[index].pos_attributes.iter() {
            let attribute = &self.get_pos_attributes()[attribute_index];
            let attribute_template = &self.get_builder().attributes[attribute.template_index];

            if attribute_template.identifier == identifer {
                return Some(attribute);
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct PosCollapseFuncData<'a, U: Clone + Debug, B: Clone + Debug> {
    nodes: &'a mut [Node<U>],
    number_attributes: &'a[NumberAttribute],
    pos_attributes: &'a[PosAttribute],
    possible_volumes: &'a[PossibleVolume],
    builder: &'a WFCBuilder<U, B>,
    current_node_index: usize,
    pub from_volume: &'a PossibleVolume,
}


impl<'a, U: Clone + Debug, B: Clone + Debug> FuncData<U, B> for PosCollapseFuncData<'a, U, B> {
    fn get_nodes(&self) -> &[Node<U>] { self.nodes }
    fn get_number_attributes(&self) -> &[NumberAttribute] { self.number_attributes }
    fn get_pos_attributes(&self) -> &[PosAttribute] { self.pos_attributes }
    fn get_builder(&self) -> &WFCBuilder<U, B> { self.builder }
    fn get_nodes_mut(&mut self) -> &mut[Node<U>] { self.nodes }
}

impl<'a, U: Clone + Debug, B: Clone + Debug> PosCollapseFuncData<'a, U, B> {
    pub fn new(
        nodes: &'a mut [Node<U>], 
        number_attributes: &'a[NumberAttribute], 
        pos_attributes: &'a[PosAttribute], 
        possible_volumes: &'a[PossibleVolume], 
        builder: &'a WFCBuilder<U, B>, 
        current_node_index: usize, 
        from_volume: &'a PossibleVolume,
    ) -> Self {
        Self {
            nodes,
            number_attributes,
            pos_attributes,
            possible_volumes,
            builder,
            current_node_index,
            from_volume
        }
    }

    pub fn get_current_node(&self) -> &Node<U> {
        &self.nodes[self.current_node_index]
    }

    pub fn get_current_node_number_attribute_with_identifier(&self, identifer: Identifier) -> Option<&NumberAttribute> {
        self.get_child_number_attribute_with_identifier(self.current_node_index, identifer)
    }

    pub fn get_current_node_pos_attribute_with_identifier(&self, identifer: Identifier) -> Option<&PosAttribute> {
        self.get_child_pos_attribute_with_identifier(self.current_node_index, identifer)
    }

    pub fn get_current_user_data_mut(&mut self) -> Option<&mut U> {
        self.get_node_user_data_mut(self.current_node_index)
    }
}

#[derive(Debug)]
pub struct BuildFuncData<'a, U: Clone + Debug, B: Clone + Debug> {
    nodes: &'a mut [Node<U>],
    number_attributes: &'a[NumberAttribute],
    pos_attributes: &'a[PosAttribute],
    builder: &'a WFCBuilder<U, B>,
    current_node_index: usize,
    build_data: &'a mut B,
}


impl<'a, U: Clone + Debug, B: Clone + Debug> FuncData<U, B> for BuildFuncData<'a, U, B> {
    fn get_nodes(&self) -> &[Node<U>] { self.nodes }
    fn get_number_attributes(&self) -> &[NumberAttribute] { self.number_attributes }
    fn get_pos_attributes(&self) -> &[PosAttribute] { self.pos_attributes }
    fn get_builder(&self) -> &WFCBuilder<U, B> { self.builder }
    fn get_nodes_mut(&mut self) -> &mut[Node<U>] { self.nodes }
}

impl<'a, U: Clone + Debug, B: Clone + Debug> BuildFuncData<'a, U, B> {
    pub fn new(
        nodes: &'a mut [Node<U>], 
        number_attributes: &'a[NumberAttribute], 
        pos_attributes: &'a[PosAttribute], 
        builder: &'a WFCBuilder<U, B>, 
        current_node_index: usize, 
        build_data: &'a mut B
    ) -> Self {
        Self {
            nodes,
            number_attributes,
            pos_attributes,
            builder,
            current_node_index,
            build_data,
        }
    }

    pub fn get_current_node(&self) -> &Node<U> {
        &self.nodes[self.current_node_index]
    }

    pub fn get_current_node_number_attribute_with_identifier(&self, identifer: Identifier) -> Option<&NumberAttribute> {
        self.get_child_number_attribute_with_identifier(self.current_node_index, identifer)
    }

    pub fn get_current_node_pos_attribute_with_identifier(&self, identifer: Identifier) -> Option<&PosAttribute> {
        self.get_child_pos_attribute_with_identifier(self.current_node_index, identifer)
    }

    pub fn get_current_user_data_mut(&mut self) -> Option<&mut U> {
        self.get_node_user_data_mut(self.current_node_index)
    }

    pub fn get_build_data(&self) -> &B {
        &self.build_data
    }

    pub fn get_build_data_mut(&mut self) -> &mut B {
        &mut self.build_data
    }
}
