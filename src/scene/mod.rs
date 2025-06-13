pub mod tree64;

use bvh::{aabb::{Aabb, Bounded}, bounding_hierarchy::{BHShape, BoundingHierarchy}, bvh::Bvh};
use octa_force::{glam::{Mat4, Vec3}, log::info, vulkan::{ash::vk, gpu_allocator::MemoryLocation, Buffer, Context}, OctaResult};
use slotmap::{new_key_type, SlotMap};
use tree64::Tree64SceneObject;
use crate::{aabb::AABB, buddy_controller::BuddyBufferAllocator, voxel_tree64::VoxelTree64};
use borrow::partial as p;
use borrow::traits::*;

new_key_type! { pub struct SceneObjectKey; }

//#[derive(borrow::Partial)]
//#[module(crate::scene)]
pub struct Scene {
    pub buffer: Buffer,
    pub allocator: BuddyBufferAllocator,
    pub objects: Vec<SceneObject>,
    pub bvh: Bvh<f32, 3>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SceneData {
  start_ptr: u64,
  bvh_offset: u32,
  root_object_index: u32,
}

pub enum SceneObject {
    Tree64(Tree64SceneObject),
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SceneObjectData {
    min: Vec3,
    child: u32,
    max: Vec3,  
    exit: u32,
}

impl Scene {
    pub fn from_objects(context: Context, mut objects: Vec<SceneObject>) -> OctaResult<Self> {
        
        let buffer_size = 2_usize.pow(4);
        info!("Scene Buffer size: {:.04} MB", buffer_size as f32 * 0.000001);

        let buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS_KHR,
            MemoryLocation::CpuToGpu,
            buffer_size as _,
        )?;

        let allocator = BuddyBufferAllocator::new(buffer_size);

        let bvh = Bvh::build_par(&mut objects); 

        Ok(Scene {
            buffer,
            allocator,
            bvh,
            objects,
        })
    }

    pub fn init_buffer(&mut self) -> OctaResult<()> {
        self.allocator.clear();

        for object in self.objects.iter_mut() {
            object.push_to_buffer(&mut self.allocator, &mut self.buffer)?;
        }

        let flat_bvh = self.bvh.flatten_custom(&|aabb, index, exit, shape| {
            let aabb: AABB = aabb.into();
            let leaf = shape != u32::MAX;

            if leaf {
                let object = &self.objects[shape as usize];
                let nr = object.get_nr();
                
                SceneObjectData {
                    min: aabb.min,
                    child: (object.get_start() as u32) << 1 | 1,
                    max: aabb.max,
                    exit: exit << 1 | nr,
                }
            } else {
                SceneObjectData {
                    min: aabb.min,
                    child: index << 1,
                    max: aabb.max,
                    exit: exit,
                }
            } 
        });

        let (start,  _) = self.allocator.alloc(flat_bvh.len() * size_of::<SceneObjectData>())?;
        self.buffer.copy_data_to_buffer_without_aligment(&flat_bvh, start)?;

        Ok(())
    }
}


impl SceneObject {
    pub fn get_nr(&self) -> u32 {
        match self {
            SceneObject::Tree64(..) => 0,
        }
    }

    pub fn push_to_buffer(&mut self, allocator: &mut BuddyBufferAllocator, buffer: &mut Buffer) -> OctaResult<()> {
        match self {
            SceneObject::Tree64(tree) => {
                tree.push_to_buffer(allocator, buffer)
            },
        }
    }

    pub fn get_start(&self) -> usize {
        match self {
            SceneObject::Tree64(tree) => tree.alloc_start,
        }
    }
}

impl Bounded<f32,3> for SceneObject {
    fn aabb(&self) -> Aabb<f32,3> {
        match self {
            SceneObject::Tree64(tree64_scene_object) => {
                let aabb = AABB::from_box(&tree64_scene_object.mat, 0.0);
                aabb.into()
            },
        }
    }
}

impl BHShape<f32,3> for SceneObject {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            SceneObject::Tree64(tree64_scene_object) => tree64_scene_object.bvh_index = index,
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            SceneObject::Tree64(tree64_scene_object) => tree64_scene_object.bvh_index,
        }
    }
}
