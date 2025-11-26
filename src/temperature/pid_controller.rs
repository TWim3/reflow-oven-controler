use crate::temperature::pid_auto_tune::AutotuneState;
use defmt::info;
use pid::{ControlOutput, Pid};

pub const DEFAULT_P_GAIN: f32 = 1.6;
pub const DEFAULT_I_GAIN: f32 = 0.02;
pub const DEFAULT_D_GAIN: f32 = 0.1;

pub struct PidController {
    pid: Pid<f32>,
    output_limit: f32,
    autotune: bool,
    autotune_state: AutotuneState,
}

impl PidController {
    pub fn new(setpoint: f32, output_limit: f32, autotune: bool) -> Self {
        let mut pid = Pid::new(setpoint, output_limit);
        pid.p(DEFAULT_P_GAIN, output_limit)
            .i(DEFAULT_I_GAIN, output_limit)
            .d(DEFAULT_D_GAIN, output_limit);
        PidController {
            pid,
            output_limit,
            autotune,
            autotune_state: AutotuneState::new(),
        }
    }

    pub fn update_setpoint(&mut self, new_setpoint: f32) {
        self.pid
            .setpoint(new_setpoint)
            .p(DEFAULT_P_GAIN, self.output_limit)
            .i(DEFAULT_I_GAIN, self.output_limit)
            .d(DEFAULT_D_GAIN, self.output_limit);
    }

    pub fn update_gains(&mut self, p: f32, i: f32, d: f32) {
        self.pid
            .p(p, self.output_limit)
            .i(i, self.output_limit)
            .d(d, self.output_limit);
    }

    pub fn compute_control(&mut self, measurement: &f32) -> ControlOutput<f32> {
        if (self.autotune) {
            let error = self.pid.setpoint - measurement;
            let (p, i, d) = self.autotune_state.tune(error);
            self.update_gains(p, i, d);
            info!("Autotune adjusted gains -> P: {}, I: {}, D: {}", p, i, d);
        }

        self.pid.next_control_output(*measurement)
    }
}
