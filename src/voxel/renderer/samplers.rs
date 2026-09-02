use octa_force::vulkan::ash;

include!(concat!(env!("OUT_DIR"), "/generated_samplers.rs"));

pub struct ReflectedSamplerContainer {
    pub samplers: Vec<vk::Sampler>,
}

impl ReflectedSamplerContainer {
    pub fn create(device: &ash::Device) -> Result<Self, vk::Result> {
        let mut samplers = Vec::new();

        for desc in REFLECTED_SAMPLERS {
            unsafe {
                let sampler = device.create_sampler(&desc.create_info, None)?;
                samplers.push(sampler);
            }
        }

        Ok(Self { samplers })
    }

    pub fn build_immutable_layout 

    pub fn build_immutable_layout_binding<'a>(
        &'a self,
        target_set: u32,
        target_binding: u32,
    ) -> Option<vk::DescriptorSetLayoutBinding<'a>> {
        for (i, desc) in REFLECTED_SAMPLERS.iter().enumerate() {
            if desc.set == target_set && desc.binding == target_binding {
                let sampler_slice = std::slice::from_ref(&self.samplers[i]);

                return Some(
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(desc.binding)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .descriptor_count(1)
                        .stage_flags(vk::ShaderStageFlags::ALL)
                        .immutable_samplers(sampler_slice),                
                );
            }
        }
        None
    }

    pub fn destroy(&self, device: &ash::Device) {
        for &sampler in &self.samplers {
            unsafe {
                device.destroy_sampler(sampler, None);
            }
        }
    }
}
