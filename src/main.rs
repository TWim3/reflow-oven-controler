#![no_std]
#![no_main]

mod oven_timer;
mod temperature;
mod test;

use crate::oven_timer::OvenTimer;
use crate::temperature::pid_controller::PidController;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{self, Adc};
#[allow(unused_imports)]
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::{bind_interrupts};
use embassy_time::Timer;
use temperature::temp_sensor::TempSensor;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(adc::SampleTime::CYCLES239_5);
    let mut temp_sensor = TempSensor::new(adc, p.PA3);

    let start_button = Input::new(p.PA8, Pull::Down);
    let stop_button = Input::new(p.PB14, Pull::Up);

    let mut should_run = false;

    let mut timer = OvenTimer::new();
    let mut pid_controller = PidController::new(1.0, 100.0);

    loop {
        if start_button.is_high() {
            Timer::after_millis(50).await;
            info!("Starting oven task...");
            should_run = true;
        }

        if stop_button.is_low() {
            Timer::after_millis(50).await;
            info!("Stopping oven task...");

            timer.clear();
            should_run = false;
        }

        if !should_run {
            Timer::after_millis(100).await;
            continue;
        }

        let temp = match temp_sensor.read_temperature().await {
            Ok(temp) => temp,
            Err(_) => {
                error!("Error reading temperature");
                continue;
            }
        };

        let _elapsed = timer.elapsed_secs();
        let _pid = pid_controller.compute_control(&temp);

        Timer::after_millis(500).await;
    }
}
