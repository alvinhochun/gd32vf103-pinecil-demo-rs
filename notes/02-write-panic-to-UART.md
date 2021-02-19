Assuming your code already initialized the UART1 at the start, this panic
handler can be used instead of `panic-halt` to print panics to the UART for
diagnosis.

Keep in mind that this increases code size by quite some amount.

```rust
mod my_panic_handler {
    use core::fmt::Write;

    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        let mut peripherals = unsafe { gd32vf103_pac::Peripherals::steal() };

        // Assume the UART is already initialized.
        let mut uart = PanicUart {
            usart: &mut peripherals.USART1,
        };
        let _ = write!(uart, "PANIC: {:?}\r\n", info);

        loop {}
    }

    struct PanicUart<'a> {
        usart: &'a mut gd32vf103_pac::USART1,
    }

    impl<'a> Write for PanicUart<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            fn send_uart_byte(usart: &mut gd32vf103_pac::USART1, b: u8) {
                // Wait for TBE (transmit data buffer empty) set.
                while !usart.stat.read().tbe().bit_is_set() {}

                // Send byte.
                usart.data.write(|w| unsafe { w.data().bits(b as u16) });
            }

            for &b in s.as_bytes() {
                send_uart_byte(&mut self.usart, b);
            }

            Ok(())
        }
    }
}
```
