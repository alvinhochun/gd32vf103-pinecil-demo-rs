#![no_std]
#![no_main]

use core::{fmt::Write, iter::repeat};

use panic_halt as _;

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use gd32vf103xx_hal::{self as hal, prelude::*};
use hal::{delay::McycleDelay, time::Bps};

use ssd1306::{prelude::*, Builder, I2CDIBuilder};

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

    let mut delay = McycleDelay::new(&rcu.clocks);

    let mut afio = peripherals.AFIO.constrain(&mut rcu);

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
    // Use PB0 as input for the '+' button (butt_B).
    let btn_b = pb.pb0.into_pull_down_input();
    // USE PB1 as input for the '-' button (butt_A).
    // Note that this pin is already pulled low externally via a 10K resistor
    // since it also operates the BOOT0 pin, so we don't need the internal
    // pull-down.
    let btn_a = pb.pb1.into_floating_input();

    // OLED reset: Pull low to reset.
    let mut oled_reset = pa
        .pa9
        .into_push_pull_output_with_state(hal::gpio::State::Low);

    let pb6_scl = pb.pb6.into_alternate_open_drain();
    let pb7_sda = pb.pb7.into_alternate_open_drain();

    // Set up i2c.
    let i2c0 = hal::i2c::BlockingI2c::i2c0(
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

    // OLED datasheet recommends 100 ms delay on power up.
    delay.delay_ms(100);

    // Init OLED.
    oled_reset.set_high().unwrap();

    // OLED datasheet recommends 3 us delay to wait for init.
    delay.delay_us(3);

    let mut disp = {
        let interface = I2CDIBuilder::new().init(i2c0);

        let mut disp_g: GraphicsMode<_, _> = Builder::new()
            .size(DisplaySize96x16)
            .with_rotation(DisplayRotation::Rotate180)
            .connect(interface)
            .into();
        disp_g.init().unwrap_or_else(|e| {
            write!(uart1_tx, "Error initializing OLED: {:?}\r\n", e).unwrap();
            panic!()
        });

        DisplayModeEnum::Graphics(disp_g)
    };

    let mut state = 1;
    let mut brightness = 0x0F_u8;

    macro_rules! set_brightness {
        ($disp:expr) => {{
            let _ = $disp.set_brightness(Brightness::custom(0xF1, brightness));
        }};
    }
    macro_rules! btn_check {
        ($e:expr) => {
            if btn_a.is_high().unwrap() {
                state += 1;
                $e
            }
            if btn_b.is_high().unwrap() {
                brightness = brightness.wrapping_add(16);
                $e
            }
        };
    }
    macro_rules! wait_btn_release {
        () => {
            while btn_a.is_high().unwrap() || btn_b.is_high().unwrap() {}
        };
    }

    loop {
        match state {
            1 => {
                disp.use_graphics(|disp_g| {
                    set_brightness!(disp_g);
                    let raw_frames = {
                        macro_rules! frame {
                            ($s:expr) => {
                                ImageRaw::<BinaryColor>::new(include_bytes!($s), 96, 16)
                            };
                        }
                        [frame!("frame0.raw"), frame!("frame1.raw")]
                    };
                    for raw in repeat(&raw_frames).flatten() {
                        let image = Image::new(raw, (0, 0).into());
                        let _ = image.draw(disp_g);
                        disp_g.flush().unwrap();

                        for _ in 0..10 {
                            delay.delay_ms(25);
                            btn_check!(return);
                        }
                    }
                });
                wait_btn_release!();
            }
            2 => {
                disp.use_terminal(|disp_t| {
                    set_brightness!(disp_t);
                    let _ = disp_t.clear();

                    let _ = disp_t.write_str("Hello world!");
                    loop {
                        btn_check!(return);
                    }
                });
                wait_btn_release!();
            }
            3 => {
                disp.use_terminal(|disp_t| {
                    set_brightness!(disp_t);
                    let _ = disp_t.clear();

                    for c in repeat((b'a'..=b'z').chain(b'A'..=b'Z').chain(b'0'..=b'9')).flatten() {
                        let _ = disp_t.print_char(c.into());
                        for _ in 0..4 {
                            delay.delay_ms(25);
                            btn_check!(return);
                        }
                    }
                });
                wait_btn_release!();
            }
            4 => {
                disp.use_terminal(|disp_t| {
                    set_brightness!(disp_t);
                    let _ = disp_t.clear();

                    let _ = write!(disp_t, "Brightness:\n--{}--", brightness);
                    loop {
                        btn_check!(return);
                    }
                });
                wait_btn_release!();
            }
            0 => state = 3,
            _ => state = 1,
        }
    }
}

enum DisplayModeEnum<T, U>
where
    T: WriteOnlyDataCommand,
    U: ssd1306::displaysize::DisplaySize + ssd1306::mode::terminal::TerminalDisplaySize,
{
    None,
    Graphics(GraphicsMode<T, U>),
    Terminal(TerminalMode<T, U>),
}

impl<T, U> DisplayModeEnum<T, U>
where
    T: WriteOnlyDataCommand,
    U: ssd1306::displaysize::DisplaySize + ssd1306::mode::terminal::TerminalDisplaySize,
{
    fn into_inner(self) -> DisplayProperties<T, U> {
        match self {
            DisplayModeEnum::Graphics(m) => m.into_properties(),
            DisplayModeEnum::Terminal(m) => m.into_properties(),
            DisplayModeEnum::None => panic!(),
        }
    }

    fn take_inner(&mut self) -> DisplayProperties<T, U> {
        core::mem::replace(self, DisplayModeEnum::None).into_inner()
    }

    fn use_graphics(&mut self, f: impl FnOnce(&mut GraphicsMode<T, U>)) {
        let mut m = match self {
            DisplayModeEnum::Graphics(m) => m,
            _ => {
                let m: GraphicsMode<_, _> = self.take_inner().into();
                *self = DisplayModeEnum::Graphics(m);
                match self {
                    DisplayModeEnum::Graphics(m) => {
                        m.init().unwrap();
                        m
                    }
                    _ => unreachable!(),
                }
            }
        };
        f(&mut m);
    }

    fn use_terminal(&mut self, f: impl FnOnce(&mut TerminalMode<T, U>)) {
        let mut m = match self {
            DisplayModeEnum::Terminal(m) => m,
            _ => {
                let m: TerminalMode<_, _> = self.take_inner().into();
                *self = DisplayModeEnum::Terminal(m);
                match self {
                    DisplayModeEnum::Terminal(m) => {
                        m.init().unwrap();
                        m
                    }
                    _ => unreachable!(),
                }
            }
        };
        f(&mut m);
    }
}
