use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use octa_force::{OctaResult, anyhow::anyhow, glam::Vec3, vulkan::Buffer};

use crate::{scene::{dag_store::SceneDAGStore, dag64::SceneDAGObject, worker::{SceneObjectKey, SceneWorker}}, util::{aabb::AABB3, vector::Ve}};


#[derive(Debug)]
pub struct SceneObject {
    pub bvh_index: usize,
    pub changed: bool,
    pub data: SceneObjectType,
}

#[derive(Debug)]
pub enum SceneObjectType {
    DAG64(SceneDAGObject)
}

#[derive(Clone, Copy, Debug)]
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
        self.objects.insert(SceneObject { bvh_index: 0, changed: true, data: object })
    }

    pub fn remove_object(&mut self, key: SceneObjectKey) -> OctaResult<SceneObjectType> {
        self.needs_bvh_update = true;
        self.objects.remove(key)
            .map(|o| Ok(o.data))
            .unwrap_or(Err(anyhow!("Scene Object Key invalid")))
    }

    fn update_bvh(&mut self) -> OctaResult<()> {

        let mut objects = self.objects.values_mut().collect::<Vec<_>>();
        self.bvh = Bvh::build_par(&mut objects);

        let flat_bvh = self.bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let object = &objects[shape as usize];
                let nr = object.get_nr();
                let aabb = object.get_aabb();
                
                SceneObjectData {
                    min: aabb.min().to_vec3(),
                    child: (object.get_start() as u32) << 1 | 1,
                    max: aabb.max().to_vec3(),
                    exit: exit << 1 | nr,
                }
            } else {
                let aabb: AABB3 = aabb.into();
                SceneObjectData {
                    min: aabb.min().to_vec3(),
                    child: index << 1,
                    max: aabb.max().to_vec3(),
                    exit: exit,
                }
            } 
        });

        self.bvh_len = flat_bvh.len();
        let flat_bvh_size =  flat_bvh.len() * size_of::<SceneObjectData>();

        if self.bvh_allocation.size() < flat_bvh_size {
            self.allocator.dealloc(self.bvh_allocation);
            self.bvh_allocation = self.allocator.alloc(flat_bvh_size)?;
        }

        todo!();
        //self.buffer.copy_data_to_buffer_without_aligment(&flat_bvh, self.bvh_allocation.start());

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

    pub fn push_to_buffer(&mut self, buffer: &mut Buffer, dag_store: &SceneDAGStore) {
        match &mut self.data {
            SceneObjectType::DAG64(o) => o.push_to_buffer(buffer, dag_store),
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
}

impl Bounded<f32,3> for SceneObject {
    fn aabb(&self) -> Aabb<f32,3> {
        self.get_aabb().into()
    }
}

impl BHShape<f32,3> for SceneObject {
    fn set_bh_node_index(&mut self, index: usize) {
        self.bvh_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.bvh_index
    }
}
