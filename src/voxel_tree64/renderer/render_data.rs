use octa_force::{egui::Vec2, glam::Vec3};


#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
#[repr(C)]
pub struct RenderData {
    packed_data: [f32; 8],
}

impl RenderData {
    pub fn set_pos(&mut self, pos: Vec3) {
        self.packed_data[0] = pos.x;
        self.packed_data[1] = pos.y;
        self.packed_data[2] = pos.z;
    }
    pub fn get_pos(&mut self) -> Vec3 {
        Vec3::new(self.packed_data[0], self.packed_data[1], self.packed_data[2])
    }
    
    pub fn set_dir(&mut self, dir: Vec3) {
        self.packed_data[3] = dir.x;
        self.packed_data[4] = dir.y;
        self.packed_data[5] = dir.z;
    }
    pub fn get_dir(&mut self) -> Vec3 {
        Vec3::new(self.packed_data[3], self.packed_data[4], self.packed_data[5])
    }
 
    pub fn set_res(&mut self, res: Vec2) {
        self.packed_data[6] = res.x;
        self.packed_data[7] = res.y;
    }
    pub fn get_res(&mut self) -> Vec2 {
        Vec2::new(self.packed_data[6], self.packed_data[7])
    }
}

