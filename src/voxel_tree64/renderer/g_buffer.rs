use std::iter;

use octa_force::{glam::{Mat4, UVec2, Vec3}, vulkan::{ash::vk::{self, ImageView}, descriptor_heap::{self, DescriptorHandle, DescriptorHandleValue, DescriptorHeap}, gpu_allocator::MemoryLocation, Context, ImageAndView}, OctaResult};

#[derive(Debug)]
pub struct GBuffer {
    history_len_tex: (ImageAndView, DescriptorHandle), 
    albedo_tex: [(ImageAndView, DescriptorHandle); 2],
    irradiance_tex: [(ImageAndView, DescriptorHandle); 2],
    depth_tex: [(ImageAndView, DescriptorHandle); 2],
    moments_tex: [(ImageAndView, DescriptorHandle); 2],
}

#[derive(Debug)]
#[repr(C)]
pub struct GBufferUniform {
    proj_mat: Mat4,
    inv_proj_mat: Mat4,
    history_mat: Mat4,
    inv_history_mat: Mat4,

    origin_frac: Vec3,
    frame_no: u32,

    histry_origin_frac: Vec3,
    num_steady_frames: u32,

    origin_delta: Vec3,
    history_len_tex: DescriptorHandleValue, 
    
    albedo_tex: [DescriptorHandleValue; 2],
    irradiance_tex: [DescriptorHandleValue; 2],
    
    depth_tex: [DescriptorHandleValue; 2],
    moments_tex: [DescriptorHandleValue; 2],
}

impl GBuffer {
    pub fn new(context: &Context, res: UVec2, descriptor_heap: &mut DescriptorHeap) -> OctaResult<Self> {
        let base_flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC;

        let mut create_image = |format: vk::Format, usage: vk::ImageUsageFlags, depth: bool|
            -> OctaResult<(ImageAndView, DescriptorHandle)> { 
            let image = context.create_image(
                base_flags | usage, 
                MemoryLocation::GpuOnly, 
                format, 
                res.x, res.y)?;

            let view = image.create_image_view(depth)?;
            let image_and_view = ImageAndView { image, view };
            
            let handle = descriptor_heap.create_image_handle(&image_and_view.view, base_flags | usage)?;

            Ok((image_and_view, handle))
        };

        let history_len_tex =  create_image(
            vk::Format::R8_UINT, vk::ImageUsageFlags::empty(), false)?;
         
        let mut create_images = |format: vk::Format, usage: vk::ImageUsageFlags, depth: bool|
            -> OctaResult<[(ImageAndView, DescriptorHandle); 2]> {
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
         
        Ok(Self {
            history_len_tex,
            albedo_tex,
            irradiance_tex,
            depth_tex,
            moments_tex,
        })
    }
}
