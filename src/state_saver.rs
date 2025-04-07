use octa_force::{egui::{self, panel::Side, Align, Frame, Id, Layout, Widget}, puffin_egui::puffin, OctaResult};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TickType {
    None,
    Back,
    Forward,
    ForwardSave,
}

pub struct StateSaver<S> {
    start_state: S,
    states: Vec<S>,
    current: usize,
    length: usize,

    next_tick: TickType,

    ticks_per_frame: usize,
    run: bool,
}

pub trait State: Clone {
    fn tick_state(&mut self) -> OctaResult<bool>;
}

impl<S: State> StateSaver<S> {
    pub fn from_state(history: S, num_saved: usize) -> Self {
        StateSaver {
            start_state: history.clone(),
            states: vec![history],
            current: 0,
            length: num_saved,
            next_tick: TickType::None,
            ticks_per_frame: 1,
            run: false,
        }
    }

    pub fn tick(&mut self) -> OctaResult<bool> {
        puffin::profile_function!();
        
        match self.next_tick {
            TickType::None => {
                Ok(false)
            }
            TickType::Back => {
                self.tick_back();
                self.set_next_tick(TickType::None);
                Ok(true)
            }
            TickType::Forward => {
                self.tick_forward(false)?;
                self.set_next_tick(TickType::None);
                Ok(true)
            }
            TickType::ForwardSave => {
                self.tick_forward(true)?;
                self.set_next_tick(TickType::None);
                Ok(true)
            }
        }
    }

    fn tick_forward(&mut self, save_tick: bool) -> OctaResult<()> {
        puffin::profile_function!();
        
        if self.current == 0 {
            let mut new_state;
            {
                puffin::profile_scope!("Clone state");
                new_state = self.states[0].clone();
            }
            
            let changed = new_state.tick_state()?;
            
            if !changed || !save_tick {
                self.states[0] = new_state;
            } else {
                puffin::profile_scope!("Update state list");
                self.states.insert(0, new_state);
                self.states.truncate(self.length);
            }
            
            return Ok(());
        }

        self.current -= 1;

        Ok(())
    }

    fn tick_back(&mut self) {
        if self.current >= self.states.len() - 1 {
            return;
        }

        self.current += 1;
    }

    pub fn get_state(&self) -> &S {
        &self.states[self.current]
    }

    pub fn get_state_mut(&mut self) -> &mut S {
        &mut self.states[self.current]
    }

    pub fn get_step_state(&self) -> (usize, usize) {
        (self.current, self.length)
    }

    pub fn reset(&mut self) {
        self.states = vec![self.start_state.clone()];
        self.current = 0;
    }
    
    pub fn set_next_tick(&mut self, next_tick: TickType) {
        self.next_tick = next_tick;
    }

    pub fn render(&mut self, ctx: &egui::Context) {

        egui::SidePanel::new(Side::Left, Id::new("Side Panel")).show(ctx, |ui| {
            puffin::profile_scope!("Left Panel");

            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                div(ui, |ui| {
                    ui.heading("Depth Tree Grid (Debug Mode)");
                });


                div(ui, |ui| {
                    ui.label("Tick: ");

                    if ui.button("<<<").clicked() {
                        self.run = false;
                        self.set_next_tick(TickType::Back);
                    }

                    if ui.button(">>>").clicked() {
                        self.run = false;
                        self.set_next_tick(TickType::ForwardSave);
                    }
                });

                div(ui, |ui| {
                    ui.label("saved ticks:");
                });

                div(ui, |ui| {
                    
                    let (current_saved, num_saved) = self.get_step_state();
                    egui::ProgressBar::new(1.0 - (current_saved as f32 / num_saved as f32)).ui(ui);
                    
                     
                });

                div(ui, |ui| {
                    ui.checkbox(&mut self.run, "run");
                    
                    ui.label("Ticks per frame: ");

                    ui.add(
                        egui::DragValue::new(&mut self.ticks_per_frame)
                            .speed(1)
                            .range(1..=100),
                    );
                    
                });

                
                div(ui, |ui| {
                    if ui.button("clear").clicked() {
                        self.run = false;
                        self.reset()
                    }
                });

            });
        });
    }
}

fn div(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none().show(ui, |ui| {
        ui.with_layout(Layout::left_to_right(Align::TOP), add_contents);
    });
}
