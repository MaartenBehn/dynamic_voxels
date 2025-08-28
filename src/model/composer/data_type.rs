use egui_snarl::ui::PinInfo;
use octa_force::{egui::Color32, glam::{IVec2, IVec3, Vec2, Vec3A}};

use crate::model::generation::number_range::NumberSet;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<i32>), 
    NumberSpace,
    
    Position2D(Option<IVec2>), 
    Position3D(Option<IVec3>), 
    PositionSpace,
    PositionSet,

    Volume2D,
    Volume3D,

    // Template 
    Ammount,
    Identifier,
}

impl ComposeDataType {
    pub fn get_pin(self) ->  impl egui_snarl::ui::SnarlPin + 'static {
        match self {
            ComposeDataType::Number(..) => PinInfo::circle().with_fill(Color32::ORANGE),
            ComposeDataType::NumberSpace => PinInfo::triangle().with_fill(Color32::ORANGE),
            ComposeDataType::Position2D(..) => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::Position3D(..) => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::PositionSpace => PinInfo::triangle().with_fill(Color32::BLUE),
            ComposeDataType::PositionSet => PinInfo::square().with_fill(Color32::BLUE),

            ComposeDataType::Volume2D => PinInfo::square().with_fill(Color32::RED),
            ComposeDataType::Volume3D => PinInfo::square().with_fill(Color32::RED),
            ComposeDataType::Ammount => PinInfo::square().with_fill(Color32::GREEN),
            ComposeDataType::Identifier => PinInfo::circle().with_fill(Color32::GREEN),
        }

    } 
}

impl PartialEq for ComposeDataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
