use core::sync::atomic::{AtomicI32, Ordering};

use defmt::*;
use embassy_time::{Duration, with_timeout};
use embedded_io_async::Write as _;
use heapless::Vec;

static TEMPERATURE: AtomicI32 = AtomicI32::new(0);

#[derive(Clone, Copy, PartialEq)]
struct TemperatureRange {
    offset: i8,
    register: u8,
    min: f32,
    max: f32,
}

// ESP-IDF's ESP32-C6 temperature_sensor_attributes table.
const TEMPERATURE_RANGES: [TemperatureRange; 5] = [
    TemperatureRange { offset: -2, register: 5, min: 50.0, max: 125.0 },
    TemperatureRange { offset: -1, register: 7, min: 20.0, max: 100.0 },
    TemperatureRange { offset: 0, register: 15, min: -10.0, max: 80.0 },
    TemperatureRange { offset: 1, register: 11, min: -30.0, max: 50.0 },
    TemperatureRange { offset: 2, register: 10, min: -40.0, max: 20.0 },
];

fn set_temperature_range(range: TemperatureRange) {
    // ESP32-C6 I2C_SARADC_TSENS_DAC is register 6, bits 3:0, on analog
    // I2C slave 0x69. This is the operation behind ESP-IDF's
    // temperature_sensor_ll_set_range().
    critical_section::with(|_| {
        let modem_lpcon = esp_hal::peripherals::MODEM_LPCON::regs();
        modem_lpcon.clk_conf().modify(|_, w| w.clk_i2c_mst_en().set_bit());
        modem_lpcon.i2c_mst_clk_conf().modify(|_, w| w.clk_i2c_mst_sel_160m().set_bit());

        let i2c = esp_hal::peripherals::I2C_ANA_MST::regs();
        let use_master_zero = i2c.ana_conf2().read().sar_i2c_mst_sel().bit_is_set();
        let master = usize::from(!use_master_zero);

        i2c.ana_conf1().write(|w| unsafe { w.bits(0x00ff_ffff).sar_i2c_rd().clear_bit() });
        while i2c.i2c_ctrl(master).read().busy().bit_is_set() {}
        i2c.i2c_ctrl(master).write(|w| unsafe { w.slave_addr().bits(0x69).slave_reg_addr().bits(6) });
        while i2c.i2c_ctrl(master).read().busy().bit_is_set() {}
        let old = i2c.i2c_ctrl(master).read().data().bits();

        i2c.i2c_ctrl(master).write(|w| unsafe {
            w.slave_addr()
                .bits(0x69)
                .slave_reg_addr()
                .bits(6)
                .read_write()
                .set_bit()
                .data()
                .bits((old & 0xf0) | range.register)
        });
        while i2c.i2c_ctrl(master).read().busy().bit_is_set() {}
    });
}

fn convert_temperature(raw: u8, offset: i8, calibration: f32) -> f32 {
    raw as f32 * 0.4386 - offset as f32 * 27.88 - 20.52 - calibration
}

fn select_temperature_range(temperature: f32) -> TemperatureRange {
    if temperature >= TEMPERATURE_RANGES[1].max {
        TEMPERATURE_RANGES[0]
    } else if temperature >= TEMPERATURE_RANGES[2].max {
        TEMPERATURE_RANGES[1]
    } else if temperature <= TEMPERATURE_RANGES[2].min && temperature > TEMPERATURE_RANGES[3].min {
        TEMPERATURE_RANGES[3]
    } else if temperature <= TEMPERATURE_RANGES[3].min {
        TEMPERATURE_RANGES[4]
    } else {
        TEMPERATURE_RANGES[2]
    }
}

/// Read the ESP32-C6 per-chip temperature correction from eFuse.
///
/// This mirrors ESP-IDF's `temperature_sensor_ll_load_calib_param`: the low
/// eight bits contain the magnitude in tenths of a degree and bit 8 is the
/// sign bit (set means negative).
fn temperature_calibration() -> f32 {
    let raw = esp_hal::peripherals::EFUSE::regs().rd_sys_part1_data4().read().sys_data_part1_4().bits() & 0x1ff;
    let magnitude = (raw & 0xff) as f32 / 10.0;

    if raw & 0x100 != 0 { -magnitude } else { magnitude }
}

#[embassy_executor::task]
pub async fn run(temperature: crate::types::TemperatureSensor) {
    let calibration = temperature_calibration();
    info!("temperature calibration: {:?} C", calibration);

    let mut range = TEMPERATURE_RANGES[2];
    set_temperature_range(range);
    embassy_time::Timer::after(Duration::from_micros(300)).await;

    loop {
        let reading = temperature.get_temperature();
        let uncalibrated = convert_temperature(reading.raw_value, range.offset, 0.0);
        let selected_range = select_temperature_range(uncalibrated);

        // ESP-IDF samples again after changing DAC range because the previous
        // raw value is only valid for the old range.
        let temp = if selected_range != range {
            range = selected_range;
            set_temperature_range(range);
            embassy_time::Timer::after(Duration::from_micros(300)).await;
            convert_temperature(temperature.get_temperature().raw_value, range.offset, calibration)
        } else {
            uncalibrated - calibration
        };

        TEMPERATURE.store((temp * 1000.0) as i32, Ordering::Relaxed);
        embassy_time::Timer::after(Duration::from_secs(1)).await;
    }
}

pub fn get_temperature() -> f32 {
    TEMPERATURE.load(Ordering::Relaxed) as f32 / 1000.0
}
