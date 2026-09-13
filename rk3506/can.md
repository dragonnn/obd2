# CAN and OBD-II

The board image enables the RK3506 CAN0 controller as the Linux SocketCAN
interface `can0`. The init script configures Classical CAN at 500 kbit/s with
automatic bus-off recovery after 100 ms. Both raw CAN sockets and the ISO-TP
transport used by ISO 15765 diagnostics are built into the kernel.

## Board pins and transceiver

The RK3506 pins carry single-ended 3.3 V logic, not the differential CAN bus.
An external 3.3 V logic-compatible high-speed CAN transceiver is mandatory.

| Signal | Lyra Zero W header | SoC signal | Connect to |
| --- | ---: | --- | --- |
| CAN0 TX | pin 35 | GPIO1_C2 / RM_IO27 | transceiver TXD |
| CAN0 RX | pin 37 | GPIO1_C3 / RM_IO28 | transceiver RXD |
| Ground | pin 39 (or another GND pin) | GND | transceiver GND and OBD ground |

Connect the transceiver bus side to OBD-II pin 6 (CAN High) and pin 14 (CAN
Low). Connect ground to OBD-II signal ground pin 5. Do not connect OBD-II pin
16 directly to the Lyra power rails; it is unswitched vehicle battery voltage
and needs an automotive-rated protected regulator if it powers the board.

The vehicle diagnostic bus is already terminated. Do not fit a 120 ohm
termination resistor at this short diagnostic stub unless measurements show
that the bus is otherwise unterminated.

## Verification

After booting the new image:

```sh
ip -details link show can0
candump can0
```

The first command should report `state UP` and `bitrate 500000`. With the
adapter connected to a powered vehicle, a basic functional OBD-II request can
be sent with:

```sh
cansend can0 7DF#0201000000000000
```

Replies normally use identifiers `7E8` through `7EF`. Passive `candump` is the
safer first hardware test. Only transmit diagnostic requests you understand;
other CAN traffic can affect vehicle systems.
