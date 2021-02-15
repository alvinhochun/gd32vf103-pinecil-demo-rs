#![no_std]
#![no_main]

use core::convert::Infallible;

use panic_halt as _;

use embedded_hal::digital::v2::{InputPin, OutputPin, ToggleableOutputPin};
use gd32vf103xx_hal as hal;
use hal::{delay::McycleDelay, prelude::*};

#[riscv_rt::entry]
fn main() -> ! {
    let peripherals = gd32vf103_pac::Peripherals::take().unwrap();

    // Use internal 8MHz RC oscillator without PLL multiplier.
    let mut rcu = peripherals.RCU.configure().freeze();

    let delay = McycleDelay::new(&rcu.clocks);

    // Use PA2 as a push-pull output.
    let pa = peripherals.GPIOA.split(&mut rcu);
    let pa2out = pa.pa2.into_push_pull_output();

    let pb = peripherals.GPIOB.split(&mut rcu);
    // Use PB0 as input for the '+' button (butt_B).
    let pb0in = pb.pb0.into_pull_down_input();
    // USE PB1 as input for the '-' button (butt_A).
    // Note that this pin is already pulled low externally via a 10K resistor
    // since it also operates the BOOT0 pin, so we don't need the internal
    // pull-down.
    let pb1in = pb.pb1.into_floating_input();

    run(pa2out, pb1in, pb0in, delay);
    loop {}
}

fn run(
    mut led: impl OutputPin<Error = Infallible> + ToggleableOutputPin<Error = Infallible>,
    btn_a: impl InputPin<Error = Infallible>,
    btn_b: impl InputPin<Error = Infallible>,
    mut delay: McycleDelay,
) {
    loop {
        match (btn_a.is_high().unwrap(), btn_b.is_high().unwrap()) {
            (false, false) => {
                led.toggle().unwrap();
                delay.delay_ms(500);
            }
            (true, false) => {
                led.set_high().unwrap();
                delay.delay_ms(10);
            }
            (false, true) => {
                led.set_low().unwrap();
                delay.delay_ms(10);
            }
            (true, true) => {
                led.toggle().unwrap();
                delay.delay_ms(100);
            }
        }
    }
}
