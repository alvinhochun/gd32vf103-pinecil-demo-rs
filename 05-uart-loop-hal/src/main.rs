#![no_std]
#![no_main]

use panic_halt as _;

use gd32vf103xx_hal as hal;
use hal::{delay::McycleDelay, prelude::*, time::Bps};

#[riscv_rt::entry]
fn main() -> ! {
    let peripherals = gd32vf103_pac::Peripherals::take().unwrap();

    // Use external 8MHz HXTAL and set PLL to get 96MHz system clock.
    let mut rcu = peripherals
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(96.mhz())
        .freeze();

    let mut afio = peripherals.AFIO.constrain(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);

    let pa = peripherals.GPIOA.split(&mut rcu);
    let pa2_tx = pa.pa2.into_alternate_push_pull();
    let pa3_rx = pa.pa3.into_pull_up_input();

    let (mut uart1_tx, _uart1_rx) = hal::serial::Serial::new(
        peripherals.USART1,
        (pa2_tx, pa3_rx),
        hal::serial::Config {
            baudrate: Bps(2_000_000),
            ..Default::default()
        },
        &mut afio,
        &mut rcu,
    )
    .split();

    loop {
        // Send "Hello world!\n".
        for &b in b"Hello world!\r\n" {
            nb::block!(uart1_tx.write(b)).unwrap();
        }

        delay.delay_ms(500);
    }
}
