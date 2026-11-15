//! nRF52840 (U2) connections on the LUCKFOX-LYRA-ZEROW PCB.
//!
//! SWDCLK/SWDIO remain dedicated to debugging. Unconnected GPIOs are omitted.

use embassy_nrf::{
    Peri, Peripherals, bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals,
    spim::{self, Spim},
    uarte::{self, Uarte},
};
use embassy_time::Timer;

bind_interrupts!(struct UartIrqs {
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
    UARTE1 => uarte::InterruptHandler<peripherals::UARTE1>;
});

bind_interrupts!(struct SpiIrqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
});

pub struct Board {
    pub pins: Pins,
    pub spi: Spim<'static>,        // SPI3, SCK P0.19 / MISO P0.23 / MOSI P0.21
    pub lyra_uart: Uarte<'static>, // UARTE0, TX P0.22 / RX P1.00, Lyra via U9
    pub ext_uart: Uarte<'static>,  // UARTE1, TX P0.04 / RX P0.05, J4.2 / J4.3
    pub radio: Peri<'static, peripherals::RADIO>,
    pub wdt: Peri<'static, peripherals::WDT>,
}

/// Keep these pins alive: dropping a GPIO driver resets its configuration.
/// I2C and ADC pins retain their types for use with peripheral drivers.
#[allow(dead_code)]
pub struct Pins {
    pub can1_shdn: Output<'static>, // P1.15, NRF_CAN1_SHDN, U6 MAX3051
    pub can2_shdn: Output<'static>, // P0.28, NRF_CAN2_SHDN, U8 MAX3051
    pub can1_int: Input<'static>,   // P1.10, NRF_CAN1_INT, U16 MCP2518FD
    pub can2_int: Input<'static>,   // P1.13, NRF_CAN2_INT, U15 MCP2518FD
    pub cs1_can1: Output<'static>,  // P0.07, NRF_CS1_CAN1
    pub cs2_can2: Output<'static>,  // P0.18, NRF_CS2_CAN2 (reset-pin-as-gpio)
    pub cs3_eink: Output<'static>,  // P1.11, NRF_CS3_EINK
    pub eink_dc: Output<'static>,   // P0.02, NRF_EINK_DC
    pub rst: Output<'static>,       // P0.29, NRF_RST

    pub spi_cs: Output<'static>,  // P0.31, NRF_SPI_CS, J6.3
    pub spi_irq: Input<'static>,  // P1.04, NRF_SPI_IRQ, J6.2
    pub spi_rst: Output<'static>, // P1.06, NRF_SPI_RST, J6.1

    pub scl: Peri<'static, peripherals::P0_12>, // NRF_SCL, J2.2
    pub sda: Peri<'static, peripherals::P1_09>, // NRF_SDA, J2.3
    pub i2c_irq: Input<'static>,                // P0.30, NRF_I2C_IRQ, J2.1

    pub ext_uart_irq: Input<'static>, // P1.03, NRF_UART2_IRQ, J4.1

    pub ignition: Peri<'static, peripherals::P0_03>, // NRF_ING12V+, R13/R14 divider
    pub lyra_power: LyraPower,
}

/// Lyra regulator enable latched by U11 (SN74LVC1G74) on CLK's rising edge.
pub struct LyraPower {
    pwr_clk: Output<'static>,   // P1.02, NRF_PWR_CLK, U11 CLK
    pwr_state: Output<'static>, // P1.01, NRF_PWR_STATE, U11 D
}

impl LyraPower {
    pub async fn power_up(&mut self) {
        self.set_power(Level::High).await;
    }

    pub async fn power_down(&mut self) {
        self.set_power(Level::Low).await;
    }

    async fn set_power(&mut self, state: Level) {
        self.pwr_state.set_level(state);
        Timer::after_millis(5).await;
        self.pwr_clk.set_low();
        // Allow data setup time before the rising edge, then hold it through
        // the clock pulse. Both pins remain driven after the latch updates.
        Timer::after_millis(5).await;
        self.pwr_clk.set_high();
        Timer::after_millis(5).await;
        self.pwr_clk.set_low();
    }
}

/// Map all 26 connected GPIOs and establish idle control levels before any await.
pub fn setup(p: Peripherals) -> Board {
    // MAX3051 shutdown is active high. Set these first and retain both drivers.
    // U3/U5 SHDN are grounded; they shut down with the switched regulator rail.
    let can1_shdn = Output::new(p.P1_15, Level::High, OutputDrive::Standard);
    let can2_shdn = Output::new(p.P0_28, Level::High, OutputDrive::Standard);

    // U11 latches D on CLK's rising edge. Start CLK low without clocking it.
    let pwr_clk = Output::new(p.P1_02, Level::Low, OutputDrive::Standard);
    let pwr_state = Output::new(p.P1_01, Level::Low, OutputDrive::Standard);

    let mut uart_config = uarte::Config::default();
    uart_config.baudrate = uarte::Baudrate::Baud115200;
    uart_config.parity = uarte::Parity::Excluded;

    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M1;
    spi_config.mode = spim::MODE_0;
    spi_config.bit_order = spim::BitOrder::MsbFirst;

    Board {
        pins: Pins {
            can1_shdn,
            can2_shdn,
            can1_int: Input::new(p.P1_10, Pull::Up),
            can2_int: Input::new(p.P1_13, Pull::Up),
            cs1_can1: Output::new(p.P0_07, Level::High, OutputDrive::Standard),
            cs2_can2: Output::new(p.P0_18, Level::High, OutputDrive::Standard),
            cs3_eink: Output::new(p.P1_11, Level::High, OutputDrive::Standard),
            eink_dc: Output::new(p.P0_02, Level::Low, OutputDrive::Standard),
            rst: Output::new(p.P0_29, Level::High, OutputDrive::Standard),
            spi_cs: Output::new(p.P0_31, Level::High, OutputDrive::Standard),
            spi_irq: Input::new(p.P1_04, Pull::Up),
            spi_rst: Output::new(p.P1_06, Level::High, OutputDrive::Standard),
            scl: p.P0_12,
            sda: p.P1_09,
            i2c_irq: Input::new(p.P0_30, Pull::Up),
            ext_uart_irq: Input::new(p.P1_03, Pull::Up),
            ignition: p.P0_03,
            lyra_power: LyraPower { pwr_clk, pwr_state },
        },
        spi: Spim::new(p.SPI3, SpiIrqs, p.P0_19, p.P0_23, p.P0_21, spi_config),
        lyra_uart: Uarte::new(p.UARTE0, p.P1_00, p.P0_22, UartIrqs, uart_config.clone()),
        ext_uart: Uarte::new(p.UARTE1, p.P0_05, p.P0_04, UartIrqs, uart_config),
        radio: p.RADIO,
        wdt: p.WDT,
    }
}
