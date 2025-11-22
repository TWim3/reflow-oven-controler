use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use embassy_time::Timer;

// 100 Halbwellen pro Sekunde bei 50 Hz Netz
const HALFWAVES_PER_SECOND: u8 = 100;

// Fester Duty Cycle in Prozent
const DUTY_CYCLE_PERCENT: u8 = 50;

// Gate-Impulsdauer für Triac (µs)
const TRIAC_PULSE_US: u64 = 2000;

fn percent_to_halfwave_count(percent: u8) -> u8 {
    let clamped = percent.min(100);
    ((clamped as u16 * HALFWAVES_PER_SECOND as u16) / 100) as u8
}

pub async fn run_test(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Zero-cross gesteuerte Halbwellen-Triacsteuerung startet");

    let mut zero_cross = ExtiInput::new(p.PA7, p.EXTI7, Pull::Up);
    let mut triac_gate = Output::new(p.PB1, Level::Low, Speed::VeryHigh);

    let duty_percent = DUTY_CYCLE_PERCENT.min(100);
    let duty_halfwaves = percent_to_halfwave_count(duty_percent);

    info!("Duty Cycle: {}%", duty_percent);

    let mut halfwave_idx: u8 = 0;

    loop {
        zero_cross.wait_for_rising_edge().await;

        halfwave_idx = (halfwave_idx + 1) % HALFWAVES_PER_SECOND;

        if halfwave_idx < duty_halfwaves {
            triac_gate.set_high();
            Timer::after_micros(TRIAC_PULSE_US).await;
            triac_gate.set_low();
        } else {
            triac_gate.set_low();
        }
    }
}
