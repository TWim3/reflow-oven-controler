#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Input, Output, Level, Pull, Speed, OutputType};
use embassy_stm32::init;

// 100 Halbwellen pro Sekunde bei 50 Hz Netz
const HALFWAVES_PER_SECOND: u8 = 100;

// Fester Duty Cycle in Prozent
const DUTY_CYCLE_PERCENT: u8 = 50;

// Gate-Impulsdauer für Triac (µs)
// Muss lang genug sein, damit der Triac sicher zündet
const TRIAC_PULSE_US: u64 = 150;

fn percent_to_halfwave_count(percent: u8) -> u8 {
    let clamped = percent.min(100);
    ((clamped as u16 * HALFWAVES_PER_SECOND as u16) / 100) as u8
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init(Default::default());
    info!("Zero-cross gesteuerte Halbwellen-Triacsteuerung startet");

    // PA7 als Zero-Crossing-Input
    let mut zero_cross = Input::new(p.PA7, Pull::Up);

    // PB1 als Triac-Gate-Output
    let mut triac_gate = Output::new(
        p.PB1,
        Level::Low,           // initial aus
        Speed::VeryHigh,
        OutputType::PushPull,
    );

    let duty_percent = DUTY_CYCLE_PERCENT.min(100);
    let duty_halfwaves = percent_to_halfwave_count(duty_percent);

    info!("Duty Cycle: {}%", duty_percent);

    // Halbwellen-Zähler
    let mut halfwave_idx: u8 = 0;

    loop {
        // Auf fallende Flanke des Zero-Cross-Eingangs warten
        //
        // -> hier wird intern EXTI genutzt, Pulsbreite (6 µs) ist egal:
        //    die Flanke wird gelatched.
        zero_cross.wait_for_falling_edge().await;

        // Neue Halbwelle beginnt
        halfwave_idx = (halfwave_idx + 1) % HALFWAVES_PER_SECOND;

        // Entscheiden, ob diese Halbwelle "an" oder "aus" ist
        if halfwave_idx < duty_halfwaves {
            // Diese Halbwelle ist "an":
            // -> direkt am Beginn (also jetzt) Triac zünden
            triac_gate.set_high();
            Timer::after_micros(TRIAC_PULSE_US).await;
            triac_gate.set_low();
            // Danach bleibt der Triac bis zum nächsten Nulldurchgang leitend
        } else {
            // Diese Halbwelle ist "aus": Triac wird nicht gezündet
            triac_gate.set_low();
        }
    }
}
