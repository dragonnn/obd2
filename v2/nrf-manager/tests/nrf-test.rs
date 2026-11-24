#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use {cortex_m as _, cortex_m_rt as _, defmt_rtt as _, embassy_nrf as _};

#[embedded_test::tests]
mod tests {
    use embassy_time::{Duration, Timer, with_timeout};
    use embedded_can::{Frame as _, StandardId};
    use mcp251xfd::{Error, Frame, ReceivedFrame};
    use nrf_manager::board::Can;

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
