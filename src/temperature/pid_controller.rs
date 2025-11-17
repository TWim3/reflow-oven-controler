use pid::{ControlOutput, Pid};

const P: f32 = 5.0;
const I: f32 = 1.0;
const D: f32 = 2.0;

pub struct PidController {
    pid: Pid<f32>,
}

impl PidController {
    pub fn new(setpoint: f32, output_limit: f32) -> Self {
        let mut pid = Pid::new(setpoint, output_limit);
        pid.p(P, output_limit).i(I, output_limit).d(D, output_limit);
        PidController { pid }
    }

    pub fn update_setpoint(&mut self, new_setpoint: f32) {
        self.pid.setpoint(new_setpoint);
    }

    pub fn compute_control(&mut self, measurement: &f32) -> ControlOutput<f32> {
        self.pid.next_control_output(*measurement)
    }
}
