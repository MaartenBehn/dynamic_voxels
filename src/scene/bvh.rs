use itertools::Itertools;
use octa_force::{OctaResult, glam::Vec3, log::debug};

use crate::{bvh::{Bvh, node::BHNode, shape::{BHShape, Shapes}}, scene::{object::SceneObject, staging_copies::SceneStagingBuilder, worker::SceneWorker}, util::aabb::AABB};


#[derive(Clone, Copy, Debug)]
pub struct BVHObject<'a> (&'a SceneObject);

#[derive(Clone, Copy, Debug)]
pub struct BVHExtraData {
    pub start: usize, 
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BVHObjectData {
    min: Vec3,
    child: u32,
    max: Vec3,  
    exit: u32,
}

impl SceneWorker { 
    pub fn update_bvh(&mut self, builder: &mut SceneStagingBuilder) -> OctaResult<()> {
        if !self.needs_bvh_update {
            return Ok(());
        }

        #[cfg(debug_assertions)]
        debug!("Scene Worker: Update BVH");

        let objects = self.objects.values().into_iter().map(|o| BVHObject (o)).collect_vec();
        let mut indecies = (0..objects.len()).into_iter().collect_vec();

        self.bvh = Bvh::build_par(
            &objects, 
            &mut indecies);

        self.bvh_len = self.bvh.nodes.len();
        let flat_bvh_size =  self.bvh_len * size_of::<BVHObjectData>();

        if self.bvh_allocation.size() < flat_bvh_size {
            self.allocator.dealloc(self.bvh_allocation)?;
            self.bvh_allocation = self.allocator.alloc(flat_bvh_size)?;
        }

        builder.push(&self.bvh.nodes, self.bvh_allocation.start());

        self.needs_bvh_update = false;
        Ok(())
    }
}

impl BHNode<BVHExtraData, Vec3, f32, 3> for BVHObjectData {
    fn new<S>(aabb: AABB<Vec3, f32, 3>, exit_index: usize, shape_index: Option<(usize, BVHExtraData)>) -> Self {

        if let Some((_, extra_data)) = shape_index {

            Self {
                min: aabb.min(),
                child: (extra_data.start as u32) << 1 | 1,
                max: aabb.max(),
                exit: exit_index as u32,
            }
        } else {
            Self {
                min: aabb.min(),
                child: 0,
                max: aabb.max(),
                exit: exit_index as u32,
            }
        }
    }
}

impl<'a> BHShape<BVHExtraData, Vec3, f32, 3> for BVHObject<'a> {
    fn aabb(&self, shapes: &Shapes<BVHExtraData, Self, Vec3, f32, 3>) -> AABB<Vec3, f32, 3> {
        self.0.get_aabb().to_f()
    }

    fn extra_data(&self, shapes: &Shapes<BVHExtraData, Self, Vec3, f32, 3>) -> BVHExtraData {
        BVHExtraData {
            start: self.0.allocation.start(),
        }
    }
}
