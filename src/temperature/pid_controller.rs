use pid::{ControlOutput, Pid};

pub const DEFAULT_P_GAIN: f32 = 5.0;
pub const DEFAULT_I_GAIN: f32 = 1.0;
pub const DEFAULT_D_GAIN: f32 = 2.0;

pub struct PidController {
    pid: Pid<f32>,
    output_limit: f32,
}

impl PidController {
    pub fn new(setpoint: f32, output_limit: f32) -> Self {
        let mut pid = Pid::new(setpoint, output_limit);
        pid.p(DEFAULT_P_GAIN, output_limit)
            .i(DEFAULT_I_GAIN, output_limit)
            .d(DEFAULT_D_GAIN, output_limit);
        PidController { pid, output_limit }
    }

    pub fn update_setpoint(&mut self, new_setpoint: f32) {
        self.pid
            .setpoint(new_setpoint)
            .p(DEFAULT_P_GAIN, self.output_limit)
            .i(DEFAULT_I_GAIN, self.output_limit)
            .d(DEFAULT_D_GAIN, self.output_limit);
    }

    pub fn update_gains(&mut self, p: f32, i: f32, d: f32) {
        let limit = self.pid.output_limit;
        self.pid.p(p, limit).i(i, limit).d(d, limit);
    }

    pub fn compute_control(&mut self, measurement: &f32) -> ControlOutput<f32> {
        self.pid.next_control_output(*measurement)
    }
}
