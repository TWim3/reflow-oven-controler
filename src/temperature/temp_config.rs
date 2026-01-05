use defmt::info;
use embassy_time::Instant;

#[derive(PartialEq)]
pub enum CurveState {
    Preheat,
    HeatSoak,
    Reflow,
    Cooldown,
}

pub struct TempState {
    pub target_temp: f32,
    pub time_limit: u64,
    pub offset: f32,
    pub limit_output: f32,
}

impl Clone for TempState {
    fn clone(&self) -> Self {
        Self {
            target_temp: self.target_temp,
            offset: self.offset,
            time_limit: self.time_limit,
            limit_output: self.limit_output,
        }
    }
}

pub struct TempConfig {
    pub current_state: CurveState,

    timer: Instant,

    preheat: TempState,
    heat_soak: TempState,
    reflow: TempState,
    cooldown: TempState,
}

impl TempConfig {
    pub fn new(config: [TempState; 4]) -> Self {
        Self {
            preheat: config[0].clone(),
            heat_soak: config[1].clone(),
            reflow: config[2].clone(),
            cooldown: config[3].clone(),

            current_state: CurveState::Preheat,
            timer: Instant::now(),
        }
    }

    pub fn get_target_temp(&mut self, current_temp: f32) -> (f32, f32) {
        let state = self.get_current_state();

        if current_temp >= state.target_temp || (state.time_limit != 0 && self.timer.elapsed().as_secs() > state.time_limit) {
            self.advance_state();
            return self.get_target_temp(current_temp);
        }

        (state.target_temp - state.offset, state.limit_output)
    }

    pub fn reset(&mut self) {
        self.current_state = CurveState::Preheat;
    }

    fn advance_state(&mut self) {
        info!("Advancing to next state");
        self.timer = Instant::now();

        self.current_state = match self.current_state {
            CurveState::Preheat => CurveState::HeatSoak,
            CurveState::HeatSoak => CurveState::Reflow,
            CurveState::Reflow => CurveState::Cooldown,
            CurveState::Cooldown => CurveState::Cooldown, // Stay in cooldown
        };
    }

    pub fn get_current_state(&self) -> &TempState {
        match self.current_state {
            CurveState::Preheat => &self.preheat,
            CurveState::HeatSoak => &self.heat_soak,
            CurveState::Reflow => &self.reflow,
            CurveState::Cooldown => &self.cooldown,
        }
    }
}
