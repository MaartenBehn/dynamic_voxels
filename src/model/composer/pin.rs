use egui_snarl::ui::{PinInfo, PinWireInfo, WireStyle};
use octa_force::egui::{self, epaint::{CircleShape, PathShape, PathStroke}, Color32, Shape};

use super::{data_type::ComposeDataType};

pub struct ComposePin {
    data_type: ComposeDataType,
    valid: bool,
    not_valid_scale: f32,
}

impl ComposePin {
    pub fn new(data_type: ComposeDataType, valid: bool, not_valid_scale: f32) -> Self {
        Self { data_type, valid, not_valid_scale }
    }
}

fn draw_exclemation_mark(
    center: egui::Pos2,
    size: f32,
    painter: &egui::Painter,
) { 
    let points = vec![
        center + egui::vec2(-0.2, -0.7) * size,
        center + egui::vec2(0.2, -0.7) * size,
        center + egui::vec2(0.1, 0.1) * size,
        center + egui::vec2(-0.1, 0.1) * size,
    ];

    painter.add(Shape::Path(PathShape {
        points,
        closed: true,
        fill: Color32::RED,
        stroke: PathStroke::new(0.05 * size, Color32::WHITE).middle(),
    }));

    painter.add(Shape::Circle(CircleShape{ 
        center: center - egui::vec2(0.0, -0.5) * size, 
        radius: 0.2 * size, 
        fill: Color32::RED, 
        stroke: egui::Stroke { width: 0.05 * size, color: Color32::WHITE },
    }));
}


impl egui_snarl::ui::SnarlPin for ComposePin {
    fn draw(
        self,
        snarl_style: &egui_snarl::ui::SnarlStyle,
        style: &egui::Style,
        rect: egui::Rect,
        painter: &egui::Painter,
    ) -> egui_snarl::ui::PinWireInfo {

        let pin_info = match self.data_type {
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
        };

        let wire_info = pin_info.draw(snarl_style, style, rect, painter);

        if !self.valid {
            let center = rect.center();
            let size = f32::min(rect.width(), rect.height()) * self.not_valid_scale;

            draw_exclemation_mark(center, size, painter);
        }

        

        wire_info
    }
}


