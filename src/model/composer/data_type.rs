use egui_snarl::ui::PinInfo;
use octa_force::{egui::Color32, glam::{IVec2, IVec3, Vec2, Vec3A}};

use crate::model::generation::number_range::NumberSet;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<i32>), 
    NumberSpace,
    
    Position2D(Option<IVec2>), 
    Position3D(Option<IVec3>), 
    
    PositionSpace2D,
    PositionSet2D,

    PositionSpace3D,
    PositionSet3D,

    Volume2D,
    Volume3D,

    // Template 
    Ammount,
    Identifier,
}



impl PartialEq for ComposeDataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
