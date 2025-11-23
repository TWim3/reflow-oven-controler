use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_time::Timer;

const HALFWAVES_PER_SECOND: u32 = 100;

const TRIAC_PULSE_US: u32 = 2000;

pub struct SignalOutput {
    zero_cross: ExtiInput<'static>,
    triac_gate: Output<'static>,
}

impl SignalOutput {
    pub fn new(zero_cross: ExtiInput<'static>, triac_gate: Output<'static>) -> Self {
        SignalOutput {
            zero_cross,
            triac_gate,
        }
    }

    pub async fn output_signal(&mut self, mut halfwave_idx: u32, duty_percent: u32) {
        self.zero_cross.wait_for_rising_edge().await;

        halfwave_idx = (halfwave_idx + 1) % HALFWAVES_PER_SECOND;

        if halfwave_idx < percent_to_halfwave_count(duty_percent) {
            self.triac_gate.set_high();
            Timer::after_micros(TRIAC_PULSE_US as u64).await;
            self.triac_gate.set_low();
        } else {
            self.triac_gate.set_low();
        }
    }
}

fn percent_to_halfwave_count(percent: u32) -> u32 {
    let clamped = percent.min(100);
    clamped * HALFWAVES_PER_SECOND / 100
}
