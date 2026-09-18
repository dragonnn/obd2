# Used 40-pin connector pins

This document covers the Luckfox Lyra Zero W 40-pin connector (`PI1` in the
schematic). Onboard-only signals and pins on the separate DSI connector are not
included.

## Used pins

- Pin 3 — `TOUCH_SDA` — GPIO0_A0 / RM_IO0 — I2C2 SDA, 3.3 V host side
- Pin 5 — `TOUCH_SCL` — GPIO0_A1 / RM_IO1 — I2C2 SCL, 3.3 V host side
- Pin 11 — `TOUCH_INT` — GPIO0_A3 / RM_IO3 — falling-edge interrupt
- Pin 13 — `TOUCH_RST` — GPIO0_A4 / RM_IO4 — active-low reset
- Pin 16 — `LCD_RST` — GPIO0_B5 / RM_IO13 — active-low reset
- Pin 29 — `CAN1_TX` — GPIO1_B2 / RM_IO25 — second transceiver TXD
- Pin 31 — `CAN1_RX` — GPIO1_B3 / RM_IO26 — second transceiver RXD
- Pin 33 — `CAN0_TX` — GPIO1_C2 / RM_IO27 — first transceiver TXD
- Pin 37 — `CAN0_RX` — GPIO1_C3 / RM_IO28 — first transceiver RXD

## Physical 40-pin connector layout

Power and ground rails are shown directly. An em dash (`—`) means the remaining
signal pin is not claimed by the project-specific display, touch, or CAN
configuration.

Pin pair 1/2 is at the top of the connector and pin pair 39/40 is at the
bottom.

| Odd-side use | Odd pin |  | Even pin | Even-side use |
| --- | ---: | :---: | ---: | --- |
| 3.3 V | 1 | ↔ | 2 | 5 V |
| `TOUCH_SDA` | 3 | ↔ | 4 | 5 V |
| `TOUCH_SCL` | 5 | ↔ | 6 | GND |
| — | 7 | ↔ | 8 | — |
| GND | 9 | ↔ | 10 | — |
| `TOUCH_INT` | 11 | ↔ | 12 | — |
| `TOUCH_RST` | 13 | ↔ | 14 | GND |
| — | 15 | ↔ | 16 | `LCD_RST` |
| 3.3 V | 17 | ↔ | 18 | — |
| — | 19 | ↔ | 20 | GND |
| — | 21 | ↔ | 22 | — |
| — | 23 | ↔ | 24 | — |
| GND | 25 | ↔ | 26 | — |
| — | 27 | ↔ | 28 | — |
| `CAN1_TX` | 29 | ↔ | 30 | GND |
| `CAN1_RX` | 31 | ↔ | 32 | — |
| `CAN0_TX` | 33 | ↔ | 34 | GND |
| — | 35 | ↔ | 36 | — |
| `CAN0_RX` | 37 | ↔ | 38 | GND |
| — | 39 | ↔ | 40 | — |

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

## Source of truth

The assignments come from the selected board device tree and its CO6300 include:

- `kernel/arch/arm/boot/dts/rk3506b-luckfox-lyra-zero-w-co6300.dts`
- `kernel/arch/arm/boot/dts/rk3506-luckfox-lyra-co6300.dtsi`

Physical connector numbering was checked against the official Luckfox Lyra
Zero W schematic, connector `PI1`.
