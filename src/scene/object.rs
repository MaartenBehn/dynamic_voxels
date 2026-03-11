use std::mem::ManuallyDrop;

use itertools::Itertools;
use octa_force::{OctaResult, anyhow::anyhow, glam::{Mat4, Vec3}, log::debug, vulkan::Buffer};

use crate::{bvh::{Bvh, node::BHNode, shape::{BHShape, Shapes}}, scene::{dag_store::SceneDAGStore, dag64::SceneDAGObject, staging_copies::SceneStagingBuilder, worker::{SceneObjectKey, SceneWorker}}, util::aabb::{AABB, AABB3}};


#[derive(Debug)]
pub struct SceneObject {
    pub bvh_index: usize,
    pub needs_update: bool,
    pub data: SceneObjectType,
}

#[derive(Clone, Copy, Debug)]
pub struct BVHObject<'a> (&'a SceneObject);

#[derive(Clone, Copy, Debug)]
pub struct BVHExtraData {
    pub start: usize, 
    pub nr: u32,
}

#[derive(Debug)]
pub enum SceneObjectType {
    DAG64(SceneDAGObject)
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct SceneObjectData {
    min: Vec3,
    child: u32,
    max: Vec3,  
    exit: u32,
}

impl SceneWorker {
    pub fn add_object(&mut self, mut object: SceneObjectType) -> SceneObjectKey {
        self.needs_bvh_update = true;
        self.objects.insert(SceneObject { bvh_index: 0, needs_update: true, data: object })
    }

    pub fn remove_object(&mut self, key: SceneObjectKey) -> OctaResult<SceneObjectType> {
        self.needs_bvh_update = true;
        self.objects.remove(key)
            .map(|o| Ok(o.data))
            .unwrap_or(Err(anyhow!("Scene Object Key invalid")))
    }

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
        let flat_bvh_size =  self.bvh_len * size_of::<SceneObjectData>();

        if self.bvh_allocation.size() < flat_bvh_size {
            self.allocator.dealloc(self.bvh_allocation)?;
            self.bvh_allocation = self.allocator.alloc(flat_bvh_size)?;
        }

        builder.push(&self.bvh.nodes, self.bvh_allocation.start());
        builder.update_bvh(self.bvh_allocation.start() as u32, self.bvh_len as u32);

        self.needs_bvh_update = false;
        Ok(())
    }
}

impl SceneObject {
    pub fn get_nr(&self) -> u32 {
        match self.data {
            SceneObjectType::DAG64(..) => 0,
        }
    }

    pub fn get_start(&self) -> usize {
        match &self.data {
            SceneObjectType::DAG64(dag) => dag.allocation.start(),
        }
    }

    pub fn get_aabb(&self) -> AABB3 {
        match &self.data {
            SceneObjectType::DAG64(o) => o.get_aabb(),
        }
    }

    // TODO: Move mat into general Body
    pub fn get_mat(&self) -> Mat4 {
        match &self.data {
            SceneObjectType::DAG64(o) => o.mat,
        }
    }

    pub fn update_mat(&mut self, mat: Mat4) {
        match &mut self.data {
            SceneObjectType::DAG64(o) => {
                o.mat = mat
            },
        }
    }

    pub fn update(&mut self, dag_store: &SceneDAGStore, builder: &mut SceneStagingBuilder) {
        if !self.needs_update {
            return;
        }

        match &self.data {
            SceneObjectType::DAG64(o) => o.update(dag_store, builder),
        }

        self.needs_update = false;
    }

    pub fn get_dag_object(&self) -> &SceneDAGObject {
        match &self.data {
            SceneObjectType::DAG64(o) => o,
        }
    }

    pub fn get_dag_object_mut(&mut self) -> &mut SceneDAGObject {
        match &mut self.data {
            SceneObjectType::DAG64(o) => o,
        }
    }
}

impl BHNode<BVHExtraData, Vec3, f32, 3> for SceneObjectData {
    fn new<S>(aabb: AABB<Vec3, f32, 3>, exit_index: usize, shape_index: Option<(usize, BVHExtraData)>) -> Self {

        if let Some((_, extra_data)) = shape_index {

            Self {
                min: aabb.min(),
                child: (extra_data.start as u32) << 1 | 1,
                max: aabb.max(),
                exit: (exit_index as u32) << 1 | extra_data.nr,
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
            start: self.0.get_start(),
            nr: self.0.get_nr(),
        }
    }
}
