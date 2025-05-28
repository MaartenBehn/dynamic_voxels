use octa_force::glam::Vec3;

#[derive(Debug, Clone)]
pub enum PositionSetRule {
    Grid { spacing: f32 },
    Possion { distance: f32 },
}

#[derive(Debug, Clone)]
pub struct PositionSet {
    radius: f32,
    center: Vec3,
    rule: PositionSetRule, 
}

impl PositionSet {
    pub fn new(radius: f32, center: Vec3, rule: PositionSetRule) -> Self {
        Self { radius, center, rule }
    }
}


