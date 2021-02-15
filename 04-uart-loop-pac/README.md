Demo 04 - Send UART output using the peripheral access crate
===

In this demo, we no longer use an LED, but instead send serial data output
through the USART. We use a USB to UART adapter or a single-board computer
with UART input (Raspberry Pi for example) compatible with 3.3V logic level,
and hook it up to the Pinecil breakout board to receive the data.

**Note:** To prevent issues caused by different ground levels, try to make sure
that the ground of both deivces do not have a significant potential difference,
that both devices are powered using the same power source, or that at least one
of them are powered from an isolated power source.

**Warning:** Do not mistakenly connect the TX pins of both devices together,
as it can damage the electronics.

```
Pinecil           UART adapter,
breakout          Raspberry Pi,
board             etc.
----+             +----
  [TXD] ------- [RXD]
    |             |
  [RXD]   -X-   [TXD]  (not connected)
    |             |
  [GND] ------- [GND]
----+             +----
```

We also switch the microcontroller to use the external 8MHz crystal instead of
using the internal 8MHz RC oscillator, and switch on the PLL frequency
mutiplier in order to get a higher system clock to get a higher UART baud rate.

This demo uses 2_000_000 / 8N1 UART by default. You can try changing it by
modifying the code.
