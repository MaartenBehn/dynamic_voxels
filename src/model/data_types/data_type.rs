use egui_snarl::ui::PinInfo;
use octa_force::{egui::Color32, glam::{IVec2, IVec3, Vec2, Vec3A}};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<i32>), 
    NumberSpace,
    
    Position2D(Option<IVec2>), 
    Position3D(Option<IVec3>), 
    
    PositionSet2D,
    PositionSet3D,

    PositionPairSet2D,
    PositionPairSet3D,

    PositionSpace2D,
    PositionSpace3D,

    Volume2D,
    Volume3D,
}

impl PartialEq for ComposeDataType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
