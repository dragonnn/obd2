# Used and reserved 40-pin connector pins

This document covers the Luckfox Lyra Zero W 40-pin connector (`PI1` in the
schematic). Onboard-only signals and pins on the separate DSI connector are not
included.

PCB connections come from `LUCKFOX-LYRA-ZEROW.net` (U1). Its symbol has incorrect
GPIO names on several even-numbered pins, so GPIO identities below use the
physical pin numbers and the official board schematic.

## Used and reserved pins

- Pin 3 — `TOUCH_SDA` — GPIO0_A0 — I2C2 SDA, 3.3 V host side
- Pin 5 — `TOUCH_SCL` — GPIO0_A1 — I2C2 SCL, 3.3 V host side
- Pin 11 — `TOUCH_INT` — GPIO0_A3 — falling-edge interrupt
- Pin 13 — `TOUCH_RST` — GPIO0_A4 — active-low reset
- Pin 18 — `SPI1_MOSI` — GPIO0_B4 — SPI1 controller output
- Pin 21 — `SPI1_CS1` — GPIO0_A7 — SPI1 chip select 1, active low
- Pin 22 — `SPI1_MISO` — GPIO0_B3 — SPI1 controller input
- Pin 23 — `SPI1_SCLK` — GPIO0_B0 — SPI1 clock
- Pin 26 — `UART1_RX` — GPIO0_B1 — UART1 receive (`/dev/ttyS1`)
- Pin 27 — `UART1_TX` — GPIO1_B1 — UART1 transmit (`/dev/ttyS1`)
- Pin 28 — `LCD_RST` — GPIO1_D3 — active-low reset
- Pin 29 — `CAN1_TX` — GPIO1_B2 — second transceiver TXD
- Pin 31 — `CAN1_RX` — GPIO1_B3 — second transceiver RXD
- Pin 33 — `CAN0_TX` — GPIO1_C2 — first transceiver TXD
- Pin 36 — `LYRA_UART0_RX` — GPIO1_D1 — UART2 receive (`/dev/ttyS2`)
- Pin 37 — `CAN0_RX` — GPIO1_C3 — first transceiver RXD
- Pin 38 — `LYRA_UART0_TX` — GPIO0_C1 — UART2 transmit (`/dev/ttyS2`)

## Physical 40-pin connector layout

Power and ground rails are shown directly. An em dash (`—`) means the remaining
signal pin is not claimed by the project-specific display, touch, CAN, UART,
or SPI configuration. The PCB debug UART pins retain their net names and use
UART2.

Pin pair 1/2 is at the top of the connector and pin pair 39/40 is at the
bottom.

| Odd-side use | Odd pin name | Odd pin | Even pin | Even pin name | Even-side use |
| --- | --- | ---: | ---: | --- | --- |
| 3.3 V | 3.3 V | 1 | 2 | 5 V | 5 V |
| `TOUCH_SDA` | GPIO0_A0 | 3 | 4 | 5 V | 5 V |
| `TOUCH_SCL` | GPIO0_A1 | 5 | 6 | GND | GND |
| — | GPIO0_A2 | 7 | 8 | GPIO0_C6 | — |
| GND | GND | 9 | 10 | GPIO0_C7 | — |
| `TOUCH_INT` | GPIO0_A3 | 11 | 12 | GPIO0_B6 | — |
| `TOUCH_RST` | GPIO0_A4 | 13 | 14 | GND | GND |
| — | GPIO0_A5 | 15 | 16 | GPIO0_B5 | — |
| 3.3 V | 3.3 V | 17 | 18 | GPIO0_B4 | `SPI1_MOSI` |
| — | GPIO0_A6 | 19 | 20 | GND | GND |
| `SPI1_CS1` | GPIO0_A7 | 21 | 22 | GPIO0_B3 | `SPI1_MISO` |
| `SPI1_SCLK` | GPIO0_B0 | 23 | 24 | GPIO0_B2 | — |
| GND | GND | 25 | 26 | GPIO0_B1 | `UART1_RX` (`/dev/ttyS1`) |
| `UART1_TX` (`/dev/ttyS1`) | GPIO1_B1 | 27 | 28 | GPIO1_D3 | `LCD_RST` |
| `CAN1_TX` | GPIO1_B2 | 29 | 30 | GND | GND |
| `CAN1_RX` | GPIO1_B3 | 31 | 32 | GPIO1_D2 | — |
| `CAN0_TX` | GPIO1_C2 | 33 | 34 | GND | GND |
| — | GPIO0_C0 | 35 | 36 | GPIO1_D1 | `LYRA_UART0_RX` (UART2 RX, `/dev/ttyS2`) |
| `CAN0_RX` | GPIO1_C3 | 37 | 38 | GPIO0_C1 | `LYRA_UART0_TX` (UART2 TX, `/dev/ttyS2`) |
| GND | GND | 39 | 40 | GPIO0_C2 | — |

## Power and ground

The following connector pins are power rails or ground rather than software
controlled signals:

- 3.3 V: pins 1 and 17
- 5 V: pins 2 and 4
- Ground: pins 6, 9, 14, 20, 25, 30, 34, and 39

Use a common ground between the Lyra and each external CAN transceiver. The CAN
controller pins are 3.3 V single-ended logic; never connect them directly to
CAN-H or CAN-L. The touch entries describe the adapter/host side. The bare
panel touch signals use different voltage levels and must not be wired directly
from this table without the appropriate adapter or level translation.

The UART pins are 3.3 V TTL logic. UART1 (`/dev/ttyS1`) uses RX pin 26 and
TX pin 27. Connect UART1 TX to the external device's RX,
UART1 RX to its TX, and use a common ground. Do not connect RS-232 voltage levels
or 5 V UART signals directly. The PCB nets named `LYRA_UART0_RX` and
`LYRA_UART0_TX` use UART2 (`/dev/ttyS2`): RX on pin 36 and TX on pin 38.
Both UART1 and UART2 have TX and RX DMA enabled. CAN0 and CAN1 receive DMA
use DMA1 to leave DMA0 capacity for SPI1 and both UARTs. UART0 remains disabled;
its fixed RX and TX pins are 10 and 8.

SPI1 is enabled as a controller with chip select 1. Its userspace device is
`/dev/spidev1.1` (10 MHz maximum configured in the device tree). Connect
`SPI1_SCLK` to the peripheral clock, `SPI1_MOSI` to its input, `SPI1_MISO` to
its output, and `SPI1_CS1` to its active-low select. Use a common ground and
3.3 V logic.

## Source of truth

The assignments come from the selected board device tree and its CO6300 include:

- `kernel/arch/arm/boot/dts/rk3506b-luckfox-lyra-zero-w-co6300.dts`
- `kernel/arch/arm/boot/dts/rk3506-luckfox-lyra-co6300.dtsi`

The UART and SPI1 matrix mux mappings are defined by the SDK's
`sdk/kernel/arch/arm/boot/dts/rk3506-pinctrl-rmio.dtsi`. UART1 uses
`rm_io24_uart1_tx` (GPIO1_B1, pin 27) and `rm_io9_uart1_rx` (GPIO0_B1,
pin 26). UART2 uses `rm_io17_uart2_tx` (GPIO0_C1, pin 38) and
`rm_io29_uart2_rx` (GPIO1_D1, pin 36). SPI1 uses `rm_io7_spi1_csn1`, `rm_io8_spi1_clk`,
`rm_io11_spi1_miso`, and `rm_io12_spi1_mosi`.

Physical connector numbering was checked against the [official Luckfox Lyra
Zero W schematic](https://github.com/LuckfoxTECH/Luckfox-Lyra-docs/blob/main/Hardware/Schematic/Luckfox-Lyra-Zero-W.pdf),
connector `PI1`.
