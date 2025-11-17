use defmt::info;
use embassy_stm32::{bind_interrupts, peripherals, timer, Peri};
use embassy_stm32::gpio::{Pull};
use embassy_stm32::peripherals::{PA7, TIM3};
use embassy_stm32::time::khz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_time::Instant;

bind_interrupts!(struct Irqs {
    TIM3 => timer::CaptureCompareInterruptHandler<peripherals::TIM3>;
});

#[embassy_executor::task]
pub async fn pwm_test(signal_input: Peri<'static, PA7>, tim3: Peri<'static, TIM3>) {
    // This is a placeholder for PWM test code.
    // Implement PWM functionality tests here.

    let ch2 = CapturePin::new(signal_input, Pull::None);
    let ic = InputCapture::new(
        tim3,
        None,
        Some(ch2),
        None,
        None,
        Irqs,
        khz(1000),
        Default::default(),
    );

    let timer = Instant::now();

    loop {
        if(ic.get_input_interrupt(Channel::Ch2)) {
            info!("Interrupt received at {:?}", timer.as_millis());
        }
    }
}

// mod SignalCatcher {
//
// }
