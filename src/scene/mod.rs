pub mod static_dag64;
pub mod dag64;
pub mod renderer;

use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use dag64::DAG64SceneObject;
use octa_force::{glam::{vec3, Mat4, Vec3, Vec4Swizzles}, log::{debug, info}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};
use static_dag64::StaticDAG64SceneObject;

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, util::{aabb::AABB, math::to_mb}, VOXELS_PER_SHADER_UNIT};

new_key_type! { pub struct SceneObjectKey; }



#[derive(Debug)]
pub struct Scene {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
    pub objects: Vec<SceneObject>,
    pub bvh: Bvh<f32, 3>,
    pub bvh_allocation: BuddyAllocation,
    pub bvh_len: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SceneData {
    start_ptr: u64,
    bvh_offset: u32,
    bvh_len: u32,
}

#[derive(Debug)]
pub enum SceneObject {
    StaticDAG64(StaticDAG64SceneObject),
    DAG64(DAG64SceneObject)
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SceneObjectData {
    min: Vec3,
    child: u32,
    max: Vec3,  
    exit: u32,
}

impl Scene {
    pub fn new(context: &Context) -> OctaResult<Self> {
        let buffer_size = 2_usize.pow(30);
        info!("Scene Buffer size: {:.04} MB", to_mb(buffer_size));

        let mut buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;

        let mut allocator = BuddyBufferAllocator::new(buffer_size, 32);
        let bvh = Bvh::build::<SceneObject>(&mut []);
        let bvh_allocation = allocator.alloc(1024)?;

        Ok(Scene {
            buffer,
            allocator,
            objects: vec![],
            bvh,
            bvh_allocation,
            bvh_len: 0,
        })

    }

    pub fn add_objects(&mut self, mut objects: Vec<SceneObject>) -> OctaResult<()> {
       
        for object in objects.iter_mut() {
            object.push_to_buffer(&mut self.allocator, &mut self.buffer)?;
        }

        self.objects.append(&mut objects);
        drop(objects);

        self.bvh = Bvh::build_par(&mut self.objects);
        
        let flat_bvh = self.bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let object = &self.objects[shape as usize];
                let nr = object.get_nr();
                let aabb = object.get_aabb();
                
                SceneObjectData {
                    min: aabb.min.xyz(),
                    child: (object.get_start() as u32) << 1 | 1,
                    max: aabb.max.xyz(),
                    exit: exit << 1 | nr,
                }
            } else {
                let aabb: AABB = aabb.into();
                SceneObjectData {
                    min: aabb.min.xyz(),
                    child: index << 1,
                    max: aabb.max.xyz(),
                    exit: exit,
                }
            } 
        });

        self.bvh_len = flat_bvh.len();
        let flat_bvh_size =  flat_bvh.len() * size_of::<SceneObjectData>();
        debug!("Flat BVH Size: {flat_bvh_size}");

        if self.bvh_allocation.size < flat_bvh_size {
            self.bvh_allocation = self.allocator.alloc(flat_bvh_size)?;
        }

        self.buffer.copy_data_to_buffer_without_aligment(&flat_bvh, self.bvh_allocation.start);

        Ok(())
    }
 
    pub fn get_data(&self) -> SceneData {
        SceneData { 
            start_ptr: self.buffer.get_device_address(), 
            bvh_offset: self.bvh_allocation.start as _,
            bvh_len: self.bvh_len as _,
        }
    }
}


impl SceneObject {
    pub fn get_nr(&self) -> u32 {
        match self {
            SceneObject::StaticDAG64(..) => 0,
            SceneObject::DAG64(..) => 0,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        match self {
            SceneObject::StaticDAG64(dag) => dag.push_to_buffer(allocator, buffer),
            SceneObject::DAG64(dag) => dag.push_to_buffer(allocator, buffer),
        }
    }

    pub fn get_start(&self) -> usize {
        match self {
            SceneObject::StaticDAG64(dag) => dag.get_allocation().start,
            SceneObject::DAG64(dag) => dag.get_allocation().start,
        }
    }

    pub fn get_aabb(&self) -> AABB {
        match self {
            SceneObject::StaticDAG64(dag) => {
                AABB::from_centered_size(&dag.mat, dag.dag.get_size())
            },
            SceneObject::DAG64(dag) => {
                AABB::from_centered_size(&dag.mat, dag.dag.get_size() / VOXELS_PER_SHADER_UNIT as f32)
            },
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
        match self {
            SceneObject::StaticDAG64(dag) => dag.bvh_index = index,
            SceneObject::DAG64(dag) => dag.bvh_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            SceneObject::StaticDAG64(dag) => dag.bvh_index,
            SceneObject::DAG64(dag) => dag.bvh_index,
        }
    }
}
