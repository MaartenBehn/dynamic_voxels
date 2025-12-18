use octa_force::egui::{Color32, Id, Popup, PopupCloseBehavior, Sense, StrokeKind, Ui, WidgetInfo, WidgetType, color_picker::{Alpha, color_edit_button_srgb, color_picker_color32, show_color_at}};

use crate::voxel::palette::Palette;

pub fn palette_color_picker<P: Palette>(ui: &mut Ui, palette: &mut P, current_color: &mut [u8; 3]) -> bool {
    
    let color = Color32::from_rgb(current_color[0], current_color[1], current_color[2]);

    let popup_id = ui.auto_id_with("palette_picker");
    let open = Popup::is_id_open(ui.ctx(), popup_id);
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));
    let stroke_width = 1.0;

    if ui.is_rect_visible(rect) {
        let visuals = if open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        let rect = rect.expand(visuals.expansion);

        show_color_at(ui.painter(), color, rect.shrink(stroke_width));

        let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
        ui.painter().rect_stroke(
            rect,
            corner_radius,
            (stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
            StrokeKind::Inside,
        );
    }

    let mut changed = false;
    Popup::menu(&response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {

            ui.label("Pick color:");

            ui.horizontal_wrapped(|ui| {
                for (mat, color) in palette.colors() {
                    let color32 = Color32::from_rgb(color[0], color[1], color[2]);

                    let size = ui.spacing().interact_size;
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));

                    if ui.is_rect_visible(rect) {
                        let visuals = if open {
                            &ui.visuals().widgets.open
                        } else {
                            ui.style().interact(&response)
                        };
                        let rect = rect.expand(visuals.expansion);

                        show_color_at(ui.painter(), color32, rect.shrink(stroke_width));

                        let corner_radius = visuals.corner_radius.at_most(2); // Can't do more rounding because the background grid doesn't do any rounding
                        ui.painter().rect_stroke(
                            rect,
                            corner_radius,
                            (stroke_width, visuals.bg_fill), // Using fill for stroke is intentional, because default style has no border
                            StrokeKind::Inside,
                        );
                    }

                    if response.clicked() {
                        (*current_color) = color;
                        changed = true;
                        ui.close();
                    }
                }
            });

            ui.vertical(|ui| {
                ui.label("New: ");

                let id = Id::new("new_color");
                let mut color = ui.memory(|m| m.data.get_temp(id).unwrap_or(Color32::WHITE));

                if color_picker_color32(ui, &mut color, Alpha::Opaque) {
                    ui.memory_mut(|m| {
                        m.data.insert_temp(id, color);
                    });
                }

                if ui.button("Add").clicked() {
                    (*current_color) = [color.r(), color.g(), color.b()];  
                    changed = true;
                    ui.close();
                }
            });
        });
    changed
}
