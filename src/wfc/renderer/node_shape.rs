use egui_graphs::{DisplayNode, DrawContext, NodeProps};
use octa_force::egui::{epaint::{CircleShape, TextShape}, Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2};
use petgraph::{csr::IndexType, EdgeType};

#[derive(Clone, Debug)]
pub struct NodeShape {
    pub pos: Pos2,

    pub selected: bool,
    pub dragged: bool,

    pub label_text: String,

    /// Shape defined property
    pub radius: f32,
}

impl<N: Clone> From<NodeProps<N>> for NodeShape {
    fn from(node_props: NodeProps<N>) -> Self {
        NodeShape {
            pos: node_props.location,
            selected: node_props.selected,
            dragged: node_props.dragged,
            label_text: node_props.label.to_string(),

            radius: 5.0,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix>
    for NodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        is_inside_circle(self.pos, self.radius, pos)
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        closest_point_on_circle(self.pos, self.radius, dir)
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);

        let is_interacted = self.selected || self.dragged;

        let style = if is_interacted {
            ctx.ctx.style().visuals.widgets.active
        } else {
            ctx.ctx.style().visuals.widgets.inactive
        };
        let color = style.fg_stroke.color;

        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_shape = CircleShape {
            center: circle_center,
            radius: circle_radius,
            fill: color,
            stroke: Stroke::default(),
        };
        res.push(circle_shape.into());

        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label_text.clone(),
                FontId::new(circle_radius, FontFamily::Monospace),
                Color32::WHITE,
            )
        });

        // display label centered over the circle
        let label_pos = Pos2::new(
            circle_center.x - galley.size().x / 2.,
            circle_center.y - circle_radius * 2.,
        );

        let label_shape = TextShape::new(label_pos, galley, Color32::WHITE);
        res.push(label_shape.into());

        res
    }

    fn update(&mut self, state: &NodeProps<N>) {
        self.pos = state.location;
        self.pos = state.location;
        self.selected = state.selected;
        self.dragged = state.dragged;
        self.label_text = state.label.to_string();
    }
}

fn closest_point_on_circle(center: Pos2, radius: f32, dir: Vec2) -> Pos2 {
    center + dir.normalized() * radius
}

fn is_inside_circle(center: Pos2, radius: f32, pos: Pos2) -> bool {
    let dir = pos - center;
    dir.length() <= radius
}
