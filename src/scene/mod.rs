pub mod dag64;
pub mod renderer;

use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use dag64::DAG64SceneObject;
use octa_force::{anyhow::anyhow, glam::{vec3, Mat4, Vec3, Vec4Swizzles}, log::{debug, info}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};

use crate::{multi_data_buffer::buddy_buffer_allocator::{BuddyAllocation, BuddyBufferAllocator}, util::{aabb::AABB, math::to_mb}, VOXELS_PER_SHADER_UNIT};

new_key_type! { pub struct SceneObjectKey; }

#[derive(Debug)]
pub struct Scene {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
    pub objects: SlotMap<SceneObjectKey, SceneObject>,
    pub bvh: Bvh<f32, 3>,
    pub bvh_allocation: BuddyAllocation,
    pub bvh_len: usize,

    pub needs_bvh_update: bool,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SceneData {
    start_ptr: u64,
    bvh_offset: u32,
    bvh_len: u32,
}

#[derive(Debug)]
pub struct SceneObject {
    pub bvh_index: usize,
    pub changed: bool,
    pub data: SceneObjectType,
}

#[derive(Debug)]
pub enum SceneObjectType {
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
        let buffer_size = 2_usize.pow(20);
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
            objects: Default::default(),
            bvh,
            bvh_allocation,
            bvh_len: 0,
            needs_bvh_update: true,
        })
    }

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

        if self.bvh_allocation.size < flat_bvh_size {
            self.bvh_allocation = self.allocator.alloc(flat_bvh_size)?;
        }

        self.buffer.copy_data_to_buffer_without_aligment(&flat_bvh, self.bvh_allocation.start);

        self.needs_bvh_update = false;
        Ok(())
    }

    pub fn flush(&mut self) -> OctaResult<()> {
        for object in self.objects.values_mut() {
            if object.changed {
                object.push_to_buffer(&mut self.buffer);
                object.changed = false;
            }
        }

        if self.needs_bvh_update {
            self.update_bvh()?;
        }

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
        match self.data {
            SceneObjectType::DAG64(..) => 0,
        }
    }

    pub fn push_to_buffer(&mut self, buffer: &mut Buffer) {
        match &mut self.data {
            SceneObjectType::DAG64(dag) => dag.push_to_buffer(buffer),
        }
    }

    pub fn get_start(&self) -> usize {
        match &self.data {
            SceneObjectType::DAG64(dag) => dag.allocation.start,
        }
    }

    pub fn get_aabb(&self) -> AABB {
        match &self.data {
            SceneObjectType::DAG64(dag) => {
                dag.get_aabb()
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
        self.bvh_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.bvh_index
    }
}
