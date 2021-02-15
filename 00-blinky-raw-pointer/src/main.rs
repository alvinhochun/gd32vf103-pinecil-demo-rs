#![no_std]
#![no_main]

use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    init_ports();
    blink_led();
    loop {}
}

const RCU_APB2EN: *mut u32 = (0x4002_1000 + 0x18) as *mut u32;
const GPIOA_CTL0: *mut u32 = (0x4001_0800 + 0x0) as *mut u32;
const GPIOA_OCTL: *mut u32 = (0x4001_0800 + 0xc) as *mut u32;

fn init_ports() {
    unsafe {
        // Enable clock to Port A (set PAEN)
        let x = core::ptr::read_volatile(RCU_APB2EN);
        core::ptr::write_volatile(RCU_APB2EN, x | (1 << 2));

        // Enable open-drain output for Port A pin 2
        let x = core::ptr::read_volatile(GPIOA_CTL0);
        core::ptr::write_volatile(GPIOA_CTL0, x | (1 << 8));
    }
}

fn delay(mut n: u32) {
    while n != 0 {
        unsafe {
            core::ptr::write_volatile(&mut n, n - 1);
        }
    }
}

// Blink LED (PA2).
fn blink_led() {
    const BITMASK: u32 = 1 << 2;
    let mut bits: u32 = BITMASK;
    loop {
        unsafe {
            // LED on when PA2 bit is 0
            core::ptr::write_volatile(GPIOA_OCTL, bits);
        }
        // Delay for an inexact duration
        delay(0x4ffff);
        bits ^= BITMASK;
    }
}
