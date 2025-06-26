pub mod static_dag64;
pub mod renderer;

use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use octa_force::{glam::{vec3, Mat4, Vec3}, log::{debug, info}, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};
use static_dag64::StaticDAG64SceneObject;
use crate::{aabb::AABB, multi_data_buffer::buddy_controller::BuddyBufferAllocator, static_voxel_dag64::StaticVoxelDAG64};

new_key_type! { pub struct SceneObjectKey; }

#[derive(Debug)]
pub struct Scene {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
    pub objects: Vec<SceneObject>,
    pub bvh: Bvh<f32, 3>,
    pub bvh_allocation_start: usize,
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
    pub fn from_objects(context: &Context, mut objects: Vec<SceneObject>) -> OctaResult<Self> {
        
        let buffer_size = 2_usize.pow(20);
        info!("Scene Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);
        debug!("Scene Buffer size: {} byte", buffer_size);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;

        let allocator = BuddyBufferAllocator::new(buffer_size, 32);

        let bvh = Bvh::build_par(&mut objects);
        dbg!(&bvh);

        Ok(Scene {
            buffer,
            allocator,
            bvh,
            objects,
            bvh_allocation_start: 0,
            bvh_len: 0,
        })
    }

    pub fn init_buffer(&mut self) -> OctaResult<()> {
        self.allocator.clear();

        for object in self.objects.iter_mut() {
            object.push_to_buffer(&mut self.allocator, &mut self.buffer)?;
        }

        let flat_bvh = self.bvh.flatten_custom(&|aabb, index, exit, shape| {
            let leaf = shape != u32::MAX;

            if leaf {
                let object = &self.objects[shape as usize];
                let nr = object.get_nr();
                let aabb = object.get_aabb();
                
                SceneObjectData {
                    min: aabb.min,
                    child: (object.get_start() as u32) << 1 | 1,
                    max: aabb.max,
                    exit: exit << 1 | nr,
                }
            } else {
                let aabb: AABB = aabb.into();
                SceneObjectData {
                    min: aabb.min,
                    child: index << 1,
                    max: aabb.max,
                    exit: exit,
                }
            } 
        });

        dbg!(&flat_bvh);

        let flat_bvh_size =  flat_bvh.len() * size_of::<SceneObjectData>();
        debug!("Flat BVH Size: {flat_bvh_size}");

        let (start,  _) = self.allocator.alloc(flat_bvh_size)?;
        self.bvh_allocation_start = start;
        self.bvh_len = flat_bvh.len();
        
        self.buffer.copy_data_to_buffer_without_aligment(&flat_bvh, self.bvh_allocation_start)?;

        Ok(())
    }

    pub fn get_data(&self) -> SceneData {
        SceneData { 
            start_ptr: self.buffer.get_device_address(), 
            bvh_offset: self.bvh_allocation_start as _,
            bvh_len: self.bvh_len as _,
        }
    }
}


impl SceneObject {
    pub fn get_nr(&self) -> u32 {
        match self {
            SceneObject::StaticDAG64(..) => 0,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        match self {
            SceneObject::StaticDAG64(tree) => {
                tree.push_to_buffer(allocator, buffer)
            },
        }
    }

    pub fn get_start(&self) -> usize {
        match self {
            SceneObject::StaticDAG64(tree) => tree.alloc_start,
        }
    }

    pub fn get_aabb(&self) -> AABB {
        match self {
            SceneObject::StaticDAG64(tree64_scene_object) => {
                AABB::from_centered_size(&tree64_scene_object.mat, tree64_scene_object.tree.get_size())
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
            SceneObject::StaticDAG64(tree64_scene_object) => tree64_scene_object.bvh_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            SceneObject::StaticDAG64(tree64_scene_object) => tree64_scene_object.bvh_index,
        }
    }
}
