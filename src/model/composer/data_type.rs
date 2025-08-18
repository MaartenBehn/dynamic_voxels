use egui_snarl::ui::PinInfo;
use octa_force::egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComposeDataType {
    Number, 
    NumberSet, 
    
    Position2D, 
    Position3D, 
    PositionSet,

    Volume2D,
    Volume3D,
}

impl ComposeDataType {
    pub fn get_pin(self) ->  impl egui_snarl::ui::SnarlPin + 'static {
        match self {
            ComposeDataType::Number => PinInfo::circle().with_fill(Color32::ORANGE),
            ComposeDataType::NumberSet => PinInfo::square().with_fill(Color32::ORANGE),
            ComposeDataType::Position2D => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::Position3D => PinInfo::circle().with_fill(Color32::BLUE),
            ComposeDataType::PositionSet => PinInfo::square().with_fill(Color32::BLUE),
            ComposeDataType::Volume2D => PinInfo::square().with_fill(Color32::RED),
            ComposeDataType::Volume3D => PinInfo::square().with_fill(Color32::RED),
        }

    } 
}
