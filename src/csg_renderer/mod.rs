pub mod data_controller;
pub mod color_controller;

use crate::profiler::ShaderProfiler;
use color_controller::ColorController;
use octa_force::anyhow::Result;
use octa_force::camera::Camera;
use octa_force::engine::Engine;
use octa_force::glam::{UVec2, Vec3};
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::gpu_allocator::MemoryLocation;
use octa_force::vulkan::{
    Buffer, CommandBuffer, ComputePipeline, ComputePipelineCreateInfo, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, ImageAndView, PipelineLayout, Swapchain, WriteDescriptorSet, WriteDescriptorSetKind
};
use std::mem::size_of;
use std::time::Duration;

use data_controller::{DataController, MAX_DATA_BUFFER_SIZE};

const RENDER_DISPATCH_GROUP_SIZE_X: u32 = 32;
const RENDER_DISPATCH_GROUP_SIZE_Y: u32 = 32;
 
#[derive(Debug)]
#[allow(dead_code)]
pub struct CSGRenderer {
    storage_images: Vec<ImageAndView>,
    render_buffer: Buffer,

    descriptor_pool: DescriptorPool,
    descriptor_layout: DescriptorSetLayout,
    descriptor_sets: Vec<DescriptorSet>,
    pipeline_layout: PipelineLayout,
    pipeline: ComputePipeline,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct CSGRenderBuffer {
    pub pos: Vec3,
    pub screen_size_x: f32,
    pub dir: Vec3,
    pub screen_size_y: f32,
    pub time: f32,
    fill1: f32,
    fill2: f32,
    fill3: f32,
}

impl CSGRenderer {
    pub fn new(
        context: &Context,
        res: UVec2,
        num_frames: usize,
        csg_controller: &DataController,
        color_controller: &ColorController,
        profiler: &Option<ShaderProfiler>,
        shader_bin: &[u8],
    ) -> Result<CSGRenderer> {
        let storage_images = context.create_storage_images(res, num_frames)?;

        let render_buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            size_of::<CSGRenderBuffer>() as _,
        )?;

        let descriptor_pool = context.create_descriptor_pool(
            num_frames as u32,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: num_frames as u32,
                },
            ],
        )?;

        let mut descriptor_layout_bindings = vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 3,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
        ];

        if profiler.is_some() {
            descriptor_layout_bindings
                .extend_from_slice(&profiler.as_ref().unwrap().descriptor_layout_bindings());
        }

        let descriptor_layout =
            context.create_descriptor_set_layout(&descriptor_layout_bindings)?;

        let mut descriptor_sets = Vec::new();
        for i in 0..num_frames {
            let descriptor_set = descriptor_pool.allocate_set(&descriptor_layout)?;

            let mut write_descriptor_sets = vec![
                WriteDescriptorSet {
                    binding: 0,
                    kind: WriteDescriptorSetKind::StorageImage {
                        layout: vk::ImageLayout::GENERAL,
                        view: &storage_images[i].view,
                    },
                },
                WriteDescriptorSet {
                    binding: 1,
                    kind: WriteDescriptorSetKind::UniformBuffer {
                        buffer: &render_buffer,
                    },
                },
                WriteDescriptorSet {
                    binding: 2,
                    kind: WriteDescriptorSetKind::StorageBuffer {
                        buffer: &csg_controller.buffer,
                    },
                },
                WriteDescriptorSet {
                    binding: 3,
                    kind: WriteDescriptorSetKind::UniformBuffer {
                        buffer: &color_controller.buffer,
                    },
                },
            ];

            if profiler.is_some() {
                write_descriptor_sets
                    .extend_from_slice(&profiler.as_ref().unwrap().write_descriptor_sets());
            }

            descriptor_set.update(&write_descriptor_sets);

            descriptor_sets.push(descriptor_set);
        }

        let pipeline_layout = context.create_pipeline_layout(&[&descriptor_layout], &[])?;

        let pipeline = context.create_compute_pipeline(
            &pipeline_layout,
            ComputePipelineCreateInfo {
                shader_source: shader_bin,
            },
        )?;

        Ok(CSGRenderer {
            storage_images,
            render_buffer,

            descriptor_pool,
            descriptor_layout,
            descriptor_sets,

            pipeline_layout,
            pipeline,
        })
    }

    pub fn update(&self, camera: &Camera, res: UVec2, time: Duration) -> Result<()> {
        self.render_buffer.copy_data_to_buffer(&[CSGRenderBuffer::new(
            camera.position,
            camera.direction,
            res,
            time.as_secs_f32(),
        )])?;
        Ok(())
    }

    pub fn render(
        &self,
        buffer: &CommandBuffer,
        engine: &Engine,
    ) -> Result<()> {
        buffer.bind_compute_pipeline(&self.pipeline);

        buffer.bind_descriptor_sets(
            vk::PipelineBindPoint::COMPUTE,
            &self.pipeline_layout,
            0,
            &[&self.descriptor_sets[engine.get_current_in_flight_frame_index()]],
        );

        buffer.dispatch(
            (engine.get_resolution().x / RENDER_DISPATCH_GROUP_SIZE_X) + 1,
            (engine.get_resolution().y / RENDER_DISPATCH_GROUP_SIZE_Y) + 1,
            1,
        );

        buffer.swapchain_image_copy_from_compute_storage_image(
            &self.storage_images[engine.get_current_in_flight_frame_index()].image,
            &engine.get_current_swapchain_image_and_view().image,
        )?;

        Ok(())
    }

    pub fn on_recreate_swapchain(
        &mut self,
        context: &Context,
        num_frames: usize,
        res: UVec2,
    ) -> Result<()> {
        self.storage_images = context.create_storage_images(res, num_frames)?;

        for (i, descriotor_set) in self.descriptor_sets.iter().enumerate() {
            descriotor_set.update(&[WriteDescriptorSet {
                binding: 0,
                kind: WriteDescriptorSetKind::StorageImage {
                    layout: vk::ImageLayout::GENERAL,
                    view: &self.storage_images[i].view,
                },
            }]);
        }

        Ok(())
    }
}

impl CSGRenderBuffer {
    pub fn new(pos: Vec3, dir: Vec3, res: UVec2, time: f32) -> CSGRenderBuffer {
        CSGRenderBuffer {
            pos,
            dir,
            screen_size_x: res.x as f32,
            screen_size_y: res.y as f32,
            time,
            fill1: 0.0,
            fill2: 0.0,
            fill3: 0.0,
        }
    }
}
