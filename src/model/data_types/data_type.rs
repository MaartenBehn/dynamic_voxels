use egui_snarl::ui::PinInfo;
use octa_force::{egui::Color32, glam::{IVec2, IVec3, Vec2, Vec3A}};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<i32>), 
    NumberSpace,
    
    Position2D(Option<IVec2>), 
    Position3D(Option<IVec3>), 
    
    PositionSpace2D,
    PositionSpace3D,

    Volume2D,
    Volume3D,
    
    Material([u8; 3]),

    Creates,
}

impl PartialEq for ComposeDataType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
