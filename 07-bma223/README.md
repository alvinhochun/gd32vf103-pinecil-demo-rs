Demo 07 - Reading BMA223 Accelerometer
===

In this demo, we attempt to read the acceleration values from the BMA223
acvcelerometer using the I2C bus. For more information, refer to the BMA223
datasheet.

Before setting up the I2C bus, the code toggles the SCL pin 16 times to attempt
to reset any I2C devices that might be stuck. This is a dirty trick which may
or may not work.

Curiously, this demo does not work properly on a debug build, with or without
the SCL pin toggling code. I have yet to figure out the reason why this
happens.
