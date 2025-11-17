#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use stm32f1xx_hal::{
    pac::{self, interrupt},
    prelude::*,
    timer::{Timer, Channel},
    gpio::gpioa::PA7,
    gpio::gpiob::PB1,
    gpio::{Input, PullUp},
};

static mut FLAG: bool = false;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // Clock setup
    let mut flash = dp.FLASH.constrain();
    let mut rcc   = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(72.mhz()).freeze(&mut flash.acr);

    // GPIO
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // PA7 Input
    let pa7: PA7<Input<PullUp>> = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);

    // PB1 → TIM3_CH4 PWM Pin
    let pb1 = gpiob.pb1.into_alternate_push_pull(&mut gpiob.crl);

    // -----------------------------
    // EXTI7 richtig konfigurieren
    // -----------------------------

    // AFIO für EXTI konfigurieren
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // PA7 → EXTI7 verbinden
    afio.exticr2.exticr2().modify(|_, w| w.exti7().pa7());

    // EXTI aktivieren (fallende Flanke)
    dp.EXTI.ftsr.modify(|_, w| w.tr7().set_bit()); // Falling trigger
    dp.EXTI.rtsr.modify(|_, w| w.tr7().clear_bit()); // Rising off
    dp.EXTI.imr.modify(|_, w| w.mr7().set_bit());   // Interrupt Mask Enable

    // Interrupt freigeben
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI9_5) };

    // -----------------------------
    // PWM auf PB1 (TIM3_CH4)
    // -----------------------------
    let mut pwm = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1)
        .pwm_hz(pb1, &mut afio.mapr, 1.khz());

    pwm.enable(Channel::C4);
    pwm.set_duty(Channel::C4, pwm.get_max_duty() / 2);

    loop {
        if unsafe { FLAG } {
            unsafe { FLAG = false };
            // z. B. Duty Cycle ändern
            pwm.set_duty(Channel::C4, pwm.get_max_duty() / 4);
        }
    }
}

#[interrupt]
fn EXTI9_5() {
    // EXTI7 pending Löschen
    let exti = unsafe { &*pac::EXTI::ptr() };
    exti.pr.write(|w| w.pr7().set_bit());

    unsafe {
        FLAG = true;
    }
}