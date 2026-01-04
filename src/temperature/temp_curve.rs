use defmt::info;
use crate::temperature::pid_controller::PidController;

const MAX_POINTS: usize = 6;

pub struct TempCurve {
    length: u8,
    points: [(u32, f32, f32); MAX_POINTS],
    pid_controller: PidController
}

impl TempCurve {
    pub fn new(length: u8, points: [(u32, f32, f32); MAX_POINTS]) -> Self {
        Self { length, points, pid_controller: PidController::new(0.0, 100.0, false) }
    }

    pub fn compute_control(&mut self, elapsed_secs: &u64, current_temp: &f32) -> Option<f32> {
        let target_temp = self.get_target_temperature(elapsed_secs, *current_temp);
        if(target_temp == 0.0) {
            return None;
        }
        info!("Target temperature: {} C", target_temp);
        self.pid_controller.update_setpoint(target_temp);
        Some(self.pid_controller.compute_control(current_temp).output)
    }

    fn get_target_temperature(&self, elapsed_secs: &u64, current_temp: f32) -> f32 {
        let elapsed = *elapsed_secs as u32;
        for i in 0..self.length as usize {
            if elapsed < self.points[i].0 && current_temp >= self.points[i].1 {
                return self.points[i].1;
            }

            if elapsed < self.points[i].0 {
                return self.points[i].1 - self.points[i].2;
            }

        }
        // If elapsed time exceeds all points, return the last temperature
        0.0
    }
}
