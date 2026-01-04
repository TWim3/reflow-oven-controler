#![no_std]
#![no_main]

mod oven_timer;
mod temperature;

mod output;

use crate::output::SignalOutput;
use crate::oven_timer::OvenTimer;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{self, Adc};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::rcc::{Hse, HseMode, Sysclk};
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts};
use embassy_time::Timer;
use temperature::temp_sensor::TempSensor;
use {defmt_rtt as _, panic_probe as _};
use crate::temperature::temp_curve::TempCurve;

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hse = Some(Hse {
        freq: mhz(16),
        mode: HseMode::Oscillator,
    });
    config.rcc.sys = Sysclk::HSE;
    let p = embassy_stm32::init(config);

    // Temperature sensor
    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(adc::SampleTime::CYCLES239_5);
    let mut temp_sensor = TempSensor::new(adc, p.PA3);

    // Input
    let start_button = Input::new(p.PA8, Pull::Down);
    let stop_button = Input::new(p.PB14, Pull::Up);
    let mut should_run = false;

    // PID Controller and Timer
    let mut temp_curve = TempCurve::new(6, [(1, 150.0, 20.0), (10, 180.0, 0.0), (60, 200.0, 0.0), (130, 255.0, 10.0), (190, 60.0, 0.0), (600, 30.0, 0.0)]);
    let mut timer = OvenTimer::new();

    // Output
    let zero_cross = ExtiInput::new(p.PA7, p.EXTI7, Pull::Up);

    let triac_gate = Output::new(p.PB1, Level::Low, Speed::VeryHigh);
    let mut signal_output = SignalOutput::new(zero_cross, triac_gate);

    loop {
        if start_button.is_high() && !should_run {
            Timer::after_millis(50).await;
            info!("Starting oven task...");
            should_run = true;
        }

        if stop_button.is_low() && should_run {
            Timer::after_millis(50).await;
            info!("Stopping oven task...");

            should_run = false;
            timer.clear();
            //TODO reset pid curve
        }

        if !should_run {
            Timer::after_millis(100).await;
            signal_output.output_signal(0).await;
            continue;
        }

        let temp = match temp_sensor.read_temperature().await {
            Ok(temp) => temp,
            Err(_) => {
                error!("Error reading temperature");
                continue;
            }
        };
        let t10 = (temp * 10.0) as i32;
        info!("Current temperature: {}.{} C", t10 / 10, (t10 % 10).abs());

        const TEMP_BEFORE_TIMER_START: f32 = 150.0;

        let elapsed = match temp {
            t if t < TEMP_BEFORE_TIMER_START => 0,
            _ => timer.elapsed_secs(),
        };

        let pid_output = temp_curve.compute_control(&elapsed, &temp);

        let pid_output = match pid_output {
            Some(output) => output,
            None => {
                info!("Temperature curve complete.");
                should_run = false;
                timer.clear();
                0.0
            }
        };

        info!("Computed pid: {}", pid_output);

        signal_output.output_signal(pid_output as u32).await;
    }
}
