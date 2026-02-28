use std::{fs::{self, File}, io::Write};

use bitvec::vec::BitVec;
use egui_snarl::{InPinId, Node, NodeId, OutPinId, Snarl};
use itertools::Itertools;
use octa_force::OctaResult;
use smallvec::SmallVec;

use crate::{model::{composer::flags::ComposerNodeFlags, data_types::data_type::ComposeDataType}, util::{number::Nu, vector::Ve}};

use super::{nodes::{ComposeNode}};

const TEMP_SAVE_FILE: &str = "./composer_temp_save.json";

#[derive(Debug)]
pub struct ComposerGraph {
    pub snarl: Snarl<ComposeNode>,
    pub flags: ComposerNodeFlags,
}

impl ComposerGraph {

    pub fn new() -> Self {
        let mut snarl = load_snarl().unwrap_or(Snarl::new());       
        let mut flags = ComposerNodeFlags::new(&mut snarl);
        flags.check_valid_for_all_nodes(&mut snarl);

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

impl ComposerGraph 
{
    pub fn get_input_remote_node_id(&self, node: &ComposeNode, index: usize) -> NodeId {
        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: index }).remotes;
        if remotes.is_empty() {
            panic!("No node connected to {:?}", node.t);
        }

        if remotes.len() >= 2 {
            panic!("More than one node connected to {:?}", node.t);
        }

        remotes[0].node
    }

    pub fn get_creates_input_remote_pin(&self, node: &ComposeNode) -> Option<NodeId> {
        let i = node.inputs.iter()
            .position(|i| i.data_type == ComposeDataType::Creates)
            .expect(&format!("{:?} does not have a creates input pin!", node.t));

        let remotes = self.snarl.in_pin(InPinId{ node: node.id, input: i }).remotes;
        remotes.first()
            .map(|pin| pin.node)
    }

    pub fn get_output_first_remote_pin_by_index(&self, node: &ComposeNode, index: usize) -> InPinId {
        let remotes = self.snarl.out_pin(OutPinId{ node: node.id, output: index }).remotes;
        if remotes.is_empty() {
            panic!("No output node connected to {:?}", node.t);
        }

        remotes[0]
    }
}

pub fn load_snarl() -> OctaResult<Snarl<ComposeNode>> {
    let content = fs::read_to_string(TEMP_SAVE_FILE)?; 
    let snarl = serde_json::from_str(&content)?;
    Ok(snarl)
}



