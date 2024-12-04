use std::{iter, mem};
use log::{debug, info};
use octa_force::egui_ash_renderer::Renderer;
use octa_force::glam::{uvec2, vec2, vec3, vec4, UVec2, Vec2, Vec4};
use octa_force::{egui, ImageAndView, OctaResult};
use octa_force::egui::{Pos2, Ui, Widget};
use octa_force::egui::load::SizedTexture;
use octa_force::egui_extras::{Column, TableBuilder};
use octa_force::vulkan::ash::vk;
use octa_force::vulkan::{Buffer, Context, DescriptorPool, DescriptorSet, DescriptorSetLayout, Sampler, WriteDescriptorSet, WriteDescriptorSetKind};
use octa_force::vulkan::ash::vk::{Format};
use octa_force::vulkan::gpu_allocator::MemoryLocation;

pub const SCOPES: usize = 20;
pub const MAX_SAMPLE_RES_WIDTH: u32 = 30;

pub struct ShaderProfiler {
    out_data_len: usize,
    profiler_in_buffer: Buffer,
    profiler_out_buffer: Buffer,

    descriptor_pool: DescriptorPool,
    scopes: Vec<String>,
    main_scope: usize,

    sample_res: UVec2,
    sample_multiplication_factor: u32,
    active_sample_pixel: UVec2,
    active_pixel: UVec2,
    pixel_results: Vec<ShaderProfilerPixelData>,
    mean_pixel_results: ShaderProfilerPixelData,
    
    _egui_descriptor_layout: DescriptorSetLayout,
    egui_descriptor_sets: Vec<DescriptorSet>,

    result_overview_samplers: Vec<Sampler>,
    result_overview_images: Vec<ImageAndView>,
    result_overview_images_staging_buffers: Vec<Buffer>,
    result_overview_textures: Vec<SizedTexture>,
    current_result_overview_image: usize,

    last_colors: Vec<u32>,
    
    selected_result_pixel: Option<UVec2>
}

#[derive(Clone)]
pub struct ShaderProfilerPixelData {
    pub scope_data: Vec<ShaderProfilerScopeData>,
    pub total_time: u64,
    pub sample_count: usize,
}

#[derive(Clone, Copy)]
pub struct ShaderProfilerScopeData {
    pub percent_mean: f32,
    pub percent_max: f32,
    pub percent_min: f32,

    pub num_call_mean: f32,
    pub num_call_max: f32,
    pub num_call_min: f32,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct ProfilerInData {
    pub active_pixel_x: u32,
    pub active_pixel_y: u32,
}


impl ShaderProfiler {
    pub fn new(
        context: &Context,
        format: Format,
        res: UVec2,
        num_frames: usize,
        scopes: &[&str],
        egui_renderer: &mut Renderer,
    ) -> OctaResult<ShaderProfiler> {
        let profiler_in_buffer = context.create_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            size_of::<ProfilerInData>() as _,
        )?;

        let active_sample_pixel = uvec2(0,0);
        profiler_in_buffer.copy_data_to_buffer(&[ProfilerInData {
            active_pixel_x: active_sample_pixel.x,
            active_pixel_y: active_sample_pixel.y,
        }])?;
        
        let out_data_len = scopes.len() * 5;
        let profiler_size: usize = size_of::<u32>() * out_data_len;
        debug!("Profiler Buffer size: {} MB", profiler_size as f32 / 1000000.0);
        let profiler_out_buffer = context.create_buffer(
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::GpuToCpu,
            profiler_size as _,
        )?;

        let descriptor_pool = context.create_descriptor_pool(
            num_frames as u32,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: num_frames as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: (num_frames * 2) as u32,
                },
            ],
        )?;

        let main_scope = scopes.iter().position(|s| *s == "main").unwrap();
        let scopes: Vec<String> = scopes.into_iter().map(|s| s.to_string()).collect();

        let (sample_res, sample_multiplication_factor) = Self::get_sample_res(res);

        let pixel_results = Self::new_pixel_results(sample_res, scopes.len());
        let mean_pixel_results = ShaderProfilerPixelData {
            scope_data: vec![ShaderProfilerScopeData {
                percent_mean: 0.0,
                percent_min: 0.0,
                percent_max: 0.0,
                num_call_mean: 0.0,
                num_call_max: f32::MIN,
                num_call_min: f32::MAX,
            }; scopes.len()],
            total_time: 0,
            sample_count: 1,
        };

        let egui_descriptor_layout = context.create_descriptor_set_layout(&[
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ]).unwrap();

        let sampler_info = vk::SamplerCreateInfo::builder();


        let mut egui_descriptor_sets = Vec::with_capacity(3);

        let mut result_samplers = Vec::with_capacity(3);
        let mut result_images = Vec::with_capacity(3);
        let mut result_images_staging_buffers = Vec::with_capacity(3);
        let mut textures = Vec::with_capacity(3);

        for _ in 0..3 {
            let egui_descriptor_set = descriptor_pool.allocate_set(&egui_descriptor_layout).unwrap();
            let texture_id = egui_renderer.add_user_texture(egui_descriptor_set.inner, false);
            let texture = SizedTexture::new(texture_id, sample_res.as_vec2().as_ref());

            let sampler = context.create_sampler(&sampler_info)?;
            let (result_image, result_staging_buffer) = context.create_live_egui_texture_image(format, sample_res)?;

            egui_descriptor_set.update(&[
                WriteDescriptorSet {
                    binding: 0,
                    kind: WriteDescriptorSetKind::CombinedImageSampler {
                        layout: vk::ImageLayout::GENERAL,
                        view: &result_image.view,
                        sampler: &sampler,
                    },
                },
            ]);

            egui_descriptor_sets.push(egui_descriptor_set);
            result_samplers.push(sampler);

            result_images.push(result_image);
            result_images_staging_buffers.push(result_staging_buffer);
            textures.push(texture);
        }

        Ok(ShaderProfiler {
            profiler_in_buffer,
            profiler_out_buffer,
            descriptor_pool,
            out_data_len,
            scopes,
            main_scope,
            active_sample_pixel,
            active_pixel: active_sample_pixel,
            pixel_results,
            mean_pixel_results,

            sample_multiplication_factor,
            sample_res,

            _egui_descriptor_layout: egui_descriptor_layout,
            egui_descriptor_sets,
            result_overview_samplers: result_samplers,
            result_overview_images: result_images,
            result_overview_images_staging_buffers: result_images_staging_buffers,

            result_overview_textures: textures,
            current_result_overview_image: 0,
            last_colors: vec![0; 3],
            
            selected_result_pixel: None,
            
        })
    }

    fn new_pixel_results(res: UVec2, num_scopes: usize) -> Vec<ShaderProfilerPixelData> {
        let num = res.element_product() as usize;

        iter::repeat(ShaderProfilerPixelData {
            scope_data: vec![ShaderProfilerScopeData {
                percent_mean: 0.0,
                percent_min: 0.0,
                percent_max: 0.0,
                num_call_mean: 0.0,
                num_call_max: f32::MIN,
                num_call_min: f32::MAX,
            }; num_scopes],
            total_time: 0,
            sample_count: 0,
        }).take(num).collect()
    }

    fn get_sample_res(res: UVec2) -> (UVec2, u32) {
        let reduction_factor = MAX_SAMPLE_RES_WIDTH as f32 / res.x as f32;
        let y = res.y as f32 * reduction_factor;

        (uvec2(MAX_SAMPLE_RES_WIDTH, y as u32), (1.0 / reduction_factor) as u32)
    }

    pub fn descriptor_layout_bindings(&self) -> [vk::DescriptorSetLayoutBinding; 2] {
        [
            vk::DescriptorSetLayoutBinding {
                binding: 10,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 11,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
        ]
    }

    pub fn write_descriptor_sets(&self) -> [WriteDescriptorSet; 2] {
        [
            WriteDescriptorSet {
                binding: 10,
                kind: WriteDescriptorSetKind::UniformBuffer {
                    buffer: &self.profiler_in_buffer,
                },
            },
            WriteDescriptorSet {
                binding: 11,
                kind: WriteDescriptorSetKind::StorageBuffer {
                    buffer: &self.profiler_out_buffer,
                },
            },
        ]
    }

    pub fn update(&mut self, frame_index: usize, context: &Context) -> OctaResult<()> {
        self.process_results(frame_index, context)?;
        self.update_active_pixel()?;

        Ok(())
    }

    pub fn process_results(&mut self, frame_index: usize, context: &Context) -> OctaResult<()> {
        let pixel_index = (self.active_sample_pixel.y * self.sample_res.x + self.active_sample_pixel.x) as usize;

        let data: Vec<u32> = self.profiler_out_buffer.get_data_from_buffer(self.out_data_len)?;

        let total_start = data[self.main_scope * 5 + 1] as u64 + (data[self.main_scope * 5 + 2] as u64) << 32;
        let total_end = data[self.main_scope * 5 + 3] as u64 + (data[self.main_scope * 5 + 4] as u64) << 32;
        if total_end < total_start {
            return Ok(())
        }
        let total_time = total_end - total_start;
        self.pixel_results[pixel_index].total_time = total_time;
        
        let sample_count = self.pixel_results[pixel_index].sample_count + 1;
        self.pixel_results[pixel_index].sample_count = sample_count;

        let global_sample_count = self.mean_pixel_results.sample_count + 1;
        self.mean_pixel_results.sample_count = global_sample_count;
        
        //info!("Shader profile: {}", self.active_pixel);
        for (i, name) in self.scopes.iter().enumerate() {
            let counter = data[i * 5];
            let start = data[i * 5 + 1] as u64 + (data[i * 5 + 2] as u64) << 32;
            let end = data[i * 5 + 3] as u64 + (data[i * 5 + 4] as u64) << 32;
            if end < start {
                continue
            }

            let mut percent = ((end - start) as f64 / total_time as f64) as f32 * counter as f32;
            
            if !percent.is_finite() {
                percent = 0.0;
            }
            
            //info!("{name}: {counter} {:0.04}%", percent * 100.0);

            self.pixel_results[pixel_index].scope_data[i].add_mean_calls(counter as f32, sample_count as f32);
            self.pixel_results[pixel_index].scope_data[i].add_mean_percent(percent, sample_count as f32);

            self.mean_pixel_results.scope_data[i].add_mean_calls(counter as f32, global_sample_count as f32);
            self.mean_pixel_results.scope_data[i].add_mean_percent(percent, global_sample_count as f32);
        }

        self.mean_pixel_results.total_time = self.mean_pixel_results.total_time.max(total_time);

        let factor = (total_time as f64) / (self.mean_pixel_results.total_time as f64);
        let color = Self::get_debug_color_gradient_from_float(factor as f32);
        let color_u8 = Self::get_color_u8(color);

        self.last_colors.rotate_left(1);
        *self.last_colors.last_mut().unwrap() = color_u8;

        let next_result_image = (self.current_result_overview_image + 1) % 3;
        let staging_result_image = (self.current_result_overview_image + 2) % 3;
        let staging_result_image_2 = self.current_result_overview_image;
        
        let offset = pixel_index as isize - 2;
        if offset < 0 {
            let offset_pos = (offset * -1) as usize;
            let upper_index = self.sample_res.element_product() as usize - offset_pos;
            self.result_overview_images_staging_buffers[staging_result_image_2].copy_data_to_buffer_complex(&self.last_colors[offset_pos..], 0, align_of::<u32>())?;
            self.result_overview_images_staging_buffers[staging_result_image_2].copy_data_to_buffer_complex(&self.last_colors[..offset_pos], upper_index, align_of::<u32>())?;
        } else {
            self.result_overview_images_staging_buffers[staging_result_image_2].copy_data_to_buffer_complex(&self.last_colors, offset as usize, align_of::<u32>())?;
        }
        
        context.copy_live_egui_texture_staging_buffer_to_image(&self.result_overview_images_staging_buffers[staging_result_image], &self.result_overview_images[staging_result_image].image)?;

        self.current_result_overview_image = next_result_image;
        Ok(())
    }

    pub fn get_debug_color_gradient_from_float(x: f32) -> Vec4 {
        let firstColor = vec4(0.0, 1.0, 0.0, 1.0); // green
        let middleColor = vec4(0.0, 0.0, 1.0, 1.0); // blue
        let endColor = vec4(1.0, 0.0, 0.0, 1.0); // red

        if x == 0.0 {
            return firstColor;
        }

        let h = 0.5; // adjust position of middleColor
        if x < h {Vec4::lerp(firstColor, middleColor, x/h)} else {Vec4::lerp(middleColor, endColor, (x - h)/(1.0 - h))}
    }

    pub fn get_color_u8(mut color: Vec4) -> u32 {
        color *= 256.0;

        color.x as u32 | (color.y as u32) << 8 | (color.z as u32) << 16 | (color.w as u32) << 24
    }

    pub fn update_active_pixel(&mut self) -> OctaResult<()> {
        self.active_sample_pixel.x += 1;
        if self.active_sample_pixel.x >= self.sample_res.x {
            self.active_sample_pixel.y += 1;
            self.active_sample_pixel.x = 0;
        }
        if self.active_sample_pixel.y >= self.sample_res.y {
            self.active_sample_pixel.y = 0;
        }

        self.active_pixel = self.active_sample_pixel * self.sample_multiplication_factor;

        self.profiler_in_buffer.copy_data_to_buffer(&[ProfilerInData {
            active_pixel_x: self.active_pixel.x,
            active_pixel_y: self.active_pixel.y,
        }])?;

        Ok(())
    }

    pub fn gui_windows(&mut self, ctx: &egui::Context, mouse_left: bool) {
        
        let mut overview_pos = None;
        egui::Window::new("Shader Profile").show(ctx, |ui| {
            
            let pos = ui.next_widget_position();

            let image = egui::Image::new(self.result_overview_textures[self.current_result_overview_image])
                .shrink_to_fit();
            
            let size = image.calc_size(ui.available_size(), image.size());
            image.ui(ui);

            if let Some(Pos2{x, y}) = ui.ctx().pointer_latest_pos() {
                let pos = vec2(
                    (x - pos.x) * (self.sample_res.x as f32 / size.x),
                    (y - pos.y) * (self.sample_res.y as f32 / size.y)
                );
                
                if pos.x > 0.0 && pos.x < self.sample_res.x as f32 && pos.y > 0.0 && pos.y < self.sample_res.y as f32 {
                    overview_pos = Some(pos.as_uvec2());
                }
            }
        });
        
        if mouse_left {
            if let Some(pos) = overview_pos {
                self.selected_result_pixel = Some(pos);
            } else {
                self.selected_result_pixel = None;
            }
        }

        egui::Window::new("Pixel result").show(ctx, |ui| {
            let pixel = if let Some(pos) = self.selected_result_pixel {
                ui.label(format!("Selected: {pos}"));
                
                let pixel_index = (pos.y * self.sample_res.x + pos.x) as usize;
                 &self.pixel_results[pixel_index]
            } else {
                &self.mean_pixel_results
            };

            ui.label(format!("Samples: {}", pixel.sample_count));

            TableBuilder::new(ui)
                .column(Column::exact(200.0).resizable(true))
                .column(Column::exact(40.0).resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Name");
                    });
                    header.col(|ui| {
                        ui.label("Calls");
                    });
                    header.col(|ui| {
                        ui.label("Percent");
                    });
                })
                .body(|mut body| {

                    for (scope, name) in pixel.scope_data.iter().zip(self.scopes.iter()) {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{name}"));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:.02}", scope.num_call_mean));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:>2.03}%", scope.percent_mean * 100.0));
                            });
                        });
                    }
                });
        });
        
        
    }
    
    pub fn on_recreate_swapchain(&mut self, context: &Context, format: Format, res: UVec2) -> OctaResult<()> {
        (self.sample_res, self.sample_multiplication_factor) = Self::get_sample_res(res);
        self.pixel_results = Self::new_pixel_results(self.sample_res, self.scopes.len());

        let mut result_images = Vec::with_capacity(3);
        let mut result_images_staging_buffers = Vec::with_capacity(3);
        let mut result_textures = Vec::with_capacity(3);
        
        let num_pixel = self.sample_res.element_product() as usize;
        for i in 0..3 {
            let (result_image, result_staging_buffer) = context.create_live_egui_texture_image(format, self.sample_res)?;

            // Clear image
            result_staging_buffer.copy_data_to_buffer(&vec![0; num_pixel])?;
            context.copy_live_egui_texture_staging_buffer_to_image(&result_staging_buffer, &result_image.image)?;
            self.last_colors[i] = 0;

            self.egui_descriptor_sets[i].update(&[
                WriteDescriptorSet {
                    binding: 0,
                    kind: WriteDescriptorSetKind::CombinedImageSampler {
                        layout: vk::ImageLayout::GENERAL,
                        view: &result_image.view,
                        sampler: &self.result_overview_samplers[i],
                    },
                },
            ]);

            let texture = SizedTexture::new(self.result_overview_textures[i].id, self.sample_res.as_vec2().as_ref());

            result_images.push(result_image);
            result_images_staging_buffers.push(result_staging_buffer);
            result_textures.push(texture);
        }
        
        self.result_overview_images = result_images;
        self.result_overview_images_staging_buffers = result_images_staging_buffers;
        self.result_overview_textures = result_textures;

        Ok(())
    }
}

impl ShaderProfilerScopeData {
    pub fn add_mean_percent(&mut self, percent: f32, n: f32) {
        self.percent_mean = (self.percent_mean * (n -1.0) + percent) / n;
        self.percent_max = self.percent_max.max(percent);
        self.percent_min = self.percent_min.min(percent);
    }

    pub fn add_mean_calls(&mut self, calls: f32, n: f32) {
        self.num_call_mean = (self.num_call_mean * (n -1.0) + calls) / n;
        self.num_call_max = self.num_call_max.max(calls);
        self.num_call_min = self.num_call_min.min(calls);
    }
}