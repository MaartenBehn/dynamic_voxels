use core::slice;

use octa_force::{log::error, puffin_egui::puffin};

use crate::csg_tree::{
    controller::MAX_CGS_TREE_DATA_SIZE,
    tree::{CSGNodeData, CSGTree, AABB_PADDING},
};

const CGS_CHILD_TYPE_NONE: u32 = 0;
const CGS_CHILD_TYPE_UNION: u32 = 1;
const CGS_CHILD_TYPE_REMOVE: u32 = 2;
const CGS_CHILD_TYPE_INTERSECT: u32 = 3;
const CGS_CHILD_TYPE_VOXEL: u32 = 4;
const CGS_CHILD_TYPE_BOX: u32 = 5;
const CGS_CHILD_TYPE_SPHERE: u32 = 6;

impl CSGTree {
    pub fn make_data(&mut self) -> Vec<u32> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        self.set_all_aabbs(AABB_PADDING);

        let (data, _) = self.add_data(0, vec![]);

        if data.len() > MAX_CGS_TREE_DATA_SIZE {
            error!(
                "CGS Tree Data to large: {} of {}",
                data.len(),
                MAX_CGS_TREE_DATA_SIZE
            )
        }

        data
    }

    fn add_data(&self, index: usize, mut data: Vec<u32>) -> (Vec<u32>, u32) {
        let node = &self.nodes[index];

        let index = data.len();
        data.extend_from_slice(any_as_u32_slice(&node.aabb));

        let t = match node.data {
            CSGNodeData::Union(..) => CGS_CHILD_TYPE_UNION,
            CSGNodeData::Remove(..) => CGS_CHILD_TYPE_REMOVE,
            CSGNodeData::Intersect(..) => CGS_CHILD_TYPE_INTERSECT,
            CSGNodeData::Box(..) => CGS_CHILD_TYPE_BOX,
            CSGNodeData::Sphere(..) => CGS_CHILD_TYPE_SPHERE,
            CSGNodeData::VoxelVolume(..) => CGS_CHILD_TYPE_VOXEL,
            CSGNodeData::All(..) => unreachable!(),
        };

        match node.data {
            CSGNodeData::Union(child1, child2)
            | CSGNodeData::Remove(child1, child2)
            | CSGNodeData::Intersect(child1, child2) => {
                data.push(0);
                data.push(0);

                (data, data[index + 6]) = self.add_data(child1, data);
                (data, data[index + 7]) = self.add_data(child2, data);
            }
            CSGNodeData::Box(transform, mat) | CSGNodeData::Sphere(transform, mat) => {
                data.extend_from_slice(any_as_u32_slice(&transform.inverse()));
                data[index + 21] = mat as u32;
            }
            CSGNodeData::VoxelVolume(mat) => {
                data.push(mat as u32);
            }
            CSGNodeData::All(..) => unreachable!(),
        };

        (data, Self::node_data(index, t))
    }

    fn node_data(pointer: usize, t: u32) -> u32 {
        ((pointer as u32) << 4) + t
    }
}

fn any_as_u32_slice<T: Sized>(p: &T) -> &[u32] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u32, size_of::<T>() / 4) }
}
