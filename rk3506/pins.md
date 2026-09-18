# Used and reserved 40-pin connector pins

This document covers the Luckfox Lyra Zero W 40-pin connector (`PI1` in the
schematic). Onboard-only signals and pins on the separate DSI connector are not
included.

## Used and reserved pins

- Pin 3 — `TOUCH_SDA` — GPIO0_A0 — I2C2 SDA, 3.3 V host side
- Pin 5 — `TOUCH_SCL` — GPIO0_A1 — I2C2 SCL, 3.3 V host side
- Pin 7 — `UART1_TX` — GPIO0_A2 — enabled application UART transmit
- Pin 8 — `UART0_TX` — GPIO0_C6 — reserved for a future debug console; not enabled
- Pin 10 — `UART0_RX` — GPIO0_C7 — reserved for a future debug console; not enabled
- Pin 11 — `TOUCH_INT` — GPIO0_A3 — falling-edge interrupt
- Pin 12 — `UART1_RX` — GPIO0_B6 — enabled application UART receive
- Pin 13 — `TOUCH_RST` — GPIO0_A4 — active-low reset
- Pin 16 — `LCD_RST` — GPIO0_B5 — active-low reset
- Pin 29 — `CAN1_TX` — GPIO1_B2 — second transceiver TXD
- Pin 31 — `CAN1_RX` — GPIO1_B3 — second transceiver RXD
- Pin 33 — `CAN0_TX` — GPIO1_C2 — first transceiver TXD
- Pin 37 — `CAN0_RX` — GPIO1_C3 — first transceiver RXD

## Physical 40-pin connector layout

Power and ground rails are shown directly. An em dash (`—`) means the remaining
signal pin is not claimed by the project-specific display, touch, CAN, or UART
configuration. UART0 is shown because its pins are reserved, although UART0 is
not currently enabled.

Pin pair 1/2 is at the top of the connector and pin pair 39/40 is at the
bottom.

| Odd-side use | Odd pin name | Odd pin | Even pin | Even pin name | Even-side use |
| --- | --- | ---: | ---: | --- | --- |
| 3.3 V | 3.3 V | 1 | 2 | 5 V | 5 V |
| `TOUCH_SDA` | GPIO0_A0 | 3 | 4 | 5 V | 5 V |
| `TOUCH_SCL` | GPIO0_A1 | 5 | 6 | GND | GND |
| `UART1_TX` | GPIO0_A2 | 7 | 8 | GPIO0_C6 | `UART0_TX` (reserved) |
| GND | GND | 9 | 10 | GPIO0_C7 | `UART0_RX` (reserved) |
| `TOUCH_INT` | GPIO0_A3 | 11 | 12 | GPIO0_B6 | `UART1_RX` |
| `TOUCH_RST` | GPIO0_A4 | 13 | 14 | GND | GND |
| — | GPIO0_A5 | 15 | 16 | GPIO0_B5 | `LCD_RST` |
| 3.3 V | 3.3 V | 17 | 18 | GPIO0_B4 | — |
| — | GPIO0_A6 | 19 | 20 | GND | GND |
| — | GPIO0_A7 | 21 | 22 | GPIO0_B3 | — |
| — | GPIO0_B0 | 23 | 24 | GPIO0_B2 | — |
| GND | GND | 25 | 26 | GPIO0_B1 | — |
| — | GPIO1_B1 | 27 | 28 | GPIO1_D3 | — |
| `CAN1_TX` | GPIO1_B2 | 29 | 30 | GND | GND |
| `CAN1_RX` | GPIO1_B3 | 31 | 32 | GPIO1_D2 | — |
| `CAN0_TX` | GPIO1_C2 | 33 | 34 | GND | GND |
| — | GPIO0_C0 | 35 | 36 | GPIO1_D1 | — |
| `CAN0_RX` | GPIO1_C3 | 37 | 38 | GND | GND |
| — | GPIO0_C2 | 39 | 40 | NC | — |

## Power and ground

The following connector pins are power rails or ground rather than software
controlled signals:

- 3.3 V: pins 1 and 17
- 5 V: pins 2 and 4
- Ground: pins 6, 9, 14, 20, 25, 30, 34, and 38

Use a common ground between the Lyra and each external CAN transceiver. The CAN
controller pins are 3.3 V single-ended logic; never connect them directly to
CAN-H or CAN-L. The touch entries describe the adapter/host side. The bare
panel touch signals use different voltage levels and must not be wired directly
from this table without the appropriate adapter or level translation.

The UART pins are 3.3 V TTL logic. Connect UART1 TX to the external device's RX,
UART1 RX to its TX, and use a common ground. Do not connect RS-232 voltage levels
or 5 V UART signals directly. UART0 remains disabled and reserved for a possible
future debug console.

## Source of truth

The assignments come from the selected board device tree and its CO6300 include:

- `kernel/arch/arm/boot/dts/rk3506b-luckfox-lyra-zero-w-co6300.dts`
- `kernel/arch/arm/boot/dts/rk3506-luckfox-lyra-co6300.dtsi`

Physical connector numbering was checked against the official Luckfox Lyra
Zero W schematic, connector `PI1`.
