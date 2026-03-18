use crate::voxel::renderer::g_buffer::ImageAndViewAndHandle;


#[derive(Debug)]
pub struct GIPoolAtlas {
    pub images: Vec<ImageAndViewAndHandle>,
}
