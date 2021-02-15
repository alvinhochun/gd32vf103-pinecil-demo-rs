#![no_std]
#![no_main]

use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    let mut peripherals = gd32vf103_pac::Peripherals::take().unwrap();
    init_ports(&mut peripherals);
    blink_led(&mut peripherals);
    loop {}
}

// const GPIO_MD_INPUT: u8 = 0b00;
// const GPIO_MD_OUTPUT_MAX10MHZ: u8 = 0b01;
const GPIO_MD_OUTPUT_MAX2MHZ: u8 = 0b10;
// const GPIO_MD_OUTPUT_MAX50MHZ: u8 = 0b11;

// const GPIO_CTL_INPUT_ANALOG: u8 = 0b00;
// const GPIO_CTL_INPUT_FLOATING: u8 = 0b01;
// const GPIO_CTL_INPUT_PULLUP_PULLDOWN: u8 = 0b10;

const GPIO_CTL_OUTPUT_GPIO_PUSH_PULL: u8 = 0b00;
// const GPIO_CTL_OUTPUT_GPIO_OPEN_DRAIN: u8 = 0b01;
// const GPIO_CTL_OUTPUT_AFIO_PUSH_PULL: u8 = 0b10;
// const GPIO_CTL_OUTPUT_AFIO_OPEN_DRAIN: u8 = 0b11;

fn init_ports(peripherals: &mut gd32vf103_pac::Peripherals) {
    // Enable clock to Port A.
    peripherals.RCU.apb2en.modify(|_r, w| w.paen().set_bit());
    // Set PA2 to push-pull output.
    peripherals.GPIOA.ctl0.modify(|_r, w| unsafe {
        w.md2()
            .bits(GPIO_MD_OUTPUT_MAX2MHZ)
            .ctl2()
            .bits(GPIO_CTL_OUTPUT_GPIO_PUSH_PULL)
    });
}

fn delay(mut n: u32) {
    while n != 0 {
        unsafe {
            core::ptr::write_volatile(&mut n, n - 1);
        }
    }
}

fn blink_led(peripherals: &mut gd32vf103_pac::Peripherals) {
    loop {
        // Toggle LED on PA2.
        peripherals
            .GPIOA
            .octl
            .modify(|r, w| w.octl2().bit(!r.octl2().bit()));
        delay(0x4ffff);
    }
}
