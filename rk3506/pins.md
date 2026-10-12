# Used and reserved 40-pin connector pins

This document covers the Luckfox Lyra Zero W 40-pin connector (`PI1` in the
schematic). Onboard-only signals and pins on the separate DSI connector are not
included.

PCB connections come from `LUCKFOX-LYRA-ZEROW.net` (U1). The symbol's GPIO and
RMIO names match the physical pin numbers in the official board schematic.

## Used and reserved pins

- Pin 3 — `TOUCH_SDA` — GPIO0_A0 — I2C2 SDA, 3.3 V host side
- Pin 5 — `TOUCH_SCL` — GPIO0_A1 — I2C2 SCL, 3.3 V host side
- Pin 7 — `CAN0_TX` — GPIO0_A2 — first transceiver TXD
- Pin 8 — `CAN0_RX` — GPIO0_C6 — first transceiver RXD
- Pin 10 — `CAN1_RX` — GPIO0_C7 — second transceiver RXD
- Pin 11 — `TOUCH_INT` — GPIO0_A3 — falling-edge interrupt
- Pin 12 — `CAN1_TX` — GPIO0_B6 — second transceiver TXD
- Pin 13 — `TOUCH_RST` — GPIO0_A4 — active-low reset
- Pin 15 — `LYRA_UART1_TX` — GPIO0_A5 — UART1 transmit (`/dev/ttyS1`)
- Pin 16 — `LYRA_UART1_RX` — GPIO0_B5 — UART1 receive (`/dev/ttyS1`)
- Pin 18 — `LYRA_SPI1_MISO` — GPIO0_B4 — SPI1 controller input
- Pin 19 — `LYRA_SPI1_SCLK` — GPIO0_A6 — SPI1 clock
- Pin 21 — `LYRA_SPI1_CS1` — GPIO0_A7 — SPI1 chip select 1, active low
- Pin 22 — `LYRA_SPI1_MOSI` — GPIO0_B3 — SPI1 controller output
- Pin 23 — `LYRA_UART0_TX` — GPIO0_B0 — UART2 transmit (`/dev/ttyS2`)
- Pin 24 — `LYRA_UART0_RX` — GPIO0_B2 — UART2 receive (`/dev/ttyS2`)
- Pin 28 — `LCD_RST` — GPIO1_D3 — active-low reset

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
| `CAN0_TX` | GPIO0_A2 | 7 | 8 | GPIO0_C6 | `CAN0_RX` |
| GND | GND | 9 | 10 | GPIO0_C7 | `CAN1_RX` |
| `TOUCH_INT` | GPIO0_A3 | 11 | 12 | GPIO0_B6 | `CAN1_TX` |
| `TOUCH_RST` | GPIO0_A4 | 13 | 14 | GND | GND |
| `LYRA_UART1_TX` (`/dev/ttyS1`) | GPIO0_A5 | 15 | 16 | GPIO0_B5 | `LYRA_UART1_RX` (`/dev/ttyS1`) |
| 3.3 V | 3.3 V | 17 | 18 | GPIO0_B4 | `LYRA_SPI1_MISO` |
| `LYRA_SPI1_SCLK` | GPIO0_A6 | 19 | 20 | GND | GND |
| `LYRA_SPI1_CS1` | GPIO0_A7 | 21 | 22 | GPIO0_B3 | `LYRA_SPI1_MOSI` |
| `LYRA_UART0_TX` (UART2, `/dev/ttyS2`) | GPIO0_B0 | 23 | 24 | GPIO0_B2 | `LYRA_UART0_RX` (UART2, `/dev/ttyS2`) |
| GND | GND | 25 | 26 | GPIO0_B1 | — |
| — | GPIO1_B1 | 27 | 28 | GPIO1_D3 | `LCD_RST` |
| — | GPIO1_B2 | 29 | 30 | GND | GND |
| — | GPIO1_B3 | 31 | 32 | GPIO1_D2 | — |
| — | GPIO1_C2 | 33 | 34 | GND | GND |
| — | GPIO0_C0 | 35 | 36 | GPIO1_D1 | — |
| — | GPIO1_C3 | 37 | 38 | GPIO0_C1 | — |
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

The UART pins are 3.3 V TTL logic. UART1 (`/dev/ttyS1`) uses RX pin 16 and
TX pin 15. Connect UART1 TX to the external device's RX,
UART1 RX to its TX, and use a common ground. Do not connect RS-232 voltage levels
or 5 V UART signals directly. The PCB nets named `LYRA_UART0_RX` and
`LYRA_UART0_TX` use UART2 (`/dev/ttyS2`): RX on pin 24 and TX on pin 23.
Both UART1 and UART2 have TX and RX DMA enabled. CAN0 and CAN1 receive DMA
use DMA1 to leave DMA0 capacity for SPI1 and both UARTs. UART0's fixed TX and RX
pins (8 and 10) carry CAN receive signals. The DDR loader, SPL, U-Boot, and Linux
disable the UART0 console and initialization to keep these pins available for CAN.

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
`rm_io5_uart1_tx` (GPIO0_A5, pin 15) and `rm_io13_uart1_rx` (GPIO0_B5,
pin 16). UART2 uses `rm_io8_uart2_tx` (GPIO0_B0, pin 23) and
`rm_io10_uart2_rx` (GPIO0_B2, pin 24). SPI1 uses `rm_io7_spi1_csn1`, `rm_io6_spi1_clk`,
`rm_io12_spi1_miso`, and `rm_io11_spi1_mosi`.

Boot UART settings are in `device/u-boot/rk3506b_lyra_can.config` and
`device/u-boot/RK3506BMINIALL-no-uart.ini`. `lyra-build` runs
`device/u-boot/sync-boot-uart.sh` to generate a UART-disabled DDR binary from the
original vendor binary, preserving the remaining DDR settings.

Physical connector numbering was checked against the [official Luckfox Lyra
Zero W schematic](https://github.com/LuckfoxTECH/Luckfox-Lyra-docs/blob/main/Hardware/Schematic/Luckfox-Lyra-Zero-W.pdf),
connector `PI1`.
