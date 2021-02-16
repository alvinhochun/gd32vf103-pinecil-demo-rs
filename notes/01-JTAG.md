Using OpenOCD fork on https://github.com/riscv/riscv-openocd

On the Pinecil, the JTAG pins JTMS and JTCK are connected to the INT1 and INT2
pins of the BMA223 accelerometer chip. These pins are inconveniently pulled
low by the BMA223, which means JTAG is not usable unless the BMA223 is
configured to use open drain and active low on these pins, which will free the
pins to be used for JTAG. To do this, one needs to write on the I2C bus:

- Address: 0x18
- Write data: 0x20, 0x0A

This sets the register `INT_OUT_CTRL` on the BMA223 to `0b1010`, which
configures both INT1 and INT2 to be open drain and active low. It can be
written by the GD32VF103, or with an external device via the breakout board.
Once it is written, it persists as long as the power to the Pinecil is not
interrupted.

---

I use a Raspberry Pi 3B+ as a JTAG debugger and run OpenOCD on it. After
compiling and installing OpenOCD, check
`/usr/local/share/openocd/scripts/interface/raspberrypi2-native.cfg` for the
pinout to use. For me it is:

```
# Each of the JTAG lines need a gpio number set: tck tms tdi tdo
# Header pin numbers: 23 22 19 21
bcm2835gpio_jtag_nums 11 25 10 9
```

Hook up these pins to the Pinecil breakout board, and don't forget to connect
GND.

Since this driver is using bit-bang I/O, you shouldn't be running heavy
applications on the RPi.

Run:

```
$ sudo openocd -f /usr/local/share/openocd/scripts/interface/raspberrypi2-native.cfg  -f /usr/local/share/openocd/scripts/target/gd32vf103.cfg -c "adapter speed 100"
```

Add `-c "bindto 0.0.0.0"` to the end if the Pi is connected to an internal
network, so you can connect to it on another PC.

I found 100 kHz to be stable enough. If you are feeling lucky you may try
bumping it up (change `adapter speed 100`).

If everything went well and you have configured the BMA223 chip properly, you
should see these lines among the output:

```
Info : JTAG tap: gd32vf103.cpu tap/device found: 0x1000563d (mfg: 0x31e (Andes Technology Corporation), part: 0x0005, ver: 0x1)
Info : JTAG tap: gd32vf103.bs tap/device found: 0x790007a3 (mfg: 0x3d1 (GigaDevice Semiconductor (Beijing) Inc), part: 0x9000, ver: 0x7)
Info : datacount=4 progbufsize=2
Info : Examined RISC-V core; found 1 harts
Info :  hart 0: XLEN=32, misa=0x40901105
```

This means OpenOCD has successfully connected to the JTAG on the GD32VF103.

---

To debug with vscode, I use this launch configuration:

```jsonc
{
    "type": "gdb",
    "gdbpath": "../../gcc/bin/riscv-nuclei-elf-gdb.exe", // change to point to a gdb binary compatible with RISC-V
    "request": "launch",
    "name": "Attach to OpenOCD on Raspberry Pi (Debug)",
    "target": "./target/riscv32imac-unknown-none-elf/debug/demo-06-oled", // change to point to the ELF binary
    "cwd": "${workspaceRoot}",
    "valuesFormatting": "parseText",
    "autorun": [
        "target extended-remote 192.168.0.100:3333", // change to point to the OpenOCD gdb server
        "load", // this command flashes the ELF file onto the Pinecil
    ],
},
```

The GD32VF103 is limited to 4 hardware breakpoints.
