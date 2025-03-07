use core::slice;
use std::iter;

use fdg::nalgebra::min;
use octa_force::{glam::Mat4, log::{debug, error, info}, puffin_egui::puffin};

use crate::csg_tree::{
    controller::MAX_CSG_TREE_DATA_SIZE,
    tree::{CSGNodeData, CSGTree, AABB_PADDING, CSG_PARENT_NONE},
};

const CSG_CHILD_TYPE_NONE: u32 = 0;
const CSG_CHILD_TYPE_UNION: u32 = 1;
const CSG_CHILD_TYPE_REMOVE: u32 = 2;
const CSG_CHILD_TYPE_INTERSECT: u32 = 3;
const CSG_CHILD_TYPE_MAT: u32 = 4;
const CSG_CHILD_TYPE_BOX: u32 = 5;
const CSG_CHILD_TYPE_SPHERE: u32 = 6;
const CSG_CHILD_TYPE_VOXEL_GRID: u32 = 7;

const CSG_DATA_AABB_SIZE: usize = 6;
const CSG_DATA_TRANSFORM_SIZE: usize = 12;

impl CSGTree {
    pub fn make_data(&mut self) -> Vec<u32> {
        #[cfg(debug_assertions)]
        puffin::profile_function!();

        self.set_parents(0, CSG_PARENT_NONE);
        self.set_all_aabbs(AABB_PADDING);

        let mut data = vec![0];
        (data, data[0]) = self.add_data(0, data);

        if data.len() > MAX_CSG_TREE_DATA_SIZE {
            error!(
                "CSG Tree Data to large: {} of {}",
                data.len(),
                MAX_CSG_TREE_DATA_SIZE
            )
        }

        data
    }

    fn add_data(&self, index: usize, mut data: Vec<u32>) -> (Vec<u32>, u32) {
        let node = &self.nodes[index];

        let index = data.len();
        data.extend_from_slice(any_as_u32_slice(&node.aabb));

        let t = match node.data {
            CSGNodeData::Union(..) => CSG_CHILD_TYPE_UNION,
            CSGNodeData::Remove(..) => CSG_CHILD_TYPE_REMOVE,
            CSGNodeData::Intersect(..) => CSG_CHILD_TYPE_INTERSECT,
            CSGNodeData::Mat(_, _) => CSG_CHILD_TYPE_MAT,
            CSGNodeData::Box(..) => CSG_CHILD_TYPE_BOX,
            CSGNodeData::Sphere(..) => CSG_CHILD_TYPE_SPHERE,
            CSGNodeData::VoxelGrid(..) => CSG_CHILD_TYPE_VOXEL_GRID,
            CSGNodeData::All(..) => unreachable!(),
        };

        match &node.data {
            CSGNodeData::Union(c1, c2)
            | CSGNodeData::Remove(c1, c2)
            | CSGNodeData::Intersect(c1, c2) => {
                data.push(0);
                data.push(0);

                (data, data[index + CSG_DATA_AABB_SIZE]) = self.add_data(*c1, data);
                (data, data[index + CSG_DATA_AABB_SIZE + 1]) = self.add_data(*c2, data);
            }
            CSGNodeData::Mat(transform, c1) => {
                write_mat4(&mut data, &transform.inverse());
                data.push(0);

                (data, data[index + CSG_DATA_AABB_SIZE + CSG_DATA_TRANSFORM_SIZE]) = self.add_data(*c1, data);
            }
            CSGNodeData::Box(transform, mat) | CSGNodeData::Sphere(transform, mat) => {
                write_mat4(&mut data, &transform.inverse());
                data.push(*mat as u32);
            }
            CSGNodeData::VoxelGrid(grid, pos) => {
                let min = -(grid.size / 2).as_vec3() + pos.as_vec3();
                let max = (grid.size / 2).as_vec3() + pos.as_vec3();

                data.extend_from_slice(any_as_u32_slice(&min));
                data.extend_from_slice(any_as_u32_slice(&max));
                data.extend_from_slice(u8_as_u32_slice(&grid.data));
            }
            CSGNodeData::All(..) => unreachable!(),
        };

        (data, Self::node_data(index, t))
    }

    fn node_data(pointer: usize, t: u32) -> u32 {
        ((pointer as u32) << 4) + t
    }
}

fn write_mat4(data: &mut Vec<u32>, mat: &Mat4) {
    data.extend(
        any_as_u32_slice(&mat.to_cols_array())
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 3 && *i != 7 && *i != 11 && *i != 15 )
            .map(|(_, &d)|d)
    );
}

fn any_as_u32_slice<T: Sized>(p: &T) -> &[u32] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u32, size_of::<T>() / 4) }
}

fn u8_as_u32_slice(p: &Vec<u8>) -> &[u32] {
    unsafe { 
        let (prefix, data, sufix) = p.align_to::<u32>();
        assert!(prefix.is_empty());
        assert!(sufix.is_empty());
        data
    }
}
