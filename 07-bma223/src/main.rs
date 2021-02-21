#![no_std]
#![no_main]

use core::fmt::Write;

use panic_halt as _;

use embedded_hal::digital::v2::OutputPin;
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

    let pb = peripherals.GPIOB.split(&mut rcu);

    let mut pb6_scl = pb
        .pb6
        .into_open_drain_output_with_state(hal::gpio::State::High);
    let pb7_sda = pb
        .pb7
        .into_open_drain_output_with_state(hal::gpio::State::High);
    // Attempt to unstuck the i2c bus.
    for _ in 0..16 {
        delay.delay_us(5);
        let _ = pb6_scl.set_low();
        delay.delay_us(5);
        let _ = pb6_scl.set_high();
    }
    let pb6_scl = pb6_scl.into_alternate_open_drain();
    let pb7_sda = pb7_sda.into_alternate_open_drain();

    // Set up i2c.
    let mut i2c0 = hal::i2c::BlockingI2c::i2c0(
        peripherals.I2C0,
        (pb6_scl, pb7_sda),
        &mut afio,
        hal::i2c::Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: hal::i2c::DutyCycle::Ratio2to1,
        },
        &mut rcu,
        1000,
        10,
        1000,
        1000,
    );

    // Write register 0x20: 0b1010 - set to open drain active low for INT1 and INT2 to prevent blocking JTAG operation
    const BMA223_ADDR: u8 = 0x18;
    const BMO223_CHIP_ID: u8 = 0b11111000;
    const BMA223_REG_BGW_CHIPID: u8 = 0x00;
    const BMA223_REG_INT_OUT_CTRL: u8 = 0x20;
    const BMA223_REG_ACCD_X_LSB: u8 = 0x02;

    // This frees up the JTAG pins (see `notes/01-JTAG.md`).
    let _ = nb::block!(i2c0.write(BMA223_ADDR, &[BMA223_REG_INT_OUT_CTRL, 0b1010])).unwrap_or_else(
        |e| {
            write!(
                uart1_tx,
                "Error writing INT_OUT_CTRL to BMA223: {:?}\r\n",
                e
            )
            .unwrap()
        },
    );

    {
        let mut read = [0; 1];
        match nb::block!(i2c0.write_read(BMA223_ADDR, &[BMA223_REG_BGW_CHIPID], &mut read)) {
            Ok(()) => write!(uart1_tx, "Read BMA223 chip id: {:#010b}\r\n", read[0]).unwrap(),
            Err(e) => write!(
                uart1_tx,
                "Error writing to and reading from BMA223: {:?}\r\n",
                e
            )
            .unwrap(),
        }
    }

    loop {
        delay.delay_ms(10);

        let mut read = [0; 7];
        match nb::block!(i2c0.write_read(BMA223_ADDR, &[BMA223_REG_ACCD_X_LSB], &mut read)) {
            Ok(()) => {}
            Err(e) => {
                write!(
                    uart1_tx,
                    "Error writing to and reading data from BMA223: {:?}\r\n",
                    e
                )
                .unwrap();
                continue;
            }
        }

        let _accd_x_updated = read[0] & 0b1 != 0;
        let accd_x = read[1] as i8;
        let _accd_y_updated = read[2] & 0b1 != 0;
        let accd_y = read[3] as i8;
        let _accd_z_updated = read[4] & 0b1 != 0;
        let accd_z = read[5] as i8;
        let accd_temp = read[6] as i8;

        write!(
            uart1_tx,
            "BMA223: x={:<+6}  y={:<+6}  z={:<+6}  temp={:<+6} \r",
            accd_x, accd_y, accd_z, accd_temp,
        )
        .unwrap();
    }
}
