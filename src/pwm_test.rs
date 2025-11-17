use defmt::info;
use embassy_stm32::gpio::{OutputType, Pull};
use embassy_stm32::peripherals::{PA1, PA7, TIM2, TIM3};
use embassy_stm32::time::{hz, khz};
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{Peri, bind_interrupts, peripherals, timer};
use embassy_time::Timer;
use embedded_hal::Pwm;

bind_interrupts!(struct Irqs {
    TIM3 => timer::CaptureCompareInterruptHandler<peripherals::TIM3>;
});

#[embassy_executor::task]
pub async fn pwm_test(
    signal_input: Peri<'static, PA7>,
    tim3: Peri<'static, TIM3>,
    signal_output: Peri<'static, PA1>,
    pwm_timer: Peri<'static, TIM2>,
) {
    // This is a placeholder for PWM test code.
    // Implement PWM functionality tests here.

    // zero crossing detection setup
    let ch2 = CapturePin::new(signal_input, Pull::None);
    let mut ic = InputCapture::new(
        tim3,
        None,
        Some(ch2),
        None,
        None,
        Irqs,
        khz(1000),
        Default::default(),
    );

    const PWM_OUTPUT_HZ: u32 = 2000;

    const DUTY_CYCLE_PERCENT: u32 = 100;

    //pwm output setup
    let pwm_ch2 = PwmPin::new(signal_output, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        pwm_timer,
        None,
        Some(pwm_ch2),
        None,
        None,
        hz(PWM_OUTPUT_HZ),
        Default::default(),
    );

    pwm.set_frequency(hz(100));
    pwm.set_duty(Channel::Ch2, pwm.get_max_duty() * DUTY_CYCLE_PERCENT / 100);

    loop {
        let _falling = ic.wait_for_falling_edge(Channel::Ch2).await;
        info!("Zero crossing detected");

        pwm.enable(Channel::Ch2);
        Timer::after_millis(2).await;
        pwm.disable(Channel::Ch2);
    }
}
