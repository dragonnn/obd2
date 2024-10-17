use core::env;

use heapless::{String, Vec};
use nrf_modem::{DtlsSocket, Error as NrfError, PeerVerification};
use rmodbus::{client::ModbusRequest, ModbusProto};

use crate::{board::Modem, tasks};

const IDENTITY_WRITE_ADDRESS: u16 = 2441;

const IDENTITY_PLATFORM: u16 = 0xFFFF;

pub struct Identity {
    pub decoder_id: u16,
    pub imie: String<16>,
    pub iccid: String<22>,
    pub serial_number: String<16>,
    pub platform: u16,
    identity_reserve: [u16; 14],
}

pub async fn send_signle(modem: &Modem) -> Result<(), NrfError> {
    defmt::info!("sending single");

    let socket =
        DtlsSocket::connect(env!("SEND_HOST"), env!("SEND_PORT").parse().unwrap(), PeerVerification::Disabled, &[])
            .await?;

    let mut modbus = ModbusRequest::new(4, ModbusProto::Rtu);

    let mut buffer: Vec<u8, 256> = Vec::new();

    let request = modbus.generate_set_holdings_bulk(IDENTITY_WRITE_ADDRESS, &[120, 120], &mut buffer);

    //defmt::info!("modbus buffer: {:?}", buffer);

    socket.send(&buffer).await?;

    Ok(())
}

pub async fn send_loop(modem: &Modem) {}
