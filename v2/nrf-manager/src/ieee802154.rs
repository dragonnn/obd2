bind_interrupts!(struct Ieee802154Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

pub trait TryIeee802154Send {
    fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> impl core::future::Future<Output = Result<(), embassy_nrf::radio::Error>>;

    fn try_send_raw(
        &mut self,
        packet: &mut radio::ieee802154::Packet,
    ) -> impl core::future::Future<Output = Result<(), embassy_nrf::radio::Error>>;
}

impl TryIeee802154Send for embassy_nrf::radio::ieee802154::Radio<'_> {
    async fn try_send_raw(
        &mut self,
        tx_packet: &mut radio::ieee802154::Packet,
    ) -> Result<(), embassy_nrf::radio::Error> {
        self.try_send(tx_packet).await?;
        Ok(())
    }
    async fn try_send_buffer(
        &mut self,
        packet: &[u8],
        seq_number: &mut u8,
    ) -> Result<(), embassy_nrf::radio::Error> {
        let chunks = packet.chunks(100);
        let chunks_count = chunks.len();
        warn!("Chunks count: {}", chunks_count);
        for (c, chunk) in chunks.enumerate() {
            let frame = ieee802154::mac::Frame {
                header: ieee802154::mac::Header {
                    frame_type: ieee802154::mac::FrameType::Data,
                    frame_pending: false,
                    ack_request: false,
                    pan_id_compress: false,
                    seq_no_suppress: false,
                    ie_present: false,
                    version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                    seq: *seq_number,
                    destination: Some(ieee802154::mac::Address::Short(
                        ieee802154::mac::PanId(chunks_count as u16),
                        ieee802154::mac::ShortAddress(c as u16),
                    )),
                    source: Some(ieee802154::mac::Address::Short(
                        ieee802154::mac::PanId(0x2223),
                        ieee802154::mac::ShortAddress(0x2223),
                    )),
                    auxiliary_security_header: None,
                },
                payload: chunk,
                footer: [0, 0],
                content: ieee802154::mac::FrameContent::Data,
            };
            let mut radio_tx_packet = radio::ieee802154::Packet::new();
            let mut tx_packet = [0; 256];
            match frame.try_write(
                &mut tx_packet,
                &mut FrameSerDesContext::no_security(FooterMode::Explicit),
            ) {
                Ok(res) => {
                    radio_tx_packet.copy_from_slice(&tx_packet[0..res]);
                    info!("radio_tx_packet.len(): {}", radio_tx_packet.len());
                    self.try_send_raw(&mut radio_tx_packet).await?;
                }
                Err(err) => {
                    error!("error writing frame: {:?}", defmt::Debug2Format(&err));
                }
            }
            *seq_number = seq_number.wrapping_add(1);
        }
        Ok(())
    }
}
