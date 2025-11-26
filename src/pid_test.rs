use crate::temperature::pid_controller::{
    PidController, DEFAULT_D_GAIN, DEFAULT_I_GAIN, DEFAULT_P_GAIN,
};
use crate::test::DUTY_CYCLE_PERCENT;
use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};
use defmt::info;

// PID-Autotune-Parameter in Deutsch:
// OUTPUT_LIMIT begrenzt die Stellgroesse (hoeher = schneller aufheizen, niedriger = schonender).
// ENABLE_AUTOTUNE steuert die Selbstjustage; auf false schalten, wenn feste Werte gemessen wurden.
// ERROR_BAND, STABLE_THRESHOLD definieren, wie lange der Fehler ruhig sein muss, bevor I erhoeht wird.
// KP_STEP, KI_STEP, KD_STEP und die jeweiligen MIN/MAX-Werte bestimmen, wie aggressiv P/I/D nachgestellt werden.
// DERIV_GAIN verstaerkt die Reaktion auf steile Fehleraenderungen; hoeher = mehr Daempfung bei schnellen Spruengen.
const OUTPUT_LIMIT: f32 = 100.0;
const ENABLE_AUTOTUNE: bool = false;
const ERROR_BAND: f32 = 0.5;
const KP_STEP: f32 = 0.25;
const KI_STEP: f32 = 0.02;
const KD_STEP: f32 = 0.05;
const KP_MIN: f32 = 0.1;
const KP_MAX: f32 = 50.0;
const KI_MAX: f32 = 5.0;
const KD_MAX: f32 = 15.0;
const DERIV_GAIN: f32 = 0.02;
const STABLE_THRESHOLD: u16 = 8;

const KP_INITIAL: f32 = 10.0;   // Startwert P-Gain
const KI_INITIAL: f32 = 0.015;  // Startwert I-Gain (1/s)
const KD_INITIAL: f32 = 0.0;    // Startwert D-Gain

struct AutotuneState {
    kp: f32,
    ki: f32,
    kd: f32,
    prev_error: f32,
    stable_cycles: u16,
}

impl AutotuneState {
    const fn new() -> Self {
        Self {
            kp: DEFAULT_P_GAIN,
            ki: DEFAULT_I_GAIN,
            kd: DEFAULT_D_GAIN,
            prev_error: 0.0,
            stable_cycles: 0,
        }
    }

    fn reset(&mut self) {
        self.kp = DEFAULT_P_GAIN;
        self.ki = DEFAULT_I_GAIN;
        self.kd = DEFAULT_D_GAIN;
        self.prev_error = 0.0;
        self.stable_cycles = 0;
    }

    fn tune(&mut self, error: f32) -> (f32, f32, f32) {
        let sign_flip = self.prev_error != 0.0
            && (self.prev_error.is_sign_positive() != error.is_sign_positive());

        if sign_flip {
            // Vorzeichenwechsel: P leicht drosseln, D/I erhoehen, um Ueberschwingen zu mindern.
            self.kp = (self.kp * 0.85).max(KP_MIN);
            self.kd = (self.kd + KD_STEP).min(KD_MAX);
            self.ki = (self.ki * 0.9).max(0.0);
            self.stable_cycles = 0;
        } else {
            if error.abs() > ERROR_BAND {
                // Grosser Fehler: P schrittweise anheben fuer schnelleres Regeln.
                self.kp = (self.kp + KP_STEP).min(KP_MAX);
                self.stable_cycles = 0;
            } else {
                self.stable_cycles = self.stable_cycles.saturating_add(1);
                if self.stable_cycles > STABLE_THRESHOLD {
                    // Fehler lange stabil: I erhoehen, damit kleiner Offset ausgeglichen wird.
                    self.ki = (self.ki + KI_STEP).min(KI_MAX);
                }
            }

            let slope = (error - self.prev_error).abs();
            if slope > ERROR_BAND {
                // Starke Fehleraenderung: D anheben, um schnelle Spruenge zu daempfen.
                self.kd = (self.kd + slope * DERIV_GAIN).min(KD_MAX);
            }
        }

        self.prev_error = error;
        (self.kp, self.ki, self.kd)
    }

    fn gains(&self) -> (f32, f32, f32) {
        (self.kp, self.ki, self.kd)
    }

    fn is_customized(&self) -> bool {
        (self.kp - DEFAULT_P_GAIN).abs() > f32::EPSILON
            || (self.ki - DEFAULT_I_GAIN).abs() > f32::EPSILON
            || (self.kd - DEFAULT_D_GAIN).abs() > f32::EPSILON
    }
}

static CONTROLLER: Mutex<RefCell<Option<PidController>>> = Mutex::new(RefCell::new(None));
static AUTOTUNE_STATE: Mutex<RefCell<AutotuneState>> = Mutex::new(RefCell::new(AutotuneState::new()));

pub fn pid_output_for_duty_cycle(measured_value: f32) -> u8 {
    let setpoint = DUTY_CYCLE_PERCENT as f32;

    let control = interrupt::free(|cs| {
        let mut controller_ref = CONTROLLER.borrow(cs).borrow_mut();
        let controller = controller_ref
            .get_or_insert_with(|| {
                let mut c = PidController::new(setpoint, OUTPUT_LIMIT);
                // Startwerte fuer den PID-Regler setzen (aus der vorherigen Empfehlung)
                c.update_gains(KP_INITIAL, KI_INITIAL, KD_INITIAL);
                c
            });

        let mut state = AUTOTUNE_STATE.borrow(cs).borrow_mut();
        if ENABLE_AUTOTUNE {
            // Autotune passt P/I/D auf Basis des aktuellen Fehlers laufend an.
            let error = setpoint - measured_value;
            let (p, i, d) = state.tune(error);
            controller.update_gains(p, i, d);
        } else if state.is_customized() {
            // Bei ausgeschaltetem Autotune zurueck auf die Default-Gains, falls sie zuvor veraendert wurden.
            state.reset();
            let (p, i, d) = state.gains();
            controller.update_gains(p, i, d);
        }

        controller.compute_control(&measured_value)
    });
    let duty_output = control.output.clamp(0.0, OUTPUT_LIMIT) as u8;

    info!(
        "PID duty cycle -> target {}%, measured {}%, p {}, i {}, d {}, output {}%",
        DUTY_CYCLE_PERCENT,
        measured_value,
        control.p,
        control.i,
        control.d,
        duty_output
    );

    duty_output
}
