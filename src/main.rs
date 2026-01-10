#![no_std]
#![no_main]

mod temperature;

mod output;

use crate::output::SignalOutput;
use crate::temperature::pid_controller::PidController;
use crate::temperature::temp_config::{CurveState, TempConfig, TempState};
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

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

/*
Step by step todo
- start via start button
- 1.
    - rise until temp x

    - next step when temp is reached
- 2.
    - start timer
    - rise until temp x+1
    - only with max duty cycle 50%

    - next step when time is t or temp is reached
- 3.
    - rise until temp x+2
    - no limit on duty cycle

    - next step when temp is reached
- 4.
    - turn off oven
    - keep temp output until less then x+3

    - next step when temp is reached

- 5.
    - reset everything
 */

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //********************************** Setup *********************************
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

    // Output
    let zero_cross = ExtiInput::new(p.PA7, p.EXTI7, Pull::Up);
    let triac_gate = Output::new(p.PB1, Level::Low, Speed::VeryHigh);
    let mut signal_output = SignalOutput::new(zero_cross, triac_gate);
    //**************************************************************************

    // PID Controller and Timer
    let mut temp_curve = TempConfig::new([
        TempState {
            target_temp: 130.0,
            time_limit: 0,
            offset: 15.0,
            limit_output: 100.0,
        },
        TempState {
            target_temp: 190.0,
            time_limit: 140,
            offset: 0.0,
            limit_output: 55.0,
        },
        TempState {
            target_temp: 245.0,
            time_limit: 0,
            offset: 0.0,
            limit_output: 100.0,
        },
        TempState {
            target_temp: 80.0,
            time_limit: 0,
            offset: 0.0,
            limit_output: 0.0,
        },
    ]);
    let mut pid_controller = PidController::new(0.0, 100.0);

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
            temp_curve.reset();
            pid_controller.reset();
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

        let curve_value = temp_curve.get_target_temp(temp);
        info!(
            "Target temperature: {} C, Output limit: {}%",
            curve_value.0, curve_value.1
        );

        if temp_curve.current_state == CurveState::Cooldown && temp <= curve_value.0 {
            info!("Cooldown complete. Stopping oven.");
            should_run = false;
            temp_curve.reset();
            signal_output.output_signal(0).await;
            pid_controller.reset();
            continue;
        }

        pid_controller.update_setpoint(curve_value.0);
        let pid_output = pid_controller
            .compute_control(&temp)
            .output
            .clamp(0.0, curve_value.1);

        info!("Computed pid: {}", pid_output);

        signal_output.output_signal(pid_output as u32).await;
    }
}
