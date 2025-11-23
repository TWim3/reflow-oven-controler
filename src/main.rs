#![no_std]
#![no_main]

mod oven_timer;
mod pid_test;
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
use embassy_stm32::{Peripherals, bind_interrupts};
use embassy_time::Timer;
use temperature::temp_sensor::TempSensor;
use {defmt_rtt as _, panic_probe as _};

const ENABLE_OVEN_CONTROLLER: bool = true;
const ENABLE_TEST: bool = true;
const ENABLE_PID_TEST: bool = true;

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = Some(embassy_stm32::init(Default::default()));

    if ENABLE_OVEN_CONTROLLER {
        run_oven_controller(peripherals.take().unwrap()).await;
    } else if ENABLE_PID_TEST {
        run_pid_test(peripherals.take().unwrap()).await;
    } else if ENABLE_TEST {
        // Test-Einstiegspunkt in test.rs, z.B.:
        // pub async fn run_test(spawner: Spawner, p: Peripherals) -> ! { ... }
        test::run_test(spawner, peripherals.take().unwrap()).await;
    } else {
        let _ = peripherals.take();
        info!(
            "Main idle. Set ENABLE_OVEN_CONTROLLER=true für den Controller, ENABLE_PID_TEST=true für den PID-Test oder ENABLE_TEST=true für die Tests."
        );
        idle_loop().await;
    }
}

async fn run_oven_controller(p: Peripherals) -> ! {
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

async fn run_pid_test(p: Peripherals) -> ! {
    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(adc::SampleTime::CYCLES239_5);
    let mut temp_sensor = TempSensor::new(adc, p.PA3);

    loop {
        match temp_sensor.read_temperature().await {
            Ok(temp) => {
                let output = pid_test::pid_output_for_duty_cycle(temp);
                info!(
                    "PID test: temperature {} °C -> duty output {}%",
                    temp, output
                );
            }
            Err(_) => error!("PID test: failed to read temperature"),
        }

        Timer::after_millis(500).await;
    }
}

async fn idle_loop() -> ! {
    loop {
        Timer::after_millis(1000).await;
    }
}
