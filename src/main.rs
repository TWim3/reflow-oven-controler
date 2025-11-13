#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::{adc, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use pt_rtd::*;

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);
    let mut pin = p.PA3;


    adc.set_sample_time(adc::SampleTime::CYCLES41_5);

    loop {
        let v = adc.read(&mut pin).await;

        match calculate_temp(v) {
            Ok(temp) => info!("Temperature: {} °C", temp),
            Err(_) => error!("Error calculating temperature"),
        }
        Timer::after_millis(1000).await;
    }
}

fn calculate_temp(v: u16) -> Result<f32, Error> {
    let res = (2.2_f32 * 1000.0) * ((v as f32 / 4096.0) - v as f32);
    calc_t(res , RTDType::PT1000)
}