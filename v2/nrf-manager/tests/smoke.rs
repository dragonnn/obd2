#![no_std]
#![no_main]

use {cortex_m as _, cortex_m_rt as _, defmt_rtt as _, embassy_nrf as _};

#[embedded_test::tests]
mod tests {
    #[test]
    fn hello_world() {
        defmt::info!("hello world");
    }
}
