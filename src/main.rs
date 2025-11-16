#![no_std]
#![no_main]

mod temp_sensor;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::{ADC1};
use embassy_stm32::{adc, bind_interrupts};
use embassy_time::{Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Initialize temperature sensor
    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(adc::SampleTime::CYCLES239_5);
    let mut temp_sensor = temp_sensor::TempSensor::new(adc, p.PA3);

    let start_button = Input::new(p.PB0, Pull::None); // TODO: Change to real pin
    let stop_button = Input::new(p.PB1, Pull::None);  // TODO: Change to real pin

    let mut should_run = false;

    loop {
        if start_button.is_high() {
            Timer::after_millis(50).await;
            info!("Starting oven task...");
            should_run = true;
        }

        if stop_button.is_high() {
            Timer::after_millis(50).await;
            info!("Stopping oven task...");
            should_run = false;
        }

        if !should_run {
            Timer::after_millis(100).await;
            continue;
        }

        match temp_sensor.read_temperature().await {
            Ok(temp) => info!("Temperature: {} °C", temp),
            Err(_) => error!("Error calculating temperature"),
        }

        Timer::after_millis(500).await;
    }
}
