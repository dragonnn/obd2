#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use {cortex_m as _, cortex_m_rt as _, defmt_rtt as _, embassy_nrf as _};

#[embedded_test::tests]
mod tests {
    #[test]
    fn hello_world() {
        defmt::info!("hello world");
    }

    #[test]
    async fn board() {
        let _board = nrf_manager::board::setup().await;
        defmt::info!("hello world");
    }
}
