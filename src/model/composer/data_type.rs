use egui_snarl::ui::PinInfo;
use octa_force::{egui::Color32, glam::{Vec2, Vec3A}};

use crate::model::generation::number_range::NumberSet;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number(Option<f32>), 
    NumberSet,
    
    Position2D(Option<Vec2>), 
    Position3D(Option<Vec3A>), 
    PositionSet,

    Volume2D,
    Volume3D,
}

impl ComposeDataType {
    pub fn get_pin(self) ->  impl egui_snarl::ui::SnarlPin + 'static {
        match self {
            ComposeDataType::Number(..) => PinInfo::circle().with_fill(Color32::ORANGE),
            ComposeDataType::NumberSet => PinInfo::square().with_fill(Color32::ORANGE),
            ComposeDataType::Position2D(..) => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::Position3D(..) => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::PositionSet => PinInfo::square().with_fill(Color32::BLUE),
            ComposeDataType::Volume2D => PinInfo::square().with_fill(Color32::RED),
            ComposeDataType::Volume3D => PinInfo::square().with_fill(Color32::RED),
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
