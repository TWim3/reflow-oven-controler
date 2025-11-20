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

// Gate-Impulsdauer für Triac (µs)
// Muss lang genug sein, damit der Triac sicher zündet
const TRIAC_PULSE_US: u64 = 150;

// Sollwert z.B. Temperatur
const SETPOINT: f32 = 180.0;

// PID
struct PID {
    kp: f32,
    ki: f32,
    kd: f32,
    integral: f32,
    prev_error: f32,
    out_min: f32,
    out_max: f32,
}

impl PID {
    fn new(kp: f32, ki: f32, kd: f32, out_min: f32, out_max: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: 0.0,
            out_min,
            out_max,
        }
    }

    fn update(&mut self, setpoint: f32, pv: f32, dt: f32) -> f32 {
        let error = setpoint - pv;
        let p = self.kp * error;

        self.integral += error * dt;
        if self.integral > 1000.0 {
            self.integral = 1000.0;
        } else if self.integral < -1000.0 {
            self.integral = -1000.0;
        }
        let i = self.ki * self.integral;

        let d = self.kd * (error - self.prev_error) / dt;
        self.prev_error = error;

        let mut out = p + i + d;
        if out > self.out_max {
            out = self.out_max;
        } else if out < self.out_min {
            out = self.out_min;
        }
        out
    }
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

    // PID initialisieren; dt ≈ 1/100 s (100 Hz Halbwellen)
    let mut pid = PID::new(2.0, 0.5, 0.1, 0.0, 100.0);
    let dt: f32 = 1.0 / (HALFWAVES_PER_SECOND as f32);

    // Halbwellen-Zähler und Duty aus PID
    let mut halfwave_idx: u8 = 0;
    let mut duty: u8 = 0;

    loop {
        // Auf fallende Flanke des Zero-Cross-Eingangs warten
        //
        // -> hier wird intern EXTI genutzt, Pulsbreite (6 µs) ist egal:
        //    die Flanke wird gelatched.
        zero_cross.wait_for_falling_edge().await;

        // Neue Halbwelle beginnt
        halfwave_idx = (halfwave_idx + 1) % HALFWAVES_PER_SECOND;

        // Einmal pro "Fenster" von 100 Halbwellen den PID updaten
        if halfwave_idx == 0 {
            let pv = read_process_value().await; // z.B. aktuelle Temperatur
            let pid_out = pid.update(SETPOINT, pv, dt);

            let pid_clamped = pid_out.clamp(0.0, 100.0);
            duty = pid_clamped as u8;

            info!("PV={} SP={} PID={} Duty={}", pv, SETPOINT, pid_out, duty);
        }

        // Entscheiden, ob diese Halbwelle "an" oder "aus" ist
        if halfwave_idx < duty {
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

// Dummy: hier musst du deinen echten Istwert (z.B. Temperatur) zurückgeben
async fn read_process_value() -> f32 {
    // TODO: ADC lesen, PT1000 berechnen, etc.
    150.0
}