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


  adc.set_sample_time(adc::SampleTime::CYCLES239_5);

    loop {
        // Anzahl der Samples für Oversampling.
        const OVERSAMPLE_COUNT: u32 = 256;

        let mut acc: u32 = 0;

        for _ in 0..OVERSAMPLE_COUNT {
        //ADC-Wert lesen und aufaddieren
            acc += adc.read(&mut pin).await as u32;
        }

        // Oversampling-Skalierung:
        // Die Summe enthält 12 Bit (ADC) + 8 Bit (256 Samples) = 20 Bit Information.
        // Wir wollen effektiv 16 Bit behalten → 20 - 16 = 4 Bit wegschieben.
        // Daher: Rechts-Shiften um 4 Bit.
        let value: u16 = (acc >> 4) as u16;

        match calculate_temp(value) {
            Ok(temp) => info!("Temperature: {} °C Volt: {}", temp,value),
            Err(_) => error!("Error calculating temperature"),
        }
        Timer::after_millis(1000).await;
    }
}

fn calculate_temp(v: u16) -> Result<f32, Error> {
    let res = ((2.2_f32 * 1000.0) * v as f32 )/( 4096.0 - v as f32);
    calc_t(res , RTDType::PT1000)
}