Demo 06 - OLED
===

In this demo, we draw something to the OLED, making use of the `ssd1306` driver
crate.

The OLED is connected to the I2C bus on the Pinecil, to pins PB6 (SCL) and
PB7 (SDA), which maps to the I2C0 peripheral. It also has a reset pin (RES#)
which is connected to PA9.
