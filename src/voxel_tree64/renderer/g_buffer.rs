use std::iter;

use octa_force::{camera::Camera, glam::{self, IVec3, Mat4, Quat, UVec2, Vec3}, log::{debug, info}, vulkan::{ash::vk, descriptor_heap::{self, DescriptorHandle, DescriptorHandleValue, DescriptorHeap}, gpu_allocator::MemoryLocation, physical_device::PhysicalDeviceCapabilities, Buffer, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, Image, ImageAndView, ImageBarrier, ImageView, WriteDescriptorSet, WriteDescriptorSetKind}, OctaResult};

use crate::NUM_FRAMES_IN_FLIGHT;

#[derive(Debug)]
pub struct ImageAndViewAndHandle {
    pub image: Image,
    pub view: ImageView,
    pub handle: DescriptorHandle,
}

#[derive(Debug)]
pub struct GBuffer {
    pub albedo_tex: [ImageAndView; NUM_FRAMES_IN_FLIGHT],
    pub irradiance_tex: [ImageAndView; NUM_FRAMES_IN_FLIGHT],
    pub depth_tex: [ImageAndView; NUM_FRAMES_IN_FLIGHT],
    pub moments_tex: [ImageAndView; NUM_FRAMES_IN_FLIGHT],
    pub history_len_tex: ImageAndView,

    pub prev_proj_mat: Mat4,
    pub prev_position: Vec3,

    pub uniform_buffer: Buffer,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_layout: DescriptorSetLayout,
    pub descriptor_sets: Vec<DescriptorSet>, // Len of NUM_FRAMES_IN_FLIGHT
    
    pub frame_no: u32,
    pub num_steady_frames: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GBufferUniform {
    proj_mat: Mat4,
    inv_proj_mat: Mat4,
    prev_proj_mat: Mat4,
    prev_inv_proj_mat: Mat4,

    position: Vec3,
    frame_no: u32,

    position_frac: Vec3,
    num_steady_frames: u32,

    prev_position_frac: Vec3,
    fill_1: u32,
    
    position_delta: Vec3,
    fill_2: u32,
}

impl GBuffer {
    pub fn new(context: &Context, res: UVec2, camera: &Camera) -> OctaResult<Self> {

        let descriptor_pool = context.create_descriptor_pool(
            NUM_FRAMES_IN_FLIGHT as u32,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: NUM_FRAMES_IN_FLIGHT as u32 * 10,
                },
            ],
        )?;
 
        let (history_len_tex, 
            albedo_tex, 
            irradiance_tex,
            depth_tex, 
            moments_tex) = Self::create_image_datas(context, res)?;

        let textures = [
            vec![
                &albedo_tex[0], &albedo_tex[1],
                &irradiance_tex[0], &irradiance_tex[1],
                &depth_tex[0], &depth_tex[1],
                &moments_tex[0], &moments_tex[1],
                &history_len_tex,
            ],
            vec![
                &albedo_tex[1], &albedo_tex[0],
                &irradiance_tex[1], &irradiance_tex[0],
                &depth_tex[1], &depth_tex[0],
                &moments_tex[1], &moments_tex[0],
                &history_len_tex,
            ]
        ];

        let mut descriptor_layout_bindings = (0..textures[0].len())
            .map(|i| vk::DescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let descriptor_layout =
            context.create_descriptor_set_layout(&descriptor_layout_bindings)?;

        let mut descriptor_sets = Vec::new();
        for texture_set in textures {
            let descriptor_set = descriptor_pool.allocate_set(&descriptor_layout)?;

            let mut write_descriptor_sets = texture_set.iter()
                .enumerate()
                .map(|(i, tex)| 
                    WriteDescriptorSet {
                        binding: i as u32,
                        kind: WriteDescriptorSetKind::StorageImage {
                            layout: vk::ImageLayout::GENERAL,
                            view: &tex.view,
                        },
                    })
                .collect::<Vec<_>>();

            descriptor_set.update(&write_descriptor_sets);

            descriptor_sets.push(descriptor_set);
        }

 
        let uniform_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS, 
            MemoryLocation::GpuOnly, 
            size_of::<GBufferUniform>() as _)?;

        let proj_mat = camera.projection_matrix().mul_mat4(&camera.view_matrix());
        let position = camera.position;

        let g_buffer = Self {
            history_len_tex,
            albedo_tex,
            irradiance_tex,
            depth_tex,
            moments_tex,

            prev_proj_mat: proj_mat,
            prev_position: position,

            uniform_buffer,
            descriptor_pool,
            descriptor_layout,
            descriptor_sets,
            frame_no: 0,
            num_steady_frames: 0,
        };

        Ok(g_buffer)
    }

    fn create_image_datas(context: &Context, res: UVec2) -> 
    OctaResult<(ImageAndView, [ImageAndView; 2], [ImageAndView; 2], [ImageAndView; 2], [ImageAndView; 2])> {
        //debug!("Supported Image Formats: {:?}", context.physical_device.supported_image_formats);
        //debug!("Supported Depth Formats: {:?}", context.physical_device.supported_depth_formats);
         
        let base_flags = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC;

        let mut create_image = |format: vk::Format, usage: vk::ImageUsageFlags|
            -> OctaResult<ImageAndView> { 
            let image = context.create_image(
                base_flags | usage, 
                MemoryLocation::GpuOnly, 
                format, 
                res.x, res.y)?;

            let view = image.create_image_view(false)?;
            
            Ok(ImageAndView {
                image,
                view,
            })
        };

        let history_len_tex =  create_image(
            vk::Format::R8_UINT, vk::ImageUsageFlags::empty())?;
         
        let mut create_images = |format: vk::Format, usage: vk::ImageUsageFlags|
            -> OctaResult<[ImageAndView; NUM_FRAMES_IN_FLIGHT]> {
            Ok([create_image(format, usage)?, create_image(format, usage)?])
        };
       
        let albedo_tex =  create_images(
            context.physical_device.surface_format.format,   
            //vk::Format::R8G8B8A8_UNORM, 
            vk::ImageUsageFlags::COLOR_ATTACHMENT)?;
        
        let irradiance_tex =  create_images(
            vk::Format::R8G8B8A8_UNORM, vk::ImageUsageFlags::empty())?;
        //let temp_irradiance_tex =  create_image(
        //    vk::Format::R16G16B16_SFLOAT, vk::ImageUsageFlags::empty(), false)?;
        
        let depth_tex =  create_images(
            vk::Format::R32_SFLOAT, vk::ImageUsageFlags::empty())?;
         
        let moments_tex =  create_images(
            vk::Format::R16G16_SFLOAT, vk::ImageUsageFlags::empty())?;


        let barriers = iter::once(&history_len_tex)
            .chain(albedo_tex.iter())
            .chain(irradiance_tex.iter())
            .chain(depth_tex.iter())
            .chain(moments_tex.iter())
            .map(|t| 
                ImageBarrier {
                    image: &t.image,
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: vk::ImageLayout::GENERAL,
                    src_access_mask: vk::AccessFlags2::NONE,
                    dst_access_mask: vk::AccessFlags2::NONE,
                    src_stage_mask: vk::PipelineStageFlags2::NONE,
                    dst_stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
                }
            ).collect::<Vec<_>>();

        context.execute_one_time_commands(|cmd_buffer| {
            cmd_buffer.pipeline_image_barriers(&barriers);
        })?;

        Ok((history_len_tex, albedo_tex, irradiance_tex, depth_tex, moments_tex))
    }
 
    pub fn update(&mut self, camera: &Camera, context: &Context, res: UVec2) -> OctaResult<()> {
        let proj_mat = camera.projection_matrix().mul_mat4(&camera.view_matrix());
        let inv_proj_mat = Self::get_inverse_proj_screen_mat(proj_mat, res);
        let prev_inv_proj_mat = Self::get_inverse_proj_screen_mat(self.prev_proj_mat, res);
        
        let position = camera.position;

        if proj_mat != self.prev_proj_mat {
            self.num_steady_frames = 0;
        }

        let uniform = GBufferUniform { 
            proj_mat, 
            inv_proj_mat, 
            prev_proj_mat: self.prev_proj_mat, 
            prev_inv_proj_mat: prev_inv_proj_mat, 
            
            position: position,
            position_frac: position.fract(),
            prev_position_frac: self.prev_position.fract(),
            position_delta: position - self.prev_position,

            frame_no: self.frame_no,  
            num_steady_frames: self.num_steady_frames,

            fill_1: 0,
            fill_2: 0,
        };

        context.copy_data_to_gpu_only_buffer(&[uniform], &self.uniform_buffer)?;

        self.prev_proj_mat = proj_mat;
        self.prev_position = position;
        self.frame_no = self.frame_no.wrapping_add(1);
        self.num_steady_frames = self.num_steady_frames.wrapping_add(1);

        Ok(())
    }
    
    // Computes inverse projection matrix, scaled to take coordinates in range [0..viewSize] rather than [-1..1]
    pub fn get_inverse_proj_screen_mat(proj_mat: Mat4, res: UVec2) -> Mat4 {
        let mut inv_proj_mat = proj_mat.inverse();
        let translation_mat = Mat4::from_scale_rotation_translation(
            Vec3::new(2.0 / res.x as f32, 2.0 / res.y as f32, 1.0), 
            Quat::IDENTITY,
            Vec3::new(-1.0, -1.0 , 0.0));
        inv_proj_mat.mul_mat4(&translation_mat)
    }

    pub fn on_recreate_swapchain(&mut self, context: &Context, res: UVec2) -> OctaResult<()> {
        let (history_len_tex, 
            albedo_tex, 
            irradiance_tex,
            depth_tex, 
            moments_tex) = Self::create_image_datas(context, res)?;

        let textures = [
            vec![
                &albedo_tex[0], &albedo_tex[1],
                &irradiance_tex[0], &irradiance_tex[1],
                &depth_tex[0], &depth_tex[1],
                &moments_tex[0], &moments_tex[1],
                &history_len_tex,
            ],
            vec![
                &albedo_tex[1], &albedo_tex[0],
                &irradiance_tex[1], &irradiance_tex[0],
                &depth_tex[1], &depth_tex[0],
                &moments_tex[1], &moments_tex[0],
                &history_len_tex,
            ]
        ];

        for (descriotor_set, texture_set) in self.descriptor_sets.iter().zip(textures) {
            
            let writes = texture_set.iter()
                .enumerate()
                .map(|(i, tex)| WriteDescriptorSet {
                    binding: i as u32, 
                    kind: WriteDescriptorSetKind::StorageImage {
                        layout: vk::ImageLayout::GENERAL,
                        view: &tex.view,
                    },
                })
                .collect::<Vec<_>>();

            descriotor_set.update(&writes);
        }

        self.history_len_tex = history_len_tex;
        self.albedo_tex = albedo_tex;
        self.irradiance_tex = irradiance_tex;
        self.depth_tex = depth_tex;
        self.moments_tex = moments_tex;
        
        Ok(())
    }
}
