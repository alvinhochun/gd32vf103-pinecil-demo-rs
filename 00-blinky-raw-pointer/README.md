Demo 00 - Blink LED using direct memory-mapped peripheral access
===

In this demo, we blink an LED connected to the TX pin on the breakout board:

```
          -|>|-
[3V3] --- LED --- Resistor --- [TXD (PA2)]
```

Use a resistor to limit the current below 10 mA (the absolute limit is 20mA),
but really even 5 mA is more than enough. A reasonable value is 470 ohm.

In order to blink the LED, we need to:

1. Enable clock to GPIO port A.
2. Set pin PA2 to be an output. In this case since the pin is sinking current,
   either push-pull or open drain is fine.
3. Toggle the pin state to switch on and off the LED, with a delay in between.
