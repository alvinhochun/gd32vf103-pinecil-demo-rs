#![no_std]
#![no_main]
#![warn(unused_unsafe)]

use gd32vf103_pac::USART1;

use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    let mut peripherals = gd32vf103_pac::Peripherals::take().unwrap();

    init_clock(&mut peripherals);

    // do something here
    init_ports(&mut peripherals);
    // blink_led(&mut peripherals);
    send_uart(&mut peripherals);
    loop {}
}

fn init_clock(peripherals: &mut gd32vf103_pac::Peripherals) {
    // Enable HXTAL (high speed crystal oscillator).
    peripherals.RCU.ctl.modify(|_r, w| w.hxtalen().set_bit());

    // Wait for clock stabilization.
    while !peripherals.RCU.ctl.read().hxtalstb().bit_is_set() {}

    // Enable HXTAL clock monitor.
    peripherals.RCU.ctl.modify(|_r, w| w.ckmen().set_bit());

    // Set PLL source to HXTAL.
    peripherals.RCU.cfg0.modify(|_r, w| w.pllsel().set_bit());

    // Targeting 96MHz system clock.
    // 96 / 8 = 12, setting PLL to x12.
    peripherals
        .RCU
        .cfg0
        .modify(|_r, w| unsafe { w.pllmf_4().clear_bit().pllmf_3_0().bits(0b1010) });

    // Set APB1 prescaler to /2 (APB1 clock must not exceed 60MHz).
    peripherals
        .RCU
        .cfg0
        .modify(|_r, w| unsafe { w.apb1psc().bits(0b100) });

    // Enable PLL.
    peripherals.RCU.ctl.modify(|_r, w| w.pllen().set_bit());

    // Wait for PLL stabiliaztion.
    while !peripherals.RCU.ctl.read().pllstb().bit_is_set() {}

    // Switch to PLL as system clock.
    peripherals
        .RCU
        .cfg0
        .modify(|_r, w| unsafe { w.scs().bits(0b10) });

    // Wait for system clock switch.
    while peripherals.RCU.cfg0.read().scss().bits() != 0b10 {}

    // Disable internal RC oscillator.
    peripherals.RCU.ctl.modify(|_r, w| w.irc8men().clear_bit());
}

const GPIO_MD_INPUT: u8 = 0b00;
const GPIO_MD_OUTPUT_MAX10MHZ: u8 = 0b01;
// const GPIO_MD_OUTPUT_MAX2MHZ: u8 = 0b10;
// const GPIO_MD_OUTPUT_MAX50MHZ: u8 = 0b11;

// const GPIO_CTL_INPUT_ANALOG: u8 = 0b00;
// const GPIO_CTL_INPUT_FLOATING: u8 = 0b01;
const GPIO_CTL_INPUT_PULLUP_PULLDOWN: u8 = 0b10;

// const GPIO_CTL_OUTPUT_GPIO_PUSH_PULL: u8 = 0b00;
// const GPIO_CTL_OUTPUT_GPIO_OPEN_DRAIN: u8 = 0b01;
const GPIO_CTL_OUTPUT_AFIO_PUSH_PULL: u8 = 0b10;
// const GPIO_CTL_OUTPUT_AFIO_OPEN_DRAIN: u8 = 0b11;

fn init_ports(peripherals: &mut gd32vf103_pac::Peripherals) {
    // Enable clock to Port A.
    peripherals.RCU.apb2en.modify(|_r, w| w.paen().set_bit());
    // Enable clock to AFIO.
    peripherals.RCU.apb2en.modify(|_r, w| w.afen().set_bit());

    // Enable clock to USART1.
    peripherals
        .RCU
        .apb1en
        .modify(|_r, w| w.usart1en().set_bit());

    // Set PA2 (USART1_TX) to AFIO push-pull output,
    // and set PA3 (USART1_RX) to AFIO input with internal pull-up/pull-down.
    peripherals.GPIOA.ctl0.modify(|_r, w| unsafe {
        let w = w.md2().bits(GPIO_MD_OUTPUT_MAX10MHZ);
        let w = w.ctl2().bits(GPIO_CTL_OUTPUT_AFIO_PUSH_PULL);
        let w = w.md3().bits(GPIO_MD_INPUT);
        let w = w.ctl3().bits(GPIO_CTL_INPUT_PULLUP_PULLDOWN);
        w
    });

    // Set PA3 (USART1_RX) to pull-up.
    peripherals.GPIOA.octl.modify(|_r, w| w.octl3().set_bit());

    // Assuming system clock is 8MHz:
    // PCLK1 == 8_000_000
    // Targeting 115_200 bps:
    // USARTDIV = PCLK / (16 * baud)
    //          = 8_000_000 / (16 * 115_200)
    //          = 4.34
    // INTDIV = 4
    // FRADIV = 16 * 0.34 = 5.44 ~= 5
    // USART_BAUD = 0x45
    // ---
    // USART_BAUD = 0x45 => 115_200 bps
    // USART_BAUD = 0x10 => 500_000 bps

    // Assuming system clock is 96MHz and APB1 prescaler is /2:
    // PCLK1 == 48_000_000
    // Targeting 115_200 bps:
    // USARTDIV = PCLK / (16 * baud)
    //          = 48_000_000 / (16 * 115_200)
    //          = 26.04
    // INTDIV = 26
    // FRADIV = 16 * 0.04 = 0.64 ~= 1
    // USART_BAUD = 0x1A1
    // ---
    // USART_BAUD = 0x1A1 => 115_200 bps
    // USART_BAUD = 0x18 => 2_000_000 bps

    peripherals.USART1.baud.write(|w| unsafe { w.bits(0x18) });

    // Enable transmitter.
    peripherals.USART1.ctl0.modify(|_r, w| w.ten().set_bit());

    // Don't enable receiver as we're not using it for now.

    // Default is 8N1, so no need to change.

    // Enable USART1.
    peripherals.USART1.ctl0.modify(|_r, w| w.uen().set_bit());
}

fn send_uart(peripherals: &mut gd32vf103_pac::Peripherals) {
    loop {
        // Send "Hello world!\n".
        for &b in b"Hello world!\r\n" {
            send_uart_byte(&mut peripherals.USART1, b);
        }

        delay(0x4ffff);
    }
}

fn send_uart_byte(usart: &mut USART1, b: u8) {
    // Wait for TBE (transmit data buffer empty) set.
    while !usart.stat.read().tbe().bit_is_set() {}

    // Send byte.
    usart.data.write(|w| unsafe { w.data().bits(b as u16) });
}

fn delay(mut n: u32) {
    while n != 0 {
        unsafe {
            core::ptr::write_volatile(&mut n, n - 1);
        }
    }
}
