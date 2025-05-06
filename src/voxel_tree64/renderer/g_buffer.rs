use std::iter;

use octa_force::{camera::Camera, glam::{IVec3, Mat4, UVec2, Vec3}, vulkan::{ash::vk, descriptor_heap::{self, DescriptorHandle, DescriptorHandleValue, DescriptorHeap}, gpu_allocator::MemoryLocation, Buffer, Context, Image, ImageAndView, ImageView}, OctaResult};

use crate::NUM_FRAMES_IN_FLIGHT;

#[derive(Debug)]
pub struct ImageAndViewAndHandle {
    pub image: Image,
    pub view: ImageView,
    pub handle: DescriptorHandle,
}

#[derive(Debug)]
pub struct GBuffer {
    pub history_len_tex: ImageAndViewAndHandle, 
    pub albedo_tex: [ImageAndViewAndHandle; NUM_FRAMES_IN_FLIGHT],
    pub irradiance_tex: [ImageAndViewAndHandle; NUM_FRAMES_IN_FLIGHT],
    pub depth_tex: [ImageAndViewAndHandle; NUM_FRAMES_IN_FLIGHT],
    pub moments_tex: [ImageAndViewAndHandle; NUM_FRAMES_IN_FLIGHT],

    pub prev_proj_mat: Mat4,
    pub prev_position: Vec3,

    pub uniform_buffer: Buffer,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GBufferUniform {
    proj_mat: Mat4,
    inv_proj_mat: Mat4,
    prev_proj_mat: Mat4,
    prev_inv_proj_mat: Mat4,

    position: IVec3,
    frame_no: u32,

    position_frac: Vec3,
    num_steady_frames: u32,

    prev_position_frac: Vec3,
    albedo_tex: DescriptorHandleValue,
    
    position_delta: Vec3,
    prev_albedo_tex: DescriptorHandleValue,
    
    irradiance_tex: [DescriptorHandleValue; NUM_FRAMES_IN_FLIGHT],
    
    depth_tex: [DescriptorHandleValue; NUM_FRAMES_IN_FLIGHT],
    moments_tex: [DescriptorHandleValue; NUM_FRAMES_IN_FLIGHT],
    
    history_len_tex: DescriptorHandleValue, 
}

impl GBuffer {
    pub fn new(context: &Context, res: UVec2, descriptor_heap: &mut DescriptorHeap, camera: &Camera) -> OctaResult<Self> {
        let base_flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC;

        let mut create_image = |format: vk::Format, usage: vk::ImageUsageFlags, depth: bool|
            -> OctaResult<ImageAndViewAndHandle> { 
            let image = context.create_image(
                base_flags | usage, 
                MemoryLocation::GpuOnly, 
                format, 
                res.x, res.y)?;

            let view = image.create_image_view(depth)?;
            
            let handle = descriptor_heap.create_image_handle(&view, base_flags | usage)?;

            Ok(ImageAndViewAndHandle {
                image,
                view,
                handle,
            })
        };

        let history_len_tex =  create_image(
            vk::Format::R8_UINT, vk::ImageUsageFlags::empty(), false)?;
         
        let mut create_images = |format: vk::Format, usage: vk::ImageUsageFlags, depth: bool|
            -> OctaResult<[ImageAndViewAndHandle; NUM_FRAMES_IN_FLIGHT]> {
            Ok([create_image(format, usage, depth)?, create_image(format, usage, depth)?])
        };
       
        let albedo_tex =  create_images(
            vk::Format::R8G8B8A8_UNORM, vk::ImageUsageFlags::COLOR_ATTACHMENT, false)?;
        
        let irradiance_tex =  create_images(
            vk::Format::R16G16B16_SFLOAT, vk::ImageUsageFlags::empty(), false)?;
        //let temp_irradiance_tex =  create_image(
        //    vk::Format::R16G16B16_SFLOAT, vk::ImageUsageFlags::empty(), false)?;
        
        let depth_tex =  create_images(
            vk::Format::R32_SFLOAT, vk::ImageUsageFlags::empty(), true)?;
        
        let moments_tex =  create_images(
            vk::Format::R16G16_SFLOAT, vk::ImageUsageFlags::empty(), false)?; 

        let uniform_buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
            MemoryLocation::CpuToGpu, 
            size_of::<GBufferUniform>() as _)?;


        let proj_mat = camera.projection_matrix().mul_mat4(&camera.view_matrix());
        let position = camera.position;

        let g_buffer = Self {
            history_len_tex,
            albedo_tex,
            irradiance_tex,
            depth_tex,
            moments_tex,
            uniform_buffer,

            prev_proj_mat: proj_mat,
            prev_position: position,
        };

        Ok(g_buffer)
    }

    pub fn push_uniform(&mut self, current_index: usize, frame_no: usize, camera: &Camera) -> OctaResult<()> {
        let last_index = 1 - current_index;
        let proj_mat = camera.projection_matrix().mul_mat4(&camera.view_matrix());
        let inv_proj_mat = proj_mat.inverse();
        let position = camera.position;

        let uniform = GBufferUniform { 
            proj_mat, 
            inv_proj_mat, 
            prev_proj_mat: self.prev_proj_mat, 
            prev_inv_proj_mat: self.prev_proj_mat.inverse(), 
            
            position: position.as_ivec3(),
            position_frac: position.fract(),
            prev_position_frac: self.prev_position.fract(),
            position_delta: position - self.prev_position,

            frame_no: frame_no as _,  
            num_steady_frames: 0, 

            albedo_tex: self.albedo_tex[current_index].handle.value,
            prev_albedo_tex: self.albedo_tex[last_index].handle.value, 
            irradiance_tex: [self.irradiance_tex[current_index].handle.value, self.albedo_tex[last_index].handle.value], 
            depth_tex: [self.depth_tex[current_index].handle.value, self.depth_tex[last_index].handle.value], 
            moments_tex: [self.moments_tex[current_index].handle.value, self.moments_tex[last_index].handle.value], 
            history_len_tex: self.history_len_tex.handle.value, 
        };

        self.uniform_buffer.copy_data_to_buffer(&[uniform])?;

        self.prev_proj_mat = proj_mat;
        self.prev_position = position;

        Ok(())
    }
}
