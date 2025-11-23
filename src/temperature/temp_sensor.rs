use embassy_stm32::Peri;
use embassy_stm32::adc::{Adc};
use embassy_stm32::peripherals::{ADC1, PA3};
use pt_rtd::{Error, RTDType, calc_t};

const OVERSAMPLE_COUNT: u32 = 256;

pub struct TempSensor<'a> {
    adc: Adc<'a, ADC1>,
    pin: Peri<'a, PA3>,
}

impl<'a> TempSensor<'a> {
    pub fn new(adc: Adc<'a, ADC1>, pin: Peri<'a, PA3>) -> Self {
        Self { adc, pin }
    }

    pub async fn read_temperature(&mut self) -> Result<f32, Error> {
        let mut acc: u32 = 0;

        for _ in 0..OVERSAMPLE_COUNT {
            acc += self.adc.read(&mut self.pin).await as u32;
        }

        let value: u16 = (acc >> 4) as u16;

        Self::calculate_temp(value)
    }

    fn calculate_temp(v: u16) -> Result<f32, Error> {
        let res = ((2.2_f32 * 1000.0) * v as f32) / (65526.0 - v as f32);
        calc_t(res, RTDType::PT1000)
    }


}
