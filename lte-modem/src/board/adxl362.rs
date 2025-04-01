use bitfield_struct::bitfield;
use defmt::error;
use embassy_nrf::gpio::{AnyPin, Input, Pull};
use embedded_hal::spi::Operation;
use embedded_hal_async::spi::{SpiBus, SpiDevice};

const WRITE_REG: u8 = 0x0A;
const READ_REG: u8 = 0x0B;

const REG_DEVID_AD: u8 = 0x00;
const REG_DEVID_MST: u8 = 0x01;
const REG_PARTID: u8 = 0x02;
const REG_SOFT_RESET: u8 = 0x1F;

const REG_STATUS: u8 = 0x0B;
const REG_THRESHOLD_ACT: u8 = 0x20;
const REG_THRESH_INACT: u8 = 0x23;
const REG_TIME_INACT: u8 = 0x25;
const REG_ACT_INACT_CTL: u8 = 0x27;
const REG_INTMAP1: u8 = 0x2A;
const REG_INTMAP2: u8 = 0x2B;
const REG_POWER_CTL: u8 = 0x2D;

const VALUE_DEVID_AD: u8 = 0xAD;
const VALUE_DEVID_MST: u8 = 0x1D;
const VALUE_PARTID: u8 = 0xF2;
const VALUE_RESET: u8 = 0x52;

pub struct Adxl362<S> {
    spi_device: S,
    irq: Input<'static>,
}

#[bitfield(u8)]
pub struct ActInactCtl {
    act_enable: bool,
    act_ref: bool,
    inact_enable: bool,
    inact_ref: bool,
    #[bits(2)]
    mode: u8,
    #[bits(2)]
    _unused: u8,
}

#[repr(u8)]
pub enum ActInactCtlMode {
    Default = 0b00,
    Linked = 0b01,
    Loop = 0b11,
}

#[bitfield(u8)]
pub struct IntMap {
    data_ready: bool,
    fifo_ready: bool,
    fifo_watermark: bool,
    fifo_overrun: bool,
    act: bool,
    inact: bool,
    awake: bool,
    active_low: bool,
}

#[bitfield(u8)]
pub struct PowerCtl {
    #[bits(2)]
    mesure_mode: u8,
    autosleep: bool,
    wakeup: bool,
    #[bits(2)]
    low_noise_mode: u8,
    ext_clk: bool,
    #[bits(1)]
    _unused: u8,
}

#[repr(u8)]
pub enum PowerCtlMesureMode {
    Standby = 0b00,
    Measurement = 0b10,
}

#[repr(u8)]
pub enum PowerCtlLowNoiseMode {
    Normal = 0b00,
    Low = 0b01,
    Ultralow = 0b10,
}

#[bitfield(u8)]
pub struct Status {
    data_ready: bool,
    fifo_ready: bool,
    fifo_watermark: bool,
    fifo_overrun: bool,
    act: bool,
    inact: bool,
    awake: bool,
    err_user_regs: bool,
}

impl<S> Adxl362<S>
where
    S: SpiDevice,
{
    pub async fn new(spi_device: S, irq: AnyPin) -> Self {
        let irq = Input::new(irq, Pull::Up);
        let mut new = Self { spi_device, irq };
        new.reset().await;
        let dev_ad = new.read_one(REG_DEVID_AD).await;
        let dev_mst = new.read_one(REG_DEVID_MST).await;
        let partid = new.read_one(REG_PARTID).await;

        //assert_eq!(dev_ad, VALUE_DEVID_AD);
        //assert_eq!(dev_mst, VALUE_DEVID_MST);
        //assert_eq!(partid, VALUE_PARTID);
        if dev_ad != VALUE_DEVID_AD {
            error!("ADXL362: dev_ad: {:#X} != {:#X}", dev_ad, VALUE_DEVID_AD);
        }

        if dev_mst != VALUE_DEVID_MST {
            error!("ADXL362: dev_mst: {:#X} != {:#X}", dev_mst, VALUE_DEVID_MST);
        }

        if partid != VALUE_PARTID {
            error!("ADXL362: partid: {:#X} != {:#X}", partid, VALUE_PARTID);
        }

        new
    }

    async fn write_one(&mut self, reg: u8, value: u8) {
        let mut tx = [WRITE_REG, reg, value];
        self.spi_device.write(&tx).await.unwrap();
    }

    async fn write_two(&mut self, reg: u8, value: u16) {
        let value = value.to_le_bytes();
        let (value_l, value_h) = (value[0], value[1]);

        self.spi_device.write(&[WRITE_REG, reg, value_l, value_h]).await.unwrap();
    }

    async fn read_one(&mut self, reg: u8) -> u8 {
        let mut rx: [u8; 1] = [0; 1];
        let mut tx = [READ_REG, reg];
        self.spi_device.transaction(&mut [Operation::Write(&tx), Operation::Read(&mut rx)]).await.unwrap();
        rx[0]
    }

    pub async fn reset(&mut self) {
        self.write_one(REG_SOFT_RESET, VALUE_RESET).await;
        embassy_time::Timer::after(embassy_time::Duration::from_millis(150)).await;
    }

    pub async fn setup_montion_detection(&mut self, threshold_act: u16, threshold_inact: u16, samples: u8) {
        self.write_two(REG_THRESHOLD_ACT, threshold_act).await;
        self.write_two(REG_THRESH_INACT, threshold_inact).await;
        self.write_one(REG_TIME_INACT, samples).await;

        let act_inact_ctl = ActInactCtl::new()
            .with_act_enable(true)
            .with_act_ref(true)
            .with_inact_enable(true)
            .with_inact_ref(true)
            .with_mode(ActInactCtlMode::Linked as u8);

        let intmap_1 = IntMap::new().with_awake(true);

        let power_ctl = PowerCtl::new()
            .with_wakeup(true)
            .with_autosleep(true)
            .with_mesure_mode(PowerCtlMesureMode::Measurement as u8)
            .with_low_noise_mode(PowerCtlLowNoiseMode::Low as u8);

        self.write_one(REG_ACT_INACT_CTL, act_inact_ctl.into()).await;
        self.write_one(REG_INTMAP1, intmap_1.into()).await;
        self.write_one(REG_POWER_CTL, power_ctl.into()).await;
    }

    pub async fn montion_detection_irq(&mut self) -> bool {
        let mut status = Status::from(self.read_one(REG_STATUS).await);
        if status.act() {
            defmt::warn!("status act before irq wait for edge: {}", status.0);
            return true;
        }

        self.irq.wait_for_rising_edge().await;

        //Timer::after(Duration::from_millis(50)).await;

        status = Status::from(self.read_one(REG_STATUS).await);

        status.act()
    }
}
