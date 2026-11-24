#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use {cortex_m as _, cortex_m_rt as _, defmt_rtt as _, embassy_nrf as _};

#[embedded_test::tests]
mod tests {
    use embassy_nrf::{
        Peri,
        gpio::{Level, Output, OutputDrive, Pin},
        peripherals::{P0_31, P1_06},
    };
    use embassy_time::{Duration, Instant, Timer, with_timeout};
    use embedded_can::{Frame as _, StandardId};
    use epdsi::graphics::buffer::{DisplayRotation, PageBuffer};
    use mcp251xfd::{Error, Frame, ReceivedFrame};
    use nrf_manager::board::{self, Can, eink};

    #[test]
    fn hello_world() {
        defmt::info!("hello world");
    }

    #[test]
    async fn board() {
        let _board = nrf_manager::board::setup().await;
        defmt::info!("hello world");
    }

    #[test]
    #[timeout(60)]
    async fn eink_hello_world() {
        let mut board = board::setup().await;
        let display = &mut board.eink;
        let message = [
            "#   # ##### #     #      ###      #   #  ###  ####  #     ####   #",
            "#   # #     #     #     #   #     #   # #   # #   # #     #   #  #",
            "#   # #     #     #     #   #     #   # #   # #   # #     #   #  #",
            "##### ####  #     #     #   #     # # # #   # ####  #     #   #  #",
            "#   # #     #     #     #   #     # # # #   # # #   #     #   #  #",
            "#   # #     #     #     #   #     ## ## #   # #  #  #     #   #   ",
            "#   # ##### ##### #####  ###      #   #  ###  #   # ##### ####   #",
        ];
        let scale = 3;
        let left = (eink::HEIGHT - message[0].len() as u32 * scale) / 2;
        let top = (eink::WIDTH - message.len() as u32 * scale) / 2;
        let mut frame = [0xFF; eink::FRAME_BYTES];
        let mut canvas = PageBuffer::new(&mut frame, eink::WIDTH, eink::HEIGHT, 0);
        canvas.set_rotation(DisplayRotation::Rotate90);
        for (row, pixels) in message.iter().enumerate() {
            for (column, pixel) in pixels.bytes().enumerate() {
                if pixel == b'#' {
                    let x = left + column as u32 * scale;
                    let y = top + row as u32 * scale;
                    for dy in 0..scale {
                        for dx in 0..scale {
                            canvas.set_pixel(x + dx, y + dy, true);
                        }
                    }
                }
            }
        }

        defmt::info!("Initializing e-ink panel");
        defmt::unwrap!(display.init().await, "E-ink init failed");
        defmt::info!("Refreshing e-ink panel");
        let refresh_started = Instant::now();
        defmt::unwrap!(display.display_frame(&frame).await, "E-ink refresh failed");
        let refresh_ms = refresh_started.elapsed().as_millis();
        // A full refresh takes seconds. A quick SPI write alone can succeed with BUSY stuck low.
        defmt::assert!(
            refresh_ms >= 100,
            "E-ink refresh ended in {} ms",
            refresh_ms
        );
        defmt::unwrap!(display.sleep().await, "E-ink sleep failed");
        defmt::info!("Check the e-ink panel for HELLO WORLD!");
    }

    #[test]
    #[ignore]
    #[timeout(30)]
    async fn eink_dc_high() {
        let p = embassy_nrf::init(Default::default());
        let _can_listen_cs = Output::new(p.P0_18, Level::High, OutputDrive::HighDrive);
        let _can_obd2_cs = Output::new(p.P0_07, Level::High, OutputDrive::HighDrive);
        cortex_m::asm::delay(128);
        hold_eink_pin_high(p.P0_23, p.P0_31, p.P1_06, "DC (P0.23 / J6.4)").await;
    }

    #[test]
    #[ignore]
    #[timeout(30)]
    async fn mosi_high() {
        let p = embassy_nrf::init(Default::default());
        hold_eink_pin_high(p.P0_21, p.P0_31, p.P1_06, "DIN (P0.21 / J6.5)").await;
    }

    #[test]
    #[ignore]
    #[timeout(30)]
    async fn sck_high() {
        let p = embassy_nrf::init(Default::default());
        hold_eink_pin_high(p.P0_19, p.P0_31, p.P1_06, "CLK (P0.19 / J6.6)").await;
    }

    async fn hold_eink_pin_high(
        pin: Peri<'static, impl Pin>,
        cs: Peri<'static, P0_31>,
        rst: Peri<'static, P1_06>,
        label: &str,
    ) {
        let _cs = Output::new(cs, Level::High, OutputDrive::HighDrive);
        let _rst = Output::new(rst, Level::High, OutputDrive::Standard);
        let signal = Output::new(pin, Level::High, OutputDrive::HighDrive);

        defmt::info!(
            "{} held high for 20 seconds. Measure at the HAT to GND.",
            label
        );
        Timer::after_secs(20).await;
        drop(signal);
        defmt::info!("Measurement complete. {} released.", label);
    }

    #[test]
    #[timeout(20)]
    async fn can() {
        let mut board = nrf_manager::board::setup().await;
        defmt::unwrap!(board.can_listen.wake().await, "CAN LISTEN wake failed");
        defmt::unwrap!(board.can_obd2.wake().await, "OBD2 CAN wake failed");

        let listen_to_obd2 = transfer(
            &mut board.can_listen,
            &mut board.can_obd2,
            "LISTEN -> OBD2",
            defmt::unwrap!(Frame::new(StandardId::new(0x123).unwrap(), b"LISTEN")),
        )
        .await;
        let obd2_to_listen = transfer(
            &mut board.can_obd2,
            &mut board.can_listen,
            "OBD2 -> LISTEN",
            defmt::unwrap!(Frame::new(StandardId::new(0x321).unwrap(), b"OBD2")),
        )
        .await;

        let listen_sleep = board.can_listen.sleep().await;
        let obd2_sleep = board.can_obd2.sleep().await;
        defmt::unwrap!(listen_sleep, "CAN LISTEN sleep failed");
        defmt::unwrap!(obd2_sleep, "OBD2 CAN sleep failed");
        defmt::assert!(listen_to_obd2 && obd2_to_listen, "CAN transfer failed");
    }

    async fn transfer(sender: &mut Can, receiver: &mut Can, direction: &str, frame: Frame) -> bool {
        if let Err(error) = sender.transmit(&frame).await {
            defmt::error!("{} transmit failed: {:?}", direction, error);
            return false;
        }

        let received = with_timeout(Duration::from_millis(200), async {
            loop {
                match receiver.receive().await {
                    Err(Error::RxFifoEmpty) => Timer::after_millis(1).await,
                    result => return result,
                }
            }
        })
        .await;

        match received {
            Ok(Ok(rx)) if rx.frame == ReceivedFrame::Classic(frame) => {
                defmt::info!("{} received {:?}", direction, frame);
                true
            }
            Ok(Ok(rx)) => {
                defmt::error!(
                    "{} expected {:?}, received {:?}",
                    direction,
                    frame,
                    rx.frame
                );
                false
            }
            Ok(Err(error)) => {
                defmt::error!("{} receive failed: {:?}", direction, error);
                false
            }
            Err(_) => {
                defmt::error!("{} timed out waiting for {:?}", direction, frame);
                false
            }
        }
    }
}
