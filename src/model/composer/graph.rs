use std::{fs::{self, File}, io::Write};

use bitvec::vec::BitVec;
use egui_snarl::{InPinId, Node, NodeId, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::OctaResult;
use smallvec::SmallVec;

use crate::{model::data_types::data_type::ComposeDataType, util::{number::Nu, vector::Ve}};

use super::{build::{ComposeTypeTrait, BS}, nodes::{ComposeNode, ComposeNodeType}};

const TEMP_SAVE_FILE: &str = "./composer_temp_save.json";

#[derive(Debug)]
pub struct ComposerGraph<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub snarl: Snarl<ComposeNode<B::ComposeType>>,
    pub flags: ComposerNodeFlags,
}

#[derive(Debug)]
pub struct ComposerNodeFlags { 
    pub deleted_nodes: SmallVec<[NodeId; 4]>,
    pub added_nodes: BitVec,
    pub changed_nodes: BitVec,
    pub needs_collapse_nodes: BitVec,
    pub invalid_nodes: BitVec,
    pub cam_nodes: SmallVec<[NodeId; 4]>,
}

impl<V2, V3, T, B> ComposerGraph<V2, V3, T, B> 
where 
    V2: Ve<T, 2>, 
    V3: Ve<T, 3>, 
    T: Nu, 
    B: BS<V2, V3, T>,
    B::ComposeType: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new() -> Self {
        let mut snarl = load_snarl().unwrap_or(Snarl::new());       
        let mut flags = ComposerNodeFlags::new(&mut snarl);

        Self { 
            snarl, 
            flags,
        }
    }

    pub fn save(&self) -> OctaResult<()> {
        let snarl = serde_json::to_string(&self.snarl).unwrap();
        let mut file = File::create(TEMP_SAVE_FILE)?;
        file.write_all(snarl.as_bytes())?;

        Ok(())
    }
}

impl<V2, V3, T, B> ComposerGraph<V2, V3, T, B> 
where 
    V2: Ve<T, 2>, 
    V3: Ve<T, 3>, 
    T: Nu, 
    B: BS<V2, V3, T>,
{
    pub fn get_input_remote_node_id(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> NodeId {
        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No node connected to {:?}", node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", node.t);
        }

        remotes[0].node
    }

    pub fn get_creates_input_remote_pin(&self, node: &ComposeNode<B::ComposeType>) -> Option<NodeId> {
        let i = node.inputs.iter()
            .position(|i| i.data_type == ComposeDataType::Creates)
            .expect(&format!("{:?} does not have a creates input pin!", node.t));

        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: i }).remotes;
        remotes.first()
            .map(|pin| pin.node)
    }

    pub fn get_output_first_remote_pin_by_index(&self, node: &ComposeNode<B::ComposeType>, index: usize) -> InPinId {
        let remotes = self.snarl.out_pin(OutPinId{ node: node.id, output: index }).remotes;
        if remotes.is_empty() {
            panic!("No output node connected to {:?}", node.t);
        }

        remotes[0]
    }
}

pub fn load_snarl<CT: ComposeTypeTrait + serde::de::DeserializeOwned>() -> OctaResult<Snarl<ComposeNode<CT>>> {
    let content = fs::read_to_string(TEMP_SAVE_FILE)?; 
    let snarl = serde_json::from_str(&content)?;
    Ok(snarl)
}


impl ComposerNodeFlags {
    pub fn new<CT: ComposeTypeTrait>(snarl: &mut Snarl<ComposeNode<CT>>) -> Self {
        let mut flags = Self { 
            deleted_nodes: SmallVec::new(),
            added_nodes: BitVec::new(),
            changed_nodes: BitVec::new(),
            needs_collapse_nodes: BitVec::new(),
            invalid_nodes: BitVec::new(),
            cam_nodes: SmallVec::new(),
        };


        for node in snarl.nodes().cloned().collect_vec() {
            let node_id = node.id; 
            flags.enshure_nodes_list_index(node_id.0);

            match node.t {
                ComposeNodeType::CamPosition => {
                    flags.cam_nodes.push(node_id);
                }
                _ => {}
            }

            let valid = flags.validate_node(node, snarl);
            flags.invalid_nodes.set(node_id.0, !valid);
        }

        flags
    }

    pub fn reset_change_flags(&mut self) {
        self.deleted_nodes.clear();
        self.added_nodes.clear();
        self.changed_nodes.clear();
        self.needs_collapse_nodes.clear();
    } 

    pub fn enshure_nodes_list_index(&mut self, i: usize) {
        if self.added_nodes.len() <= i {
            self.added_nodes.resize(i + 1, false);
            self.changed_nodes.resize(i + 1, false);
            self.invalid_nodes.resize(i + 1, false);
            self.needs_collapse_nodes.resize(i + 1, false);
        }
    }

    pub fn set_added<CT: ComposeTypeTrait>(&mut self, node_id: NodeId, snarl: &Snarl<ComposeNode<CT>>) {
        self.enshure_nodes_list_index(node_id.0);
        
        self.added_nodes.set(node_id.0, true);
        self.set_needs_collapse(node_id, snarl);
    }

    pub fn set_changed<CT: ComposeTypeTrait>(&mut self, node_id: NodeId, snarl: &Snarl<ComposeNode<CT>>) {
        self.enshure_nodes_list_index(node_id.0);

        if self.added_nodes.get(node_id.0).as_deref().copied().unwrap_or(false) {
            return;
        }

        self.changed_nodes.set(node_id.0, true);
        self.set_needs_collapse(node_id, snarl);
    }

    pub fn set_deleted(&mut self, node_id: NodeId) {
        self.added_nodes.set(node_id.0, false);
        self.changed_nodes.set(node_id.0, false);

        if !self.deleted_nodes.contains(&node_id) {
            self.deleted_nodes.push(node_id);
        }
    }

    pub fn set_needs_collapse<CT: ComposeTypeTrait>(&mut self, node_id: NodeId, snarl: &Snarl<ComposeNode<CT>>) {
        self.enshure_nodes_list_index(node_id.0);

        if *self.needs_collapse_nodes.get(node_id.0).as_deref().unwrap() {
            return;
        }

        self.needs_collapse_nodes.set(node_id.0, true);
        
        let node = snarl.get_node(node_id)
            .expect("NodeId was not valid")
            .to_owned();

        for (i, output) in node.outputs.iter().enumerate() {
            let out_pin = snarl.out_pin(OutPinId { node: node.id, output: i });

            for remote in out_pin.remotes {
                self.set_needs_collapse(remote.node, snarl);
            }       
        }
    }

    pub fn set_cam_notes_as_changed<CT: ComposeTypeTrait>(&mut self, snarl: &Snarl<ComposeNode<CT>>) {
        for node_id in self.cam_nodes.to_owned() {
            self.set_changed(node_id, snarl);
        }
    }
}


