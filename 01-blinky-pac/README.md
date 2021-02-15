Demo 01 - Blink LED using the peripheral access crate
===

In this demo, we blink an LED connected to the TX pin on the breakout board,
same as the previous demo. But instead of defining the addresses to the
registers ourselves, we use a peripheral access crate (PAC) which already has
the addresses defined and provides a light layer of abstraction over register
accesses.
