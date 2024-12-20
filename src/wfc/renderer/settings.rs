pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            zoom_speed: 0.1,
            fit_to_screen_enabled: true,
            zoom_and_pan_enabled: false,
        }
    }
}



pub struct ForceSettings {
    pub dt: f32,
    pub cooloff_factor: f32,
    pub scale: f32,
}

impl Default for ForceSettings {
    fn default() -> Self {
        Self {
            dt: 0.03,
            cooloff_factor: 0.80,
            scale: 400.,
        }
    }
}

pub struct SimulationSettings {
    pub center_force: f32,
    pub running: bool,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self { 
            center_force: 0.01, 
            running: true,
        }
    }
}
