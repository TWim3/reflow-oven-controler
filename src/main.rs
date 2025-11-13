#![no_std]
#![no_main]

mod fmt;

#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc};
use embassy_time::{Timer};
use fmt::info;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

        let mut temp_adc = Adc::new(p.ADC1);
        let mut temp_pin = p.PA3;

    loop {
        let measurement = temp_adc.read(&mut temp_pin).await;
        info!("temperature: {}", measurement);
        Timer::after_millis(1000).await;
    }
}
