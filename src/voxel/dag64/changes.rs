use octa_force::{glam::{UVec3, Vec3A}, log::debug, OctaResult};

use crate::multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator;

use super::{node::VoxelDAG64Node, VoxelDAG64};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DAG64NodeChange {
    pub old_node: VoxelDAG64Node,
    pub new_node: VoxelDAG64Node,
    pub index: u32,
}

#[derive(Debug)]
pub struct DAG64Transaction {
    pub node_changes: Vec<DAG64NodeChange>,
    pub old_root_index: u32,
    pub new_root_index: u32,
    pub old_levels: u8,
    pub new_levels: u8,
    pub old_offset: Vec3A,
    pub new_offset: Vec3A,
}

impl DAG64Transaction {
    pub fn change_node(&mut self, index: u32, new_node: VoxelDAG64Node, old_node: VoxelDAG64Node) {
        let node_change = DAG64NodeChange {
            index,
            new_node,
            old_node,
        };

        if self.node_changes.contains(&node_change) {
            return;
        }

        self.node_changes.push(node_change);
    }

    pub fn root_index_changed(&self) -> bool {
        self.new_root_index != self.old_root_index
    }

    pub fn rebase(&mut self, other: &DAG64Transaction) {
        if !self.root_index_changed() && other.root_index_changed() {
            self.new_root_index = other.new_root_index;
            self.new_offset = other.new_offset;
            self.new_levels = other.new_levels;
        }

        for change in other.node_changes.iter() {
            if self.node_changes.iter().any(|c| c.index == change.index ) {
                continue;
            }

            self.node_changes.push(change.invert());
        }
    }

    pub fn get_node(&self, dag: &VoxelDAG64, index: u32) -> OctaResult<VoxelDAG64Node> {
        self.node_changes.iter()
            .find(|c| c.index == index )
            .map(|c| Ok(c.new_node) )
            .unwrap_or_else(|| dag.nodes.get(index as usize))
    }

    pub fn get_node_range(&self, dag: &VoxelDAG64, r: std::ops::Range<usize>) -> OctaResult<Vec<VoxelDAG64Node>> {
        Ok(dag.nodes.get_range(r.to_owned())?.iter()
            .enumerate()
            .map(|(i, n)| ((i + r.start) as u32, n) )
            .map(|(i, n)| {
                self.node_changes.iter()
                    .find(|c| c.index == i )
                    .map(|c| c.new_node )
                    .unwrap_or(n.to_owned())
            })
            .collect())
    }

    pub fn clean(&mut self, dag: &VoxelDAG64) {
        let mut new_changed_nodes = vec![];

        self.clean_internal(dag, &mut new_changed_nodes, self.new_root_index);

        self.node_changes = new_changed_nodes;
    } 

    fn clean_internal(&self, dag: &VoxelDAG64, new_changed_nodes: &mut Vec<DAG64NodeChange>, index: u32) {
        let node = if let Some(change) = self.node_changes.iter()
            .find(|c| c.index == index ) {
            new_changed_nodes.push(*change);
            change.new_node
        } else {
            dag.nodes.get(index as usize).unwrap()
        };

        if node.is_leaf() {
            return;
        }

        for i in node.range() {
            self.clean_internal(dag, new_changed_nodes, i as u32);
        }
    }

    pub fn apply(&mut self, dag: &mut VoxelDAG64) -> OctaResult<()> {
        dag.root_index = self.new_root_index;
        dag.offset = self.new_offset;
        dag.levels = self.new_levels;

        for change in self.node_changes.iter() {
            dag.nodes.set(change.index as usize, &[change.new_node])?;
        }

        Ok(())
    }
}

impl DAG64NodeChange {
    pub fn invert(&self) -> Self {
        Self {
            index: self.index,
            new_node: self.old_node,
            old_node: self.new_node,
        }
    }
}

impl VoxelDAG64 {
    pub fn create_transaction(&self) -> DAG64Transaction {
        DAG64Transaction { 
            node_changes: vec![], 
            old_root_index: self.root_index,
            new_root_index: self.root_index,
            old_levels: self.levels,
            new_levels: self.levels,
            old_offset: self.offset,
            new_offset: self.offset, 
        }
    }
}

impl From<&VoxelDAG64> for DAG64Transaction {
    fn from(value: &VoxelDAG64) -> Self { value.create_transaction() }
}
