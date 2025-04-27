#![feature(prelude_import)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trivial_bounds)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use embassy_executor::Executor;
use static_cell::StaticCell;
extern crate alloc;
static COUNT: AtomicUsize = AtomicUsize::new(0);
const _: () = {
    #[export_name = "_defmt_timestamp"]
    #[inline(never)]
    fn defmt_timestamp(fmt: defmt::Formatter<'_>) {
        match (&(COUNT.fetch_add(1, Ordering::Relaxed))) {
            (arg0) => {
                defmt::export::usize(arg0);
            }
        }
    }
    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_timestamp\",\"data\":\"{=usize}\",\"disambiguator\":\"5398398130257944791\",\"crate_name\":\"display\"}"]
    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_timestamp\",\"data\":\"{=usize}\",\"disambiguator\":\"5398398130257944791\",\"crate_name\":\"display\"}"]
    static S: u8 = 0;;
    #[no_mangle]
    #[link_section = ".defmt.end.timestamp"]
    static __DEFMT_MARKER_TIMESTAMP_WAS_DEFINED: &u8 = &S;
};
pub mod lcd {
    use defmt::*;
    use embassy_futures::select::{select, Either::*};
    use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
    use embassy_time::{with_timeout, Duration, Timer};
    use embedded_graphics::geometry::{Point, Size};
    use heapless::String;
    use statig::prelude::*;
    use types::Pid as Obd2Event;
    use crate::{
        debug::DEBUG_STRING_LEN,
        display::widgets::{Battery, BatteryOrientation, DebugScroll},
        tasks::obd2::{obd2_init_wait, Obd2Debug},
        types::{Display1, Display2},
    };
    mod boot {
        use defmt::{info, unwrap};
        use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
        use embassy_time::Instant;
        use heapless::FnvIndexMap;
        use crate::{
            display::widgets::DebugScroll,
            types::{Display1, Display2},
        };
        pub enum BootUp {
            Buttons,
            Obd2,
            CanListen,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BootUp {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        BootUp::Buttons => "Buttons",
                        BootUp::Obd2 => "Obd2",
                        BootUp::CanListen => "CanListen",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BootUp {
            #[inline]
            fn clone(&self) -> BootUp {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for BootUp {}
        pub struct LcdBootState {
            bootup_state: FnvIndexMap<BootUp, bool, 3>,
        }
        #[automatically_derived]
        impl ::core::default::Default for LcdBootState {
            #[inline]
            fn default() -> LcdBootState {
                LcdBootState {
                    bootup_state: ::core::default::Default::default(),
                }
            }
        }
        impl LcdBootState {
            pub fn new() -> Self {
                Self {
                    bootup_state: FnvIndexMap::new(),
                }
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::boot".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"12679748094058582176\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"12679748094058582176\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::boot".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"16635703411121821976\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"16635703411121821976\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
    }
    mod debug {
        use defmt::{info, unwrap};
        use crate::{
            display::widgets::DebugScroll,
            types::{Display1, Display2},
        };
        pub struct LcdDebugState {
            debug: DebugScroll,
        }
        #[automatically_derived]
        impl ::core::default::Default for LcdDebugState {
            #[inline]
            fn default() -> LcdDebugState {
                LcdDebugState {
                    debug: ::core::default::Default::default(),
                }
            }
        }
        impl LcdDebugState {
            pub fn new() -> Self {
                Self {
                    debug: DebugScroll::new(),
                }
            }
            pub fn add_line(&mut self, line: &str) {
                self.debug.add_line(line);
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                self.debug.draw(display1, display2);
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::debug".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"462742691240498155\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"462742691240498155\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::debug".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"5455156759315764712\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"5455156759315764712\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
    }
    mod main {
        use defmt::{info, unwrap, warn};
        use embedded_graphics::geometry::{Point, Size};
        use types::{BmsPid, IceTemperaturePid, Pid as Obd2Event};
        use crate::{
            display::widgets::{
                Arrow, ArrowDirection, Battery, Battery12V, BatteryOrientation, Connection,
                GearboxGear, IceFuelRate, Icon, MotorElectric, MotorIce, Position, Power,
                Temperature, Value,
            },
            tasks::ieee802154::{last_position, last_receive, last_send},
            types::{Display1, Display2},
        };
        pub struct LcdMainState {
            hv_battery: Battery,
            aux_battery: Battery12V,
            ice_temperature: Temperature,
            ice_fuel_rate: IceFuelRate,
            electric_power: Power,
            electric_power_arrow: Arrow,
            motor_electric: MotorElectric,
            motor_electric_rpm: Value,
            motor_ice: MotorIce,
            gearbox_gear: GearboxGear,
            vehicle_speed: Value,
            connection: Connection,
            position: Position,
            ac_compressor: Icon<embedded_iconoir::icons::size18px::weather::SnowFlake>,
            ice_fuel_rate_value: f32,
            hv_battery_current: f32,
            vehicle_speed_value: f32,
        }
        #[automatically_derived]
        impl ::core::default::Default for LcdMainState {
            #[inline]
            fn default() -> LcdMainState {
                LcdMainState {
                    hv_battery: ::core::default::Default::default(),
                    aux_battery: ::core::default::Default::default(),
                    ice_temperature: ::core::default::Default::default(),
                    ice_fuel_rate: ::core::default::Default::default(),
                    electric_power: ::core::default::Default::default(),
                    electric_power_arrow: ::core::default::Default::default(),
                    motor_electric: ::core::default::Default::default(),
                    motor_electric_rpm: ::core::default::Default::default(),
                    motor_ice: ::core::default::Default::default(),
                    gearbox_gear: ::core::default::Default::default(),
                    vehicle_speed: ::core::default::Default::default(),
                    connection: ::core::default::Default::default(),
                    position: ::core::default::Default::default(),
                    ac_compressor: ::core::default::Default::default(),
                    ice_fuel_rate_value: ::core::default::Default::default(),
                    hv_battery_current: ::core::default::Default::default(),
                    vehicle_speed_value: ::core::default::Default::default(),
                }
            }
        }
        impl LcdMainState {
            pub fn new() -> Self {
                match () {
                    () => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd::main".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"LcdMainState::new()\",\"disambiguator\":\"11318265972515252832\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"LcdMainState::new()\",\"disambiguator\":\"11318265972515252832\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                Self {
                    hv_battery: Battery::new(
                        Point::new(9, 1),
                        Size::new(128, 62),
                        BatteryOrientation::HorizontalRight,
                        Some(Size::new(8, 32)),
                        4,
                        true,
                    ),
                    aux_battery: Battery12V::new(Point::new(256 - 18 * 2 - 16 * 2 - 6, 31)),
                    ice_temperature: Temperature::new(
                        Point::new(256 - 18 * 2 - 4, 0),
                        Size::new(16, 64),
                        0.0,
                        130.0,
                        4,
                    ),
                    electric_power: Power::new(Point::new(128 + 36, 14)),
                    electric_power_arrow: Arrow::new(
                        Point {
                            x: 9 + 128,
                            y: 64 / 2 - 9,
                        },
                        Size {
                            width: 54,
                            height: 16,
                        },
                        14,
                        ArrowDirection::Reverse,
                    ),
                    motor_electric: MotorElectric::new(Point::new(256 - 60, 0)),
                    motor_electric_rpm: Value::new(
                        Point::new(128 + 12, 55),
                        &profont::PROFONT_10_POINT,
                        "rpm",
                        0,
                    ),
                    motor_ice: MotorIce::new(Point::new(0, 0)),
                    gearbox_gear: GearboxGear::new(Point::new(40, 14)),
                    vehicle_speed: Value::new(
                        Point::new(58, 12),
                        &profont::PROFONT_14_POINT,
                        "km/h",
                        0,
                    ),
                    ice_fuel_rate: IceFuelRate::new(Point::new(60, 24)),
                    connection: Connection::new(Point::new(256 - 18, 0)),
                    position: Position::new(Point::new(256 - 18, 18)),
                    ac_compressor: Icon::new(Point::new(256 - 18, 18 + 18), false),
                    ice_fuel_rate_value: 0.0,
                    hv_battery_current: 0.0,
                    vehicle_speed_value: 0.0,
                }
            }
            pub fn handle_obd2_event(&mut self, event: &Obd2Event) {
                match event {
                    Obd2Event::BmsPid(bms_pid) => {
                        self.update_bms_pid(bms_pid);
                    }
                    Obd2Event::IceTemperaturePid(ice_temperature_pid) => {
                        self.ice_temperature
                            .update_temp(ice_temperature_pid.temperature);
                    }
                    Obd2Event::IceFuelRatePid(ice_fuel_rate_pid) => {
                        self.ice_fuel_rate_value = ice_fuel_rate_pid.fuel_rate;
                        self.ice_fuel_rate
                            .update_ice_fuel_rate(ice_fuel_rate_pid.fuel_rate);
                    }
                    Obd2Event::VehicleSpeedPid(vehicle_speed_pid) => {
                        let speed = vehicle_speed_pid.vehicle_speed as f32;
                        self.vehicle_speed.update_value(speed + speed * 0.1);
                        self.ice_fuel_rate.update_vehicle_speed(speed);
                        self.vehicle_speed_value = speed;
                    }
                    Obd2Event::TransaxlePid(transaxle_pid) => {
                        self.gearbox_gear.update_gear(transaxle_pid.gear.into());
                    }
                    Obd2Event::AcPid(ac_pid) => {
                        self.ac_compressor.enabled(ac_pid.compressor_on);
                    }
                    _ => {}
                }
            }
            pub fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
                self.hv_battery.update_percentage(bms_pid.hv_soc);
                self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
                self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
                self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
                self.hv_battery.update_cell_voltage(
                    (bms_pid.hv_max_cell_voltage + bms_pid.hv_min_cell_voltage) / 2.0,
                );
                self.hv_battery.update_cell_voltage_deviation(
                    (bms_pid.hv_max_cell_voltage - bms_pid.hv_min_cell_voltage) * 100.0,
                );
                self.aux_battery.update_voltage(bms_pid.aux_dc_voltage);
                self.electric_power_arrow.update_speed(50.0);
                self.electric_power
                    .update_power(bms_pid.hv_battery_current * bms_pid.hv_dc_voltage);
                self.electric_power
                    .update_current(bms_pid.hv_battery_current);
                if bms_pid.hv_battery_current > 0.0 {
                    self.electric_power_arrow
                        .update_direction(ArrowDirection::Forward);
                } else {
                    self.electric_power_arrow
                        .update_direction(ArrowDirection::Reverse);
                }
                self.hv_battery_current = bms_pid.hv_battery_current;
                self.motor_electric_rpm
                    .update_value(bms_pid.motor_electric_rpm);
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                if let Some(last_send) = last_send() {
                    self.connection
                        .update_last_send(last_send.elapsed().as_millis() < 250);
                }
                if let Some(last_receive) = last_receive() {
                    self.connection
                        .update_last_receive(last_receive.elapsed().as_millis() < 250);
                }
                if let Some(last_position) = last_position() {
                    self.position
                        .update_last_position(last_position.elapsed().as_millis() < 250);
                }
                if self.motor_ice.update_on(self.ice_fuel_rate_value > 0.0) {
                    self.gearbox_gear.force_redraw();
                }
                self.motor_electric
                    .update_on(if self.ice_fuel_rate_value == 0.0 {
                        true
                    } else {
                        self.hv_battery_current > 0.0
                    });
                self.hv_battery.draw(display1).ok();
                self.aux_battery.draw(display2).ok();
                self.ice_temperature.draw(display2).ok();
                self.motor_electric.draw(display1).ok();
                self.motor_ice.draw(display2).ok();
                self.electric_power.draw(display1).ok();
                self.electric_power_arrow.draw(display1).ok();
                self.gearbox_gear.draw(display2).ok();
                self.ice_fuel_rate.draw(display2).ok();
                self.vehicle_speed.draw(display2).ok();
                self.motor_electric_rpm.draw(display1).ok();
                self.connection.draw(display2).ok();
                self.position.draw(display2).ok();
                self.ac_compressor.draw(display2).ok();
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::main".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"406523074915118856\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"406523074915118856\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::main".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"14246366442303142053\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"14246366442303142053\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
    }
    mod menu {
        use defmt::{info, unwrap};
        use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
        use embedded_iconoir::prelude::*;
        use statig::Response::{self, Transition};
        use super::State;
        use crate::{
            display::widgets::DebugScroll,
            tasks::{
                buttons::{Action, Button},
                lcd::{
                    debug::LcdDebugState, main::LcdMainState, obd2_pids::LcdObd2Pids,
                    settings::LcdSettingsState,
                },
            },
            types::{Display1, Display2},
        };
        pub struct LcdMenuState {}
        #[automatically_derived]
        impl ::core::default::Default for LcdMenuState {
            #[inline]
            fn default() -> LcdMenuState {
                LcdMenuState {}
            }
        }
        impl LcdMenuState {
            pub fn new() -> Self {
                Self {}
            }
            pub fn handle_button(&mut self, button: &Action) -> Option<Response<State>> {
                match (&(button)) {
                    (arg0) => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd::menu".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"menu button: {:?}\",\"disambiguator\":\"11697105165103193822\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"menu button: {:?}\",\"disambiguator\":\"11697105165103193822\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            defmt::export::fmt(arg0);
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                match button {
                    Action::Pressed(Button::B4) => {
                        Some(Transition(State::main(LcdMainState::new())))
                    }
                    Action::Pressed(Button::B2) => {
                        Some(Transition(State::debug(LcdDebugState::new())))
                    }
                    Action::Pressed(Button::B1) => {
                        Some(Transition(State::obd2_pids(LcdObd2Pids::new())))
                    }
                    Action::Pressed(Button::B0) => {
                        Some(Transition(State::settings(LcdSettingsState::new())))
                    }
                    Action::Pressed(Button::B3) => {
                        crate::hal::reset();
                        None
                    }
                    _ => None,
                }
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                let icon =
                    embedded_iconoir::icons::size48px::devices::Computer::new(GrayColor::WHITE);
                let image = Image::new(&icon, Point::zero());
                match defmt::export::into_result(image.draw(display1)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display1)'\\nerror: `{:?}`\",\"disambiguator\":\"13129690753094884728\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display1)'\\nerror: `{:?}`\",\"disambiguator\":\"13129690753094884728\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                let icon =
                    embedded_iconoir::icons::size48px::weather::SnowFlake::new(GrayColor::WHITE);
                let image = Image::new(&icon, Point { x: 52, y: 0 });
                match defmt::export::into_result(image.draw(display1)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display1)'\\nerror: `{:?}`\",\"disambiguator\":\"8593521478869420196\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display1)'\\nerror: `{:?}`\",\"disambiguator\":\"8593521478869420196\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                let icon =
                    embedded_iconoir::icons::size48px::system::Settings::new(GrayColor::WHITE);
                let image = Image::new(&icon, Point { x: 52 * 1, y: 0 });
                match defmt::export::into_result(image.draw(display2)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"4517444473733141882\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"4517444473733141882\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                let icon = embedded_iconoir::icons::size48px::editor::List::new(GrayColor::WHITE);
                let image = Image::new(&icon, Point { x: 52 * 2, y: 0 });
                match defmt::export::into_result(image.draw(display2)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"13629979858741484347\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"13629979858741484347\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                let icon = embedded_iconoir::icons::size48px::development::CodeBrackets::new(
                    GrayColor::WHITE,
                );
                let image = Image::new(&icon, Point { x: 52 * 3, y: 0 });
                match defmt::export::into_result(image.draw(display2)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"1982916758908889896\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"1982916758908889896\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                let icon =
                    embedded_iconoir::icons::size48px::actions::Restart::new(GrayColor::WHITE);
                let image = Image::new(&icon, Point { x: 52 * 4, y: 0 });
                match defmt::export::into_result(image.draw(display2)) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"15172188447931358498\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: image.draw(display2)'\\nerror: `{:?}`\",\"disambiguator\":\"15172188447931358498\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"13718195074905577657\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"13718195074905577657\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::menu".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"5860298215527943750\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"5860298215527943750\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
    }
    mod obd2_pids {
        use core::sync::atomic::AtomicBool;
        use defmt::*;
        use crate::{
            display::widgets::{DebugScroll, Obd2DebugSelector},
            tasks::obd2::Obd2Debug,
            types::{Display1, Display2},
        };
        static OBD2_DEBUG_PIDS_ENABLED: AtomicBool = AtomicBool::new(false);
        pub struct LcdObd2Pids {
            debug: Obd2DebugSelector,
        }
        #[automatically_derived]
        impl ::core::default::Default for LcdObd2Pids {
            #[inline]
            fn default() -> LcdObd2Pids {
                LcdObd2Pids {
                    debug: ::core::default::Default::default(),
                }
            }
        }
        impl LcdObd2Pids {
            pub fn new() -> Self {
                OBD2_DEBUG_PIDS_ENABLED.store(true, core::sync::atomic::Ordering::Relaxed);
                Self {
                    debug: Obd2DebugSelector::new(),
                }
            }
            pub fn handle_obd2_debug(&mut self, event: &Obd2Debug) {
                self.debug.handle_obd2_debug(event);
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                self.debug.draw(display1, display2);
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::obd2_pids".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"921782619882349156\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"921782619882349156\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::obd2_pids".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"11624918828858082774\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"11624918828858082774\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
        impl Drop for LcdObd2Pids {
            fn drop(&mut self) {
                OBD2_DEBUG_PIDS_ENABLED.store(false, core::sync::atomic::Ordering::Relaxed);
            }
        }
        pub fn obd2_debug_pids_enabled() -> bool {
            OBD2_DEBUG_PIDS_ENABLED.load(core::sync::atomic::Ordering::Relaxed)
        }
    }
    mod settings {
        use core::sync::atomic::AtomicBool;
        use defmt::*;
        use embedded_graphics::prelude::*;
        use statig::Response::{self, Handled, Transition};
        use super::State;
        use crate::{
            display::widgets::{DebugScroll, Obd2DebugSelector, Slider, Text},
            tasks::{buttons::Action, lcd::menu::LcdMenuState, obd2::Obd2Debug},
            types::{Display1, Display2},
        };
        pub enum LcdSettingsEdit {
            #[default]
            Contrast,
        }
        impl defmt::Format for LcdSettingsEdit {
            fn format(&self, f: defmt::Formatter) {
                {
                    match () {
                        () => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd::settings".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"3589510316011211090\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"3589510316011211090\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            }
            fn _format_tag() -> defmt::Str {
                {
                    defmt::export::make_istr({
                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"Contrast\",\"disambiguator\":\"7908174106885248990\",\"crate_name\":\"display\"}"]
                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"Contrast\",\"disambiguator\":\"7908174106885248990\",\"crate_name\":\"display\"}"]
                        static S: u8 = 0;
                        &S as *const u8 as u16
                    })
                }
            }
            fn _format_data(&self) {
                match self {
                    LcdSettingsEdit::Contrast {} => {}
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for LcdSettingsEdit {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for LcdSettingsEdit {
            #[inline]
            fn eq(&self, other: &LcdSettingsEdit) -> bool {
                true
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for LcdSettingsEdit {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::clone::Clone for LcdSettingsEdit {
            #[inline]
            fn clone(&self) -> LcdSettingsEdit {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for LcdSettingsEdit {}
        #[automatically_derived]
        impl ::core::default::Default for LcdSettingsEdit {
            #[inline]
            fn default() -> LcdSettingsEdit {
                Self::Contrast
            }
        }
        pub struct LcdSettingsState {
            contrast: u8,
            contrast_text: Text,
            contrast_slider: Slider,
            current_contrast: u8,
            init: bool,
            edit: LcdSettingsEdit,
        }
        #[automatically_derived]
        impl ::core::default::Default for LcdSettingsState {
            #[inline]
            fn default() -> LcdSettingsState {
                LcdSettingsState {
                    contrast: ::core::default::Default::default(),
                    contrast_text: ::core::default::Default::default(),
                    contrast_slider: ::core::default::Default::default(),
                    current_contrast: ::core::default::Default::default(),
                    init: ::core::default::Default::default(),
                    edit: ::core::default::Default::default(),
                }
            }
        }
        impl LcdSettingsState {
            pub fn new() -> Self {
                match () {
                    () => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd::settings".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"LcdSettingsState::new()\",\"disambiguator\":\"5949588119352052354\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"LcdSettingsState::new()\",\"disambiguator\":\"5949588119352052354\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                let mut ret = Self {
                    contrast: 128,
                    current_contrast: 128,
                    contrast_slider: Slider::new(Point::new(128, 0), Size::new(128, 10)),
                    contrast_text: Text::new(
                        Point::new(30, 7),
                        &embedded_graphics::mono_font::ascii::FONT_6X10,
                        Some("Contrast: "),
                    ),
                    init: true,
                    edit: LcdSettingsEdit::Contrast,
                };
                ret.contrast_text.update_selected(true);
                ret
            }
            pub fn handle_button(&mut self, button: &Action) -> Option<Response<State>> {
                use crate::tasks::buttons::{Action::*, Button::*};
                match (&(button)) {
                    (arg0) => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd::settings".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"settings button: {:?}\",\"disambiguator\":\"4342158786706065345\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"settings button: {:?}\",\"disambiguator\":\"4342158786706065345\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            defmt::export::fmt(arg0);
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                match button {
                    Pressed(B0) => {
                        self.edit = LcdSettingsEdit::Contrast;
                        None
                    }
                    Pressed(B1) => {
                        self.edit = LcdSettingsEdit::Contrast;
                        None
                    }
                    Pressed(B4) => {
                        return Some(Transition(State::menu(LcdMenuState::new())));
                        None
                    }
                    Pressed(_) => {
                        self.handle_edit_contrast(button);
                        None
                    }
                    _ => Some(0),
                };
                None
            }
            fn handle_edit_contrast(&mut self, button: &Action) {
                use crate::tasks::buttons::{Action::*, Button::*};
                match button {
                    Pressed(B6) => {
                        self.contrast = self.contrast.saturating_sub(5);
                        self.contrast_slider
                            .update_percentage(self.contrast as f64 / 255.0 * 100.0);
                    }
                    Pressed(B7) => {
                        self.contrast = self.contrast.saturating_add(5);
                        self.contrast_slider
                            .update_percentage(self.contrast as f64 / 255.0 * 100.0);
                    }
                    _ => {}
                }
                match (&(self.contrast)) {
                    (arg0) => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd::settings".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"contrast: {}\",\"disambiguator\":\"10861960950488162461\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"contrast: {}\",\"disambiguator\":\"10861960950488162461\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            defmt::export::fmt(arg0);
                            unsafe { defmt::export::release() }
                        }
                    }
                };
            }
            pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
                if self.init {
                    self.contrast = display1.get_contrast();
                    self.contrast_slider
                        .update_percentage(self.contrast as f64 / 255.0 * 100.0);
                    self.init = false;
                }
                if self.current_contrast != self.contrast {
                    display1.set_contrast(self.contrast).await;
                    display2.set_contrast(self.contrast).await;
                    self.current_contrast = self.contrast;
                }
                self.contrast_text.draw(display1).ok();
                self.contrast_slider.draw(display1).ok();
                match defmt::export::into_result(display1.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::settings".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"1168144178811592011\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"1168144178811592011\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
                match defmt::export::into_result(display2.flush().await) {
                    ::core::result::Result::Ok(res) => res,
                    ::core::result::Result::Err(_unwrap_err) => {
                        match (&(_unwrap_err)) {
                            (arg0) => {
                                if {
                                    const CHECK: bool = {
                                        const fn check() -> bool {
                                            let module_path = "display::lcd::settings".as_bytes();
                                            if if 7usize > module_path.len() {
                                                false
                                            } else {
                                                module_path[0usize] == 100u8
                                                    && module_path[1usize] == 105u8
                                                    && module_path[2usize] == 115u8
                                                    && module_path[3usize] == 112u8
                                                    && module_path[4usize] == 108u8
                                                    && module_path[5usize] == 97u8
                                                    && module_path[6usize] == 121u8
                                                    && if 7usize == module_path.len() {
                                                        true
                                                    } else {
                                                        module_path[7usize] == b':'
                                                    }
                                            } {
                                                return true;
                                            }
                                            false
                                        }
                                        check()
                                    };
                                    CHECK
                                } {
                                    unsafe { defmt::export::acquire() };
                                    defmt::export::header(&{
                                        defmt::export::make_istr({
                                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"11770628413953855408\",\"crate_name\":\"display\"}"]
                                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.flush().await'\\nerror: `{:?}`\",\"disambiguator\":\"11770628413953855408\",\"crate_name\":\"display\"}"]
                                            static DEFMT_LOG_STATEMENT: u8 = 0;
                                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                                        })
                                    });
                                    defmt::export::fmt(arg0);
                                    unsafe { defmt::export::release() }
                                }
                            }
                        };
                        defmt::export::panic()
                    }
                };
            }
        }
    }
    use debug::LcdDebugState;
    use main::LcdMainState;
    use menu::LcdMenuState;
    use obd2_pids::LcdObd2Pids;
    use settings::LcdSettingsState;
    use crate::tasks::buttons::{Action, Button};
    pub static EVENTS: Channel<CriticalSectionRawMutex, LcdEvent, 128> = Channel::new();
    pub use obd2_pids::obd2_debug_pids_enabled;
    pub struct LcdContext {
        panic: Option<&'static str>,
    }
    pub enum LcdEvent {
        PowerOff,
        Main,
        Debug,
        Menu,
        Render,
        DebugLine(String<DEBUG_STRING_LEN>),
        Obd2Event(Obd2Event),
        Obd2Debug(Obd2Debug),
        Button(Action),
    }
    impl defmt::Format for LcdEvent
    where
        String<DEBUG_STRING_LEN>: defmt::Format,
        Obd2Event: defmt::Format,
        Obd2Debug: defmt::Format,
        Action: defmt::Format,
    {
        fn format(&self, f: defmt::Formatter) {
            {
                match () {
                    () => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"14190950444174031385\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"14190950444174031385\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                defmt::export::panic()
            }
        }
        fn _format_tag() -> defmt::Str {
            {
                defmt::export::make_istr({
                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"PowerOff|Main|Debug|Menu|Render|DebugLine({=?})|Obd2Event({=?})|Obd2Debug({=?})|Button({=?})\",\"disambiguator\":\"16330263780419894227\",\"crate_name\":\"display\"}"]
                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"PowerOff|Main|Debug|Menu|Render|DebugLine({=?})|Obd2Event({=?})|Obd2Debug({=?})|Button({=?})\",\"disambiguator\":\"16330263780419894227\",\"crate_name\":\"display\"}"]
                    static S: u8 = 0;
                    &S as *const u8 as u16
                })
            }
        }
        fn _format_data(&self) {
            match self {
                LcdEvent::PowerOff {} => {
                    defmt::export::u8(&0u8);
                }
                LcdEvent::Main {} => {
                    defmt::export::u8(&1u8);
                }
                LcdEvent::Debug {} => {
                    defmt::export::u8(&2u8);
                }
                LcdEvent::Menu {} => {
                    defmt::export::u8(&3u8);
                }
                LcdEvent::Render {} => {
                    defmt::export::u8(&4u8);
                }
                LcdEvent::DebugLine { 0: arg0 } => {
                    defmt::export::u8(&5u8);
                    defmt::export::fmt(arg0);
                }
                LcdEvent::Obd2Event { 0: arg0 } => {
                    defmt::export::u8(&6u8);
                    defmt::export::fmt(arg0);
                }
                LcdEvent::Obd2Debug { 0: arg0 } => {
                    defmt::export::u8(&7u8);
                    defmt::export::fmt(arg0);
                }
                LcdEvent::Button { 0: arg0 } => {
                    defmt::export::u8(&8u8);
                    defmt::export::fmt(arg0);
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for LcdEvent {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for LcdEvent {
        #[inline]
        fn eq(&self, other: &LcdEvent) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (LcdEvent::DebugLine(__self_0), LcdEvent::DebugLine(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (LcdEvent::Obd2Event(__self_0), LcdEvent::Obd2Event(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (LcdEvent::Obd2Debug(__self_0), LcdEvent::Obd2Debug(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (LcdEvent::Button(__self_0), LcdEvent::Button(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    _ => true,
                }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LcdEvent {
        #[inline]
        fn clone(&self) -> LcdEvent {
            match self {
                LcdEvent::PowerOff => LcdEvent::PowerOff,
                LcdEvent::Main => LcdEvent::Main,
                LcdEvent::Debug => LcdEvent::Debug,
                LcdEvent::Menu => LcdEvent::Menu,
                LcdEvent::Render => LcdEvent::Render,
                LcdEvent::DebugLine(__self_0) => {
                    LcdEvent::DebugLine(::core::clone::Clone::clone(__self_0))
                }
                LcdEvent::Obd2Event(__self_0) => {
                    LcdEvent::Obd2Event(::core::clone::Clone::clone(__self_0))
                }
                LcdEvent::Obd2Debug(__self_0) => {
                    LcdEvent::Obd2Debug(::core::clone::Clone::clone(__self_0))
                }
                LcdEvent::Button(__self_0) => {
                    LcdEvent::Button(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    pub struct LcdState {
        display1: Display1,
        display2: Display2,
        display_on: bool,
        is_debug: bool,
    }
    impl LcdState {
        pub fn new(display1: Display1, display2: Display2) -> Self {
            Self {
                display1,
                display2,
                display_on: false,
                is_debug: false,
            }
        }
        pub fn is_debug(&self) -> bool {
            self.is_debug
        }
    }
    use statig::{state, superstate, action};
    impl LcdState {
        async fn display_on(&mut self) {
            if self.display_on {
                match () {
                    () => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::lcd".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"display already on\",\"disambiguator\":\"12820456511266374955\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"display already on\",\"disambiguator\":\"12820456511266374955\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                return;
            }
            match () {
                _ => {}
            };
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display on got spi lock\",\"disambiguator\":\"10575143924977015487\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display on got spi lock\",\"disambiguator\":\"10575143924977015487\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display1.clear();
            self.display2.clear();
            self.display1.flush().await.ok();
            self.display2.flush().await.ok();
            match defmt::export::into_result(self.display1.sleep(false).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display1.sleep(false).await'\\nerror: `{:?}`\",\"disambiguator\":\"12774237946407484031\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display1.sleep(false).await'\\nerror: `{:?}`\",\"disambiguator\":\"12774237946407484031\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
            match defmt::export::into_result(self.display2.sleep(false).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display2.sleep(false).await'\\nerror: `{:?}`\",\"disambiguator\":\"14063787777851179239\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display2.sleep(false).await'\\nerror: `{:?}`\",\"disambiguator\":\"14063787777851179239\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
            crate::tasks::buttons::init();
            self.display_on = true;
            Timer::after(Duration::from_millis(100)).await;
        }
        async fn display_off(&mut self) {
            if !self.display_on {
                return;
            }
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display off\",\"disambiguator\":\"18303753972093519222\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display off\",\"disambiguator\":\"18303753972093519222\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display off got spi lock\",\"disambiguator\":\"10554418361551208975\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"display off got spi lock\",\"disambiguator\":\"10554418361551208975\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display1.clear();
            self.display2.clear();
            self.display1.flush().await.ok();
            self.display2.flush().await.ok();
            match defmt::export::into_result(self.display1.sleep(true).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display1.sleep(true).await'\\nerror: `{:?}`\",\"disambiguator\":\"11581709229629601300\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display1.sleep(true).await'\\nerror: `{:?}`\",\"disambiguator\":\"11581709229629601300\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
            match defmt::export::into_result(self.display2.sleep(true).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display2.sleep(true).await'\\nerror: `{:?}`\",\"disambiguator\":\"17336321589230936471\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: self.display2.sleep(true).await'\\nerror: `{:?}`\",\"disambiguator\":\"17336321589230936471\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
            self.display_on = false;
        }
        async fn state_dispatch(&mut self, event: &LcdEvent) -> Response<State> {
            match event {
                LcdEvent::Main => Transition(State::main(LcdMainState::new())),
                LcdEvent::Debug => Transition(State::debug(LcdDebugState::new())),
                LcdEvent::Button(Action::Pressed(pressed)) => {
                    if *pressed != Button::B3 {
                        Transition(State::menu(LcdMenuState::new()))
                    } else {
                        Handled
                    }
                }
                LcdEvent::Menu => Transition(State::menu(LcdMenuState::new())),
                LcdEvent::PowerOff => Transition(State::init()),
                _ => Handled,
            }
        }
        async fn enter_init(&mut self) {
            self.display_off().await;
        }
        async fn init(&mut self, event: &LcdEvent) -> Response<State> {
            match event {
                LcdEvent::Main => Transition(State::main(LcdMainState::new())),
                _ => Handled,
            }
        }
        async fn enter_main(&mut self, main: &mut LcdMainState) {
            self.display_on().await;
            let lock = crate::locks::SPI_BUS.lock().await;
            self.display1.clear();
            self.display2.clear();
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_main\",\"disambiguator\":\"14599589591247183675\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_main\",\"disambiguator\":\"14599589591247183675\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            main.draw(&mut self.display1, &mut self.display2).await;
        }
        async fn main(&mut self, main: &mut LcdMainState, event: &LcdEvent) -> Response<State> {
            let lock = crate::locks::SPI_BUS.lock().await;
            let ret = match event {
                LcdEvent::Obd2Event(obd2_event) => {
                    main.handle_obd2_event(obd2_event);
                    Handled
                }
                LcdEvent::Render => {
                    main.draw(&mut self.display1, &mut self.display2).await;
                    Handled
                }
                _ => Super,
            };
            ret
        }
        async fn enter_debug(&mut self, debug: &mut LcdDebugState) {
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"4496219794408097953\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"4496219794408097953\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display_on().await;
            self.display1.clear();
            self.display2.clear();
            debug.draw(&mut self.display1, &mut self.display2).await;
        }
        async fn debug(
            &mut self,
            context: &mut LcdContext,
            debug: &mut LcdDebugState,
            event: &LcdEvent,
        ) -> Response<State> {
            let lock = crate::locks::SPI_BUS.lock().await;
            match event {
                LcdEvent::DebugLine(line) => {
                    debug.add_line(line);
                    debug.draw(&mut self.display1, &mut self.display2).await;
                    Handled
                }
                _ => Super,
            }
        }
        async fn enter_menu(&mut self, menu: &mut LcdMenuState) {
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"6503881188828121615\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"6503881188828121615\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display_on().await;
            self.display1.clear();
            self.display2.clear();
            menu.draw(&mut self.display1, &mut self.display2).await;
        }
        async fn menu(
            &mut self,
            context: &mut LcdContext,
            menu: &mut LcdMenuState,
            event: &LcdEvent,
        ) -> Response<State> {
            let lock = crate::locks::SPI_BUS.lock().await;
            match event {
                LcdEvent::Button(action) => {
                    if let Some(transition) = menu.handle_button(action) {
                        return transition;
                    }
                    menu.draw(&mut self.display1, &mut self.display2).await;
                    Handled
                }
                _ => Super,
            }
        }
        async fn enter_obd2_pids(&mut self, obd2_pids: &mut LcdObd2Pids) {
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"13882256023322104540\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"13882256023322104540\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display_on().await;
            self.display1.clear();
            self.display2.clear();
            obd2_pids.draw(&mut self.display1, &mut self.display2).await;
        }
        async fn obd2_pids(
            &mut self,
            context: &mut LcdContext,
            obd2_pids: &mut LcdObd2Pids,
            event: &LcdEvent,
        ) -> Response<State> {
            let lock = crate::locks::SPI_BUS.lock().await;
            match event {
                LcdEvent::Obd2Debug(obd2_debug) => {
                    obd2_pids.handle_obd2_debug(obd2_debug);
                    obd2_pids.draw(&mut self.display1, &mut self.display2).await;
                    Handled
                }
                _ => Super,
            }
        }
        async fn enter_settings(&mut self, settings: &mut LcdSettingsState) {
            let lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"11476284367931974524\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_warn\",\"data\":\"enter_debug\",\"disambiguator\":\"11476284367931974524\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            self.display_on().await;
            self.display1.clear();
            self.display2.clear();
            settings.draw(&mut self.display1, &mut self.display2).await;
        }
        async fn settings(
            &mut self,
            context: &mut LcdContext,
            settings: &mut LcdSettingsState,
            event: &LcdEvent,
        ) -> Response<State> {
            let lock = crate::locks::SPI_BUS.lock().await;
            match event {
                LcdEvent::Button(action) => {
                    if let Some(transition) = settings.handle_button(action) {
                        return transition;
                    }
                    settings.draw(&mut self.display1, &mut self.display2).await;
                    Handled
                }
                _ => Super,
            }
        }
    }
    impl statig::awaitable::IntoStateMachine for LcdState {
        type Event<'event> = LcdEvent;
        type Context<'context> = LcdContext;
        type State = State;
        type Superstate<'sub> = Superstate;
        const INITIAL: State = State::init();
        const ON_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = Self::on_transition;
        const ON_DISPATCH: fn(&mut Self, StateOrSuperstate<'_, '_, Self>, &Self::Event<'_>) =
            Self::on_dispatch;
    }
    pub enum State {
        Init {},
        Debug { debug: LcdDebugState },
        Menu { menu: LcdMenuState },
        Main { main: LcdMainState },
        Settings { settings: LcdSettingsState },
        Obd2Pids { obd2_pids: LcdObd2Pids },
    }
    impl State {
        const fn init() -> Self {
            Self::Init {}
        }
        const fn debug(debug: LcdDebugState) -> Self {
            Self::Debug { debug }
        }
        const fn menu(menu: LcdMenuState) -> Self {
            Self::Menu { menu }
        }
        const fn main(main: LcdMainState) -> Self {
            Self::Main { main }
        }
        const fn settings(settings: LcdSettingsState) -> Self {
            Self::Settings { settings }
        }
        const fn obd2_pids(obd2_pids: LcdObd2Pids) -> Self {
            Self::Obd2Pids { obd2_pids }
        }
    }
    #[allow(unused)]
    impl statig::awaitable::State<LcdState> for State {
        fn call_handler<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            event: &'fut <LcdState as statig::IntoStateMachine>::Event<'_>,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<
            statig::Box<dyn core::future::Future<Output = statig::Response<Self>> + 'fut>,
        > {
            statig::Box::pin(async move {
                match self {
                    State::Init {} => LcdState::init(shared_storage, event).await,
                    State::Debug { debug } => {
                        LcdState::debug(shared_storage, context, debug, event).await
                    }
                    State::Menu { menu } => {
                        LcdState::menu(shared_storage, context, menu, event).await
                    }
                    State::Main { main } => LcdState::main(shared_storage, main, event).await,
                    State::Settings { settings } => {
                        LcdState::settings(shared_storage, context, settings, event).await
                    }
                    State::Obd2Pids { obd2_pids } => {
                        LcdState::obd2_pids(shared_storage, context, obd2_pids, event).await
                    }
                    _ => statig::Response::Super,
                }
            })
        }
        fn call_entry_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<statig::Box<dyn core::future::Future<Output = ()> + 'fut>> {
            statig::Box::pin(async move {
                match self {
                    State::Init {} => LcdState::enter_init(shared_storage).await,
                    State::Debug { debug } => LcdState::enter_debug(shared_storage, debug).await,
                    State::Menu { menu } => LcdState::enter_menu(shared_storage, menu).await,
                    State::Main { main } => LcdState::enter_main(shared_storage, main).await,
                    State::Settings { settings } => {
                        LcdState::enter_settings(shared_storage, settings).await
                    }
                    State::Obd2Pids { obd2_pids } => {
                        LcdState::enter_obd2_pids(shared_storage, obd2_pids).await
                    }
                    _ => {}
                }
            })
        }
        fn call_exit_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<statig::Box<dyn core::future::Future<Output = ()> + 'fut>> {
            statig::Box::pin(async move {
                match self {
                    State::Init {} => {}
                    State::Debug { debug } => {}
                    State::Menu { menu } => {}
                    State::Main { main } => {}
                    State::Settings { settings } => {}
                    State::Obd2Pids { obd2_pids } => {}
                    _ => {}
                }
            })
        }
        fn superstate(&mut self) -> Option<<LcdState as statig::IntoStateMachine>::Superstate<'_>> {
            match self {
                State::Init {} => None,
                State::Debug { debug } => Some(Superstate::StateDispatch {}),
                State::Menu { menu } => Some(Superstate::StateDispatch {}),
                State::Main { main } => Some(Superstate::StateDispatch {}),
                State::Settings { settings } => Some(Superstate::StateDispatch {}),
                State::Obd2Pids { obd2_pids } => Some(Superstate::StateDispatch {}),
                _ => None,
            }
        }
    }
    pub enum Superstate {
        StateDispatch {},
    }
    #[allow(unused)]
    impl<'sub> statig::awaitable::Superstate<LcdState> for Superstate
    where
        Self: 'sub,
    {
        fn call_handler<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            event: &'fut <LcdState as statig::IntoStateMachine>::Event<'_>,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<
            statig::Box<
                dyn core::future::Future<
                        Output = statig::Response<<LcdState as statig::IntoStateMachine>::State>,
                    > + 'fut,
            >,
        > {
            statig::Box::pin(async move {
                match self {
                    Superstate::StateDispatch {} => {
                        LcdState::state_dispatch(shared_storage, event).await
                    }
                    _ => statig::Response::Super,
                }
            })
        }
        fn call_entry_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<statig::Box<dyn core::future::Future<Output = ()> + 'fut>> {
            statig::Box::pin(async move {
                match self {
                    Superstate::StateDispatch {} => {}
                    _ => {}
                }
            })
        }
        fn call_exit_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut LcdState,
            context: &'fut mut <LcdState as statig::IntoStateMachine>::Context<'_>,
        ) -> core::pin::Pin<statig::Box<dyn core::future::Future<Output = ()> + 'fut>> {
            statig::Box::pin(async move {
                match self {
                    Superstate::StateDispatch {} => {}
                    _ => {}
                }
            })
        }
        fn superstate(&mut self) -> Option<<LcdState as statig::IntoStateMachine>::Superstate<'_>> {
            match self {
                Superstate::StateDispatch {} => None,
                _ => None,
            }
        }
    }
    impl LcdState {
        fn on_transition(&mut self, source: &State, target: &State) {
            self.is_debug = false;
            match target {
                State::Debug { debug: _ } => self.is_debug = true,
                _ => {}
            }
        }
        fn on_dispatch(&mut self, state: StateOrSuperstate<Self>, event: &LcdEvent) {}
    }
    #[doc(hidden)]
    async fn __run_task(
        mut display1: Display1,
        mut display2: Display2,
        panic: Option<&'static str>,
    ) {
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "display::lcd".as_bytes();
                            if if 7usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 100u8
                                    && module_path[1usize] == 105u8
                                    && module_path[2usize] == 115u8
                                    && module_path[3usize] == 112u8
                                    && module_path[4usize] == 108u8
                                    && module_path[5usize] == 97u8
                                    && module_path[6usize] == 121u8
                                    && if 7usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[7usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(&{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd init start\",\"disambiguator\":\"8171048178580756530\",\"crate_name\":\"display\"}"]
                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd init start\",\"disambiguator\":\"8171048178580756530\",\"crate_name\":\"display\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    });
                    unsafe { defmt::export::release() }
                }
            }
        };
        with_timeout(Duration::from_secs(5 * 60), obd2_init_wait())
            .await
            .ok();
        {
            let _lock = crate::locks::SPI_BUS.lock().await;
            match () {
                () => {
                    if {
                        const CHECK: bool = {
                            const fn check() -> bool {
                                let module_path = "display::lcd".as_bytes();
                                if if 7usize > module_path.len() {
                                    false
                                } else {
                                    module_path[0usize] == 100u8
                                        && module_path[1usize] == 105u8
                                        && module_path[2usize] == 115u8
                                        && module_path[3usize] == 112u8
                                        && module_path[4usize] == 108u8
                                        && module_path[5usize] == 97u8
                                        && module_path[6usize] == 121u8
                                        && if 7usize == module_path.len() {
                                            true
                                        } else {
                                            module_path[7usize] == b':'
                                        }
                                } {
                                    return true;
                                }
                                false
                            }
                            check()
                        };
                        CHECK
                    } {
                        unsafe { defmt::export::acquire() };
                        defmt::export::header(&{
                            defmt::export::make_istr({
                                #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"display init\",\"disambiguator\":\"10884967483059196705\",\"crate_name\":\"display\"}"]
                                #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"display init\",\"disambiguator\":\"10884967483059196705\",\"crate_name\":\"display\"}"]
                                static DEFMT_LOG_STATEMENT: u8 = 0;
                                &DEFMT_LOG_STATEMENT as *const u8 as u16
                            })
                        });
                        unsafe { defmt::export::release() }
                    }
                }
            };
            match defmt::export::into_result(display1.init(None).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.init(None).await'\\nerror: `{:?}`\",\"disambiguator\":\"15025305341182446243\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display1.init(None).await'\\nerror: `{:?}`\",\"disambiguator\":\"15025305341182446243\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
            match defmt::export::into_result(display2.init(None).await) {
                ::core::result::Result::Ok(res) => res,
                ::core::result::Result::Err(_unwrap_err) => {
                    match (&(_unwrap_err)) {
                        (arg0) => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::lcd".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.init(None).await'\\nerror: `{:?}`\",\"disambiguator\":\"6571679752895382647\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: display2.init(None).await'\\nerror: `{:?}`\",\"disambiguator\":\"6571679752895382647\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                defmt::export::fmt(arg0);
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            };
        }
        display1.set_contrast(50).await.ok();
        display2.set_contrast(50).await.ok();
        display1.clear();
        display2.clear();
        display1.flush().await.ok();
        display2.flush().await.ok();
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "display::lcd".as_bytes();
                            if if 7usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 100u8
                                    && module_path[1usize] == 105u8
                                    && module_path[2usize] == 115u8
                                    && module_path[3usize] == 112u8
                                    && module_path[4usize] == 108u8
                                    && module_path[5usize] == 97u8
                                    && module_path[6usize] == 121u8
                                    && if 7usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[7usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(&{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd init end\",\"disambiguator\":\"14681620971276020849\",\"crate_name\":\"display\"}"]
                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd init end\",\"disambiguator\":\"14681620971276020849\",\"crate_name\":\"display\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    });
                    unsafe { defmt::export::release() }
                }
            }
        };
        let mut context = LcdContext { panic };
        let mut state = LcdState::new(display1, display2)
            .uninitialized_state_machine()
            .init_with_context(&mut context)
            .await;
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "display::lcd".as_bytes();
                            if if 7usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 100u8
                                    && module_path[1usize] == 105u8
                                    && module_path[2usize] == 115u8
                                    && module_path[3usize] == 112u8
                                    && module_path[4usize] == 108u8
                                    && module_path[5usize] == 97u8
                                    && module_path[6usize] == 121u8
                                    && if 7usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[7usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(&{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd state machine initialized\",\"disambiguator\":\"18309588253354233080\",\"crate_name\":\"display\"}"]
                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"lcd state machine initialized\",\"disambiguator\":\"18309588253354233080\",\"crate_name\":\"display\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    });
                    unsafe { defmt::export::release() }
                }
            }
        };
        let mut render_ticker = embassy_time::Ticker::every(Duration::from_millis(1000 / 12));
        loop {
            match state.state() {
                State::Debug { debug: _ } => {
                    match select(EVENTS.receive(), crate::debug::receive()).await {
                        First(event) => state.handle_with_context(&event, &mut context).await,
                        Second(line) => {
                            state
                                .handle_with_context(&LcdEvent::DebugLine(line), &mut context)
                                .await
                        }
                    }
                }
                _ => {
                    let event = match select(EVENTS.receive(), render_ticker.next()).await {
                        First(event) => event,
                        Second(_) => LcdEvent::Render,
                    };
                    state.handle_with_context(&event, &mut context).await;
                }
            }
        }
    }
    pub fn run(
        display1: Display1,
        display2: Display2,
        panic: Option<&'static str>,
    ) -> ::embassy_executor::SpawnToken<impl Sized> {
        trait _EmbassyInternalTaskTrait {
            type Fut: ::core::future::Future + 'static;
            fn construct(
                display1: Display1,
                display2: Display2,
                panic: Option<&'static str>,
            ) -> Self::Fut;
        }
        impl _EmbassyInternalTaskTrait for () {
            type Fut = impl core::future::Future + 'static;
            fn construct(
                display1: Display1,
                display2: Display2,
                panic: Option<&'static str>,
            ) -> Self::Fut {
                __run_task(display1, display2, panic)
            }
        }
        const POOL_SIZE: usize = 1;
        static POOL: ::embassy_executor::raw::TaskPool<
            <() as _EmbassyInternalTaskTrait>::Fut,
            POOL_SIZE,
        > = ::embassy_executor::raw::TaskPool::new();
        unsafe {
            POOL._spawn_async_fn(move || {
                <() as _EmbassyInternalTaskTrait>::construct(display1, display2, panic)
            })
        }
    }
}
mod display {
    use embedded_graphics::{prelude::*, primitives::Rectangle};
    pub mod widgets {
        mod arrow {
            use core::fmt::Write;
            use defmt::info;
            use display_interface::DisplayError;
            use embedded_graphics::{
                draw_target::Clipped,
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            pub enum ArrowDirection {
                #[default]
                Forward,
                Reverse,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for ArrowDirection {
                #[inline]
                fn clone(&self) -> ArrowDirection {
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for ArrowDirection {}
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for ArrowDirection {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for ArrowDirection {
                #[inline]
                fn eq(&self, other: &ArrowDirection) -> bool {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    __self_discr == __arg1_discr
                }
            }
            #[automatically_derived]
            impl ::core::cmp::Eq for ArrowDirection {
                #[inline]
                #[doc(hidden)]
                #[coverage(off)]
                fn assert_receiver_is_total_eq(&self) -> () {}
            }
            #[automatically_derived]
            impl ::core::default::Default for ArrowDirection {
                #[inline]
                fn default() -> ArrowDirection {
                    Self::Forward
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for ArrowDirection {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(
                        f,
                        match self {
                            ArrowDirection::Forward => "Forward",
                            ArrowDirection::Reverse => "Reverse",
                        },
                    )
                }
            }
            pub struct Arrow {
                size: Size,
                position: Point,
                arrow_width: u32,
                offset: f64,
                old_offest: i32,
                force_update: bool,
                color: u8,
                speed: f64,
                direction: ArrowDirection,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Arrow {
                #[inline]
                fn clone(&self) -> Arrow {
                    let _: ::core::clone::AssertParamIsClone<Size>;
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    let _: ::core::clone::AssertParamIsClone<f64>;
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    let _: ::core::clone::AssertParamIsClone<u8>;
                    let _: ::core::clone::AssertParamIsClone<ArrowDirection>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Arrow {}
            #[automatically_derived]
            impl ::core::fmt::Debug for Arrow {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    let names: &'static _ = &[
                        "size",
                        "position",
                        "arrow_width",
                        "offset",
                        "old_offest",
                        "force_update",
                        "color",
                        "speed",
                        "direction",
                    ];
                    let values: &[&dyn ::core::fmt::Debug] = &[
                        &self.size,
                        &self.position,
                        &self.arrow_width,
                        &self.offset,
                        &self.old_offest,
                        &self.force_update,
                        &self.color,
                        &self.speed,
                        &&self.direction,
                    ];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f, "Arrow", names, values)
                }
            }
            #[automatically_derived]
            impl ::core::default::Default for Arrow {
                #[inline]
                fn default() -> Arrow {
                    Arrow {
                        size: ::core::default::Default::default(),
                        position: ::core::default::Default::default(),
                        arrow_width: ::core::default::Default::default(),
                        offset: ::core::default::Default::default(),
                        old_offest: ::core::default::Default::default(),
                        force_update: ::core::default::Default::default(),
                        color: ::core::default::Default::default(),
                        speed: ::core::default::Default::default(),
                        direction: ::core::default::Default::default(),
                    }
                }
            }
            impl Arrow {
                pub fn new(
                    position: Point,
                    size: Size,
                    arrow_width: u32,
                    direction: ArrowDirection,
                ) -> Self {
                    Self {
                        position,
                        size,
                        arrow_width,
                        old_offest: i32::MAX,
                        force_update: true,
                        color: 0,
                        offset: 0.0,
                        speed: 0.0,
                        direction,
                    }
                }
                pub fn update_direction(&mut self, direction: ArrowDirection) {
                    if self.direction != direction {
                        self.direction = direction;
                        self.force_update = true;
                    }
                }
                pub fn update_speed(&mut self, speed: f64) {
                    let old_speed = self.speed;
                    if speed > 0.0 {
                        self.speed = speed / 100.0 * 3.5 + 1.0;
                    } else {
                        self.speed = 0.0;
                    }
                    self.color = (speed / 100.0 * 16.0).round() as u8;
                    if speed != 0.0 && self.color == 0 {
                        self.color = 1;
                    }
                    if speed != old_speed {
                        self.force_update = true;
                    }
                    if self.color > 15 {
                        self.color = 15;
                    }
                    if self.speed > 4.5 {
                        self.speed = 4.5;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.offset >= self.arrow_width as f64 {
                        self.offset = self.speed;
                    }
                    let new_offest = self.offset.ceil() as i32;
                    if new_offest != self.old_offest || self.force_update {
                        let mut size = self.size;
                        size.height += 1;
                        let style_black = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::BLACK)
                            .fill_color(Gray4::BLACK)
                            .build();
                        let area = Rectangle::new(self.position, size);
                        area.draw_styled(&style_black, target)?;
                        let mut area = target.clipped(&area);
                        let style = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::new(self.color))
                            .fill_color(Gray4::new(self.color))
                            .build();
                        let triangle_offset = match self.direction {
                            ArrowDirection::Forward => -1,
                            ArrowDirection::Reverse => 1,
                        };
                        let triangle = Triangle::new(
                            Point::new(self.position.x, self.position.y),
                            Point::new(
                                self.position.x - (self.arrow_width as i32 - 6) * triangle_offset,
                                self.position.y + self.size.height as i32 / 2,
                            ),
                            Point::new(self.position.x, self.position.y + self.size.height as i32),
                        )
                        .translate(Point::new(-(triangle_offset * new_offest), 0));
                        if self.direction == ArrowDirection::Forward {
                            for a in (-1..(self.size.width / self.arrow_width) as i32 + 2).rev() {
                                self.draw_triangle(
                                    &mut area,
                                    &style,
                                    &style_black,
                                    triangle,
                                    triangle_offset,
                                    a,
                                )?;
                            }
                        } else {
                            for a in 0..(self.size.width / self.arrow_width) as i32 + 4 {
                                self.draw_triangle(
                                    &mut area,
                                    &style,
                                    &style_black,
                                    triangle,
                                    triangle_offset,
                                    a,
                                )?;
                            }
                        }
                        self.old_offest = new_offest;
                        self.force_update = false;
                    }
                    self.offset += self.speed;
                    Ok(())
                }
                fn draw_triangle<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    area: &mut Clipped<D>,
                    style: &PrimitiveStyle<Gray4>,
                    style_black: &PrimitiveStyle<Gray4>,
                    triangle: Triangle,
                    triangle_offset: i32,
                    a: i32,
                ) -> Result<(), D::Error> {
                    let triangle_a = triangle.translate(Point::new(
                        (self.arrow_width as f64 / 1.2).ceil() as i32 * a,
                        0,
                    ));
                    triangle_a.draw_styled(style, area)?;
                    triangle_a
                        .translate(Point::new(
                            triangle_offset * (self.arrow_width as i32 / 3),
                            0,
                        ))
                        .draw_styled(style_black, area)?;
                    Ok(())
                }
            }
        }
        mod battery {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            pub enum BatteryOrientation {
                #[default]
                VerticalTop,
                VerticalDown,
                HorizontalLeft,
                HorizontalRight,
            }
            #[automatically_derived]
            impl ::core::default::Default for BatteryOrientation {
                #[inline]
                fn default() -> BatteryOrientation {
                    Self::VerticalTop
                }
            }
            pub struct Battery {
                min_temp: f32,
                max_temp: f32,
                voltage: f32,
                cell_voltage_deviation: f32,
                cell_voltage: f32,
                percentage: f32,
                size: Size,
                position: Point,
                orientation: BatteryOrientation,
                cap: Option<Size>,
                bars: i32,
                inited: Option<(Point, Size)>,
                redraw: bool,
                text: bool,
            }
            #[automatically_derived]
            impl ::core::default::Default for Battery {
                #[inline]
                fn default() -> Battery {
                    Battery {
                        min_temp: ::core::default::Default::default(),
                        max_temp: ::core::default::Default::default(),
                        voltage: ::core::default::Default::default(),
                        cell_voltage_deviation: ::core::default::Default::default(),
                        cell_voltage: ::core::default::Default::default(),
                        percentage: ::core::default::Default::default(),
                        size: ::core::default::Default::default(),
                        position: ::core::default::Default::default(),
                        orientation: ::core::default::Default::default(),
                        cap: ::core::default::Default::default(),
                        bars: ::core::default::Default::default(),
                        inited: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                        text: ::core::default::Default::default(),
                    }
                }
            }
            impl Battery {
                pub fn new(
                    position: Point,
                    size: Size,
                    orientation: BatteryOrientation,
                    cap: Option<Size>,
                    bars: i32,
                    text: bool,
                ) -> Self {
                    Self {
                        position,
                        size,
                        orientation,
                        percentage: 0.0,
                        min_temp: 0.0,
                        max_temp: 0.0,
                        voltage: 0.0,
                        cell_voltage_deviation: 0.0,
                        cell_voltage: 0.0,
                        cap,
                        bars,
                        inited: None,
                        text,
                        redraw: true,
                    }
                }
                fn cap_draw<D: DrawTarget<Color = Gray4>>(
                    &self,
                    target: &mut D,
                ) -> Result<(Point, Size), D::Error> {
                    use BatteryOrientation::*;
                    Ok(if let Some(cap) = self.cap {
                        let style = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::WHITE)
                            .fill_color(Gray4::WHITE)
                            .build();
                        let mut size = self.size;
                        match self.orientation {
                            VerticalDown => {
                                size.height -= cap.height;
                                let mut position = self.position;
                                position.x += (self.size.width / 2) as i32 - (cap.width / 2) as i32;
                                position.y = self.size.height as i32 - cap.height as i32;
                                Rectangle::new(position, cap).draw_styled(&style, target)?;
                                (self.position, size)
                            }
                            VerticalTop => {
                                size.height -= cap.height + 2;
                                let mut position = self.position;
                                position.x += (self.size.width / 2) as i32 - (cap.width / 2) as i32;
                                let mut bar_position = self.position;
                                bar_position.y += cap.height as i32;
                                Rectangle::new(position, cap).draw_styled(&style, target)?;
                                (bar_position, size)
                            }
                            HorizontalRight => {
                                size.width -= cap.width;
                                let mut position = self.position;
                                position.x += self.size.width as i32 - cap.width as i32 - 6;
                                position.y +=
                                    (self.size.height / 2) as i32 - (cap.height / 2) as i32;
                                Rectangle::new(position, cap).draw_styled(&style, target)?;
                                let mut bar_position = self.position;
                                bar_position.x -= cap.width as i32;
                                (bar_position, size)
                            }
                            HorizontalLeft => {
                                size.width -= cap.width;
                                let mut position = self.position;
                                position.x = self.size.width as i32 - cap.width as i32;
                                position.y +=
                                    (self.size.height / 2) as i32 - (cap.height / 2) as i32;
                                Rectangle::new(position, cap).draw_styled(&style, target)?;
                                (self.position, size)
                            }
                        }
                    } else {
                        (self.position, self.size)
                    })
                }
                fn init_draw<D: DrawTarget<Color = Gray4>>(
                    &self,
                    target: &mut D,
                ) -> Result<(Point, Size), D::Error> {
                    let style = PrimitiveStyleBuilder::new()
                        .stroke_width(2)
                        .stroke_color(Gray4::WHITE)
                        .fill_color(Gray4::BLACK)
                        .build();
                    let (mut position, mut size) = self.cap_draw(target)?;
                    Rectangle::new(position, size).draw_styled(&style, target)?;
                    size.width -= 8;
                    size.height -= 8;
                    position.x += 4;
                    position.y += 4;
                    Ok((position, size))
                }
                pub fn update_percentage(&mut self, percentage: f32) {
                    if self.percentage != percentage {
                        self.percentage = percentage;
                        self.redraw = true;
                    }
                }
                pub fn update_voltage(&mut self, voltage: f32) {
                    if self.voltage != voltage {
                        self.voltage = voltage;
                        self.redraw = true;
                    }
                }
                pub fn update_cell_voltage_deviation(&mut self, cell_voltage_deviation: f32) {
                    if self.cell_voltage_deviation != cell_voltage_deviation {
                        self.cell_voltage_deviation = cell_voltage_deviation;
                        self.redraw = true;
                    }
                }
                pub fn update_cell_voltage(&mut self, cell_voltage: f32) {
                    if self.cell_voltage != cell_voltage {
                        self.cell_voltage = cell_voltage;
                        self.redraw = true;
                    }
                }
                pub fn update_min_temp(&mut self, min_temp: f32) {
                    if self.min_temp != min_temp {
                        self.min_temp = min_temp;
                        self.redraw = true;
                    }
                }
                pub fn update_max_temp(&mut self, max_temp: f32) {
                    if self.max_temp != max_temp {
                        self.max_temp = max_temp;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.inited.is_none() {
                        self.inited = Some(self.init_draw(target)?);
                    }
                    if let Some((mut position, mut size)) = self.inited
                        && self.redraw
                    {
                        use BatteryOrientation::*;
                        let style = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::WHITE)
                            .fill_color(Gray4::WHITE)
                            .build();
                        let mut style_black = style;
                        style_black.fill_color = Some(Gray4::BLACK);
                        style_black.stroke_color = Some(Gray4::BLACK);
                        Rectangle::new(position, size).draw_styled(&style_black, target)?;
                        let org_size = size;
                        let org_position = position;
                        let mut bar_style = style;
                        bar_style.stroke_color = Some(Gray4::new(0x02));
                        bar_style.fill_color = Some(Gray4::new(0x02));
                        match self.orientation {
                            VerticalDown => {
                                size.height =
                                    ((size.height as f32 * self.percentage) / 100.0).round() as u32;
                                Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                            }
                            VerticalTop => {
                                size.height =
                                    ((size.height as f32 * self.percentage) / 100.0).round() as u32;
                                position.y += org_size.height as i32 - size.height as i32;
                                Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                            }
                            HorizontalRight => {
                                size.width =
                                    ((size.width as f32 * self.percentage) / 100.0).round() as u32;
                                Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                            }
                            HorizontalLeft => {
                                size.width =
                                    ((size.width as f32 * self.percentage) / 100.0).round() as u32;
                                position.x += org_size.width as i32 - size.width as i32;
                                Rectangle::new(position, size).draw_styled(&bar_style, target)?;
                            }
                        }
                        if self.bars > 2 {
                            match self.orientation {
                                VerticalDown | VerticalTop => {
                                    let bar_size = org_size.height as i32 / self.bars;
                                    for bar in 0..self.bars {
                                        let mut bar_position = org_position;
                                        bar_position.y +=
                                            bar_size * (bar + 1) - (bar_size / 2 - 2) - (bar * 2);
                                        Rectangle::new(
                                            bar_position,
                                            Size {
                                                width: size.width,
                                                height: 1,
                                            },
                                        )
                                        .draw_styled(&style_black, target)?;
                                    }
                                }
                                HorizontalLeft | HorizontalRight => {
                                    let bar_size = ((org_size.width as i32 - self.bars * 2) as f32
                                        / (self.bars + 1) as f32)
                                        .floor()
                                        as i32
                                        + self.bars * 2;
                                    for bar in 0..(self.bars - 1) {
                                        let bar_translate = Point::new(
                                            (bar_size + 2) * bar - 2 + 1 + bar_size - 2,
                                            0,
                                        );
                                        Rectangle::new(
                                            org_position + bar_translate,
                                            Size {
                                                width: 1,
                                                height: size.height,
                                            },
                                        )
                                        .draw_styled(&style_black, target)?;
                                    }
                                }
                            }
                        }
                        if self.text {
                            let mut text: String<32> = String::new();
                            text.write_fmt(format_args!("{0:.1}%", self.percentage))
                                .ok();
                            let character_style =
                                MonoTextStyle::new(&PROFONT_14_POINT, Gray4::WHITE);
                            let mut text_style = TextStyleBuilder::new()
                                .alignment(Alignment::Center)
                                .line_height(LineHeight::Percent(100))
                                .build();
                            let mut text_position = org_position;
                            text_position.x +=
                                org_size.width as i32 / 2 / 2 + org_size.width as i32 / 2;
                            text_position.y += org_size.height as i32 / 2 + 5;
                            Text::with_text_style(
                                text.as_str(),
                                text_position,
                                character_style,
                                text_style,
                            )
                            .draw(target)?;
                            let character_style =
                                MonoTextStyle::new(&PROFONT_12_POINT, Gray4::WHITE);
                            text_style.alignment = Alignment::Left;
                            let mut text_position = org_position;
                            text_position.x += 2;
                            text_position.y +=
                                org_size.height as i32 / 2 / 2 + org_size.height as i32 / 2 + 6;
                            text.clear();
                            text.write_fmt(format_args!(
                                "{0:.1}V {1:.2}±{2:.0}",
                                self.voltage, self.cell_voltage, self.cell_voltage_deviation
                            ))
                            .ok();
                            Text::with_text_style(
                                text.as_str(),
                                text_position,
                                character_style,
                                text_style,
                            )
                            .draw(target)?;
                            let mut text_position = org_position;
                            text_position.x += 2;
                            text_position.y += org_size.height as i32 / 2 / 2 + 2;
                            text.clear();
                            text.write_fmt(format_args!(
                                "{0:.0}/{1:.0}°C",
                                self.min_temp, self.max_temp
                            ))
                            .ok();
                            Text::with_text_style(
                                text.as_str(),
                                text_position,
                                character_style,
                                text_style,
                            )
                            .draw(target)?;
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod battery_12v {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            pub struct Battery12V {
                voltage: f32,
                position: Point,
                redraw: bool,
                inited: bool,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Battery12V {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field4_finish(
                        f,
                        "Battery12V",
                        "voltage",
                        &self.voltage,
                        "position",
                        &self.position,
                        "redraw",
                        &self.redraw,
                        "inited",
                        &&self.inited,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Battery12V {
                #[inline]
                fn clone(&self) -> Battery12V {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Battery12V {}
            #[automatically_derived]
            impl ::core::default::Default for Battery12V {
                #[inline]
                fn default() -> Battery12V {
                    Battery12V {
                        voltage: ::core::default::Default::default(),
                        position: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                        inited: ::core::default::Default::default(),
                    }
                }
            }
            impl Battery12V {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        voltage: 0.0,
                        redraw: true,
                        inited: false,
                    }
                }
                pub fn update_voltage(&mut self, voltage: f32) {
                    if self.voltage != voltage {
                        self.voltage = voltage;
                        self.redraw = true;
                    }
                }
                pub fn init<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    let cap_height = 7;
                    let cap_width = 10;
                    let main_width = 32;
                    let main_height = 32;
                    let mut style = PrimitiveStyleBuilder::new()
                        .stroke_width(2)
                        .stroke_color(Gray4::WHITE)
                        .fill_color(Gray4::BLACK)
                        .build();
                    let main_rectangle = Rectangle::new(
                        self.position + Point::new(0, cap_height),
                        Size::new(main_width, main_height - cap_height as u32),
                    );
                    main_rectangle.draw_styled(&style, target)?;
                    let cap1_rectangle = Rectangle::with_center(
                        self.position + Point::new(main_width as i32 / 2 / 2, cap_height / 2),
                        Size::new(cap_width, cap_height as u32),
                    );
                    style.stroke_color = None;
                    style.fill_color = Some(Gray4::WHITE);
                    cap1_rectangle.draw_styled(&style, target)?;
                    let cap2_rectangle = cap1_rectangle.translate(Point::new(
                        main_width as i32 / 2 / 2 + cap_width as i32 / 2 + 1,
                        0,
                    ));
                    cap2_rectangle.draw_styled(&style, target)?;
                    let minus_rectangle = Rectangle::with_center(
                        Point::new(
                            cap1_rectangle.center().x,
                            main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2,
                        ),
                        Size::new(cap_width - 2, 3),
                    );
                    minus_rectangle.draw_styled(&style, target)?;
                    let plus1_rectangle = Rectangle::with_center(
                        Point::new(
                            cap2_rectangle.center().x + 1,
                            main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2,
                        ),
                        Size::new(cap_width - 1, 3),
                    );
                    let plus2_rectangle = Rectangle::with_center(
                        Point::new(
                            cap2_rectangle.center().x + 1,
                            main_rectangle.top_left.y + main_width as i32 / 2 / 2 - 2,
                        ),
                        Size::new(3, cap_width - 1),
                    );
                    plus1_rectangle.draw_styled(&style, target)?;
                    plus2_rectangle.draw_styled(&style, target)?;
                    self.inited = true;
                    Ok(())
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if !self.inited {
                        self.init(target)?;
                    }
                    if self.redraw {
                        let style = PrimitiveStyleBuilder::new()
                            .fill_color(Gray4::BLACK)
                            .build();
                        let main_width = 32;
                        let main_height = 30;
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0:.1}V", self.voltage)).ok();
                        let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Center)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let text = Text::with_text_style(
                            text.as_str(),
                            self.position + Point::new(main_width / 2, main_height - 4),
                            character_style,
                            text_style,
                        );
                        let mut rectangle = text.bounding_box();
                        rectangle.top_left.x = self.position.x + 1;
                        rectangle.size.width = main_width as u32 - 2;
                        rectangle.draw_styled(&style, target)?;
                        text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod connection {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                image::Image,
                pixelcolor::Gray4,
                prelude::*,
                primitives::{Rectangle, StyledDrawable as _},
            };
            use embedded_iconoir::prelude::IconoirNewIcon as _;
            pub struct Connection {
                position: Point,
                redraw: bool,
                last_send: bool,
                last_receive: bool,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Connection {
                #[inline]
                fn clone(&self) -> Connection {
                    Connection {
                        position: ::core::clone::Clone::clone(&self.position),
                        redraw: ::core::clone::Clone::clone(&self.redraw),
                        last_send: ::core::clone::Clone::clone(&self.last_send),
                        last_receive: ::core::clone::Clone::clone(&self.last_receive),
                    }
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Connection {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field4_finish(
                        f,
                        "Connection",
                        "position",
                        &self.position,
                        "redraw",
                        &self.redraw,
                        "last_send",
                        &self.last_send,
                        "last_receive",
                        &&self.last_receive,
                    )
                }
            }
            impl Default for Connection {
                fn default() -> Self {
                    Self {
                        position: Point::zero(),
                        redraw: true,
                        last_send: false,
                        last_receive: false,
                    }
                }
            }
            impl Connection {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        ..Default::default()
                    }
                }
                pub fn update_last_send(&mut self, last_send: bool) {
                    if self.last_send == last_send {
                        return;
                    }
                    self.last_send = last_send;
                    self.redraw = true;
                }
                pub fn update_last_receive(&mut self, last_receive: bool) {
                    if self.last_receive == last_receive {
                        return;
                    }
                    self.last_receive = last_receive;
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let icon =
                            embedded_iconoir::icons::size18px::connectivity::DataTransferBoth::new(
                                GrayColor::WHITE,
                            );
                        let image = Image::new(&icon, self.position);
                        image.draw(target)?;
                        let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                            .stroke_width(0)
                            .stroke_color(Gray4::BLACK)
                            .fill_color(Gray4::BLACK)
                            .build();
                        if !self.last_receive {
                            let bounding_box = Rectangle::new(
                                self.position + Point::new(0, 0),
                                Size::new(18 / 2, 18),
                            );
                            bounding_box.draw_styled(&style, target)?;
                        }
                        if !self.last_send {
                            let bounding_box = Rectangle::new(
                                self.position + Point::new(18 / 2, 0),
                                Size::new(18 / 2, 18),
                            );
                            bounding_box.draw_styled(&style, target)?;
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod debug {
            use core::{fmt::Write, str::FromStr as _};
            use defmt::{info, unwrap};
            use display_interface::DisplayError;
            use embassy_time::Instant;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::debug::{DEBUG_CHANNEL_LEN, DEBUG_STRING_LEN};
            pub struct DebugScroll {
                text_buffer: [heapless::String<DEBUG_STRING_LEN>; DEBUG_CHANNEL_LEN],
                redraw: bool,
            }
            #[automatically_derived]
            impl ::core::default::Default for DebugScroll {
                #[inline]
                fn default() -> DebugScroll {
                    DebugScroll {
                        text_buffer: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                    }
                }
            }
            impl DebugScroll {
                pub fn new() -> Self {
                    Self {
                        text_buffer: Default::default(),
                        redraw: false,
                    }
                }
                pub fn add_line(&mut self, line: &str) {
                    for i in 0..self.text_buffer.len() - 1 {
                        self.text_buffer[i] = self.text_buffer[i + 1].clone();
                    }
                    self.text_buffer[self.text_buffer.len() - 1] =
                        String::from_str(line).unwrap_or_default();
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>, D2: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                    target2: &mut D2,
                ) -> Result<(), ()> {
                    if self.redraw {
                        target.clear(Gray4::BLACK).map_err(|_| ())?;
                        target2.clear(Gray4::BLACK).map_err(|_| ())?;
                        let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let mut position = Point::new(0, 6);
                        let mut text_buffer_chunks = self.text_buffer.chunks(DEBUG_CHANNEL_LEN / 2);
                        for text in match defmt::export::into_result(text_buffer_chunks.next()) {
                            ::core::result::Result::Ok(res) => res,
                            ::core::result::Result::Err(_unwrap_err) => {
                                match (&(_unwrap_err)) {
                                    (arg0) => {
                                        if {
                                            const CHECK: bool = {
                                                const fn check() -> bool {
                                                    let module_path =
                                                        "display::display::widgets::debug"
                                                            .as_bytes();
                                                    if if 7usize > module_path.len() {
                                                        false
                                                    } else {
                                                        module_path[0usize] == 100u8
                                                            && module_path[1usize] == 105u8
                                                            && module_path[2usize] == 115u8
                                                            && module_path[3usize] == 112u8
                                                            && module_path[4usize] == 108u8
                                                            && module_path[5usize] == 97u8
                                                            && module_path[6usize] == 121u8
                                                            && if 7usize == module_path.len() {
                                                                true
                                                            } else {
                                                                module_path[7usize] == b':'
                                                            }
                                                    } {
                                                        return true;
                                                    }
                                                    false
                                                }
                                                check()
                                            };
                                            CHECK
                                        } {
                                            unsafe { defmt::export::acquire() };
                                            defmt::export::header(&{
                                                defmt::export::make_istr({
                                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: text_buffer_chunks.next()'\\nerror: `{:?}`\",\"disambiguator\":\"1866887524491613964\",\"crate_name\":\"display\"}"]
                                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: text_buffer_chunks.next()'\\nerror: `{:?}`\",\"disambiguator\":\"1866887524491613964\",\"crate_name\":\"display\"}"]
                                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                                })
                                            });
                                            defmt::export::fmt(arg0);
                                            unsafe { defmt::export::release() }
                                        }
                                    }
                                };
                                defmt::export::panic()
                            }
                        } {
                            let text =
                                Text::with_text_style(text, position, character_style, text_style);
                            text.draw(target).map_err(|_| ())?;
                            position += Point::new(0, 8);
                        }
                        let mut position = Point::new(0, 6);
                        for text in match defmt::export::into_result(text_buffer_chunks.next()) {
                            ::core::result::Result::Ok(res) => res,
                            ::core::result::Result::Err(_unwrap_err) => {
                                match (&(_unwrap_err)) {
                                    (arg0) => {
                                        if {
                                            const CHECK: bool = {
                                                const fn check() -> bool {
                                                    let module_path =
                                                        "display::display::widgets::debug"
                                                            .as_bytes();
                                                    if if 7usize > module_path.len() {
                                                        false
                                                    } else {
                                                        module_path[0usize] == 100u8
                                                            && module_path[1usize] == 105u8
                                                            && module_path[2usize] == 115u8
                                                            && module_path[3usize] == 112u8
                                                            && module_path[4usize] == 108u8
                                                            && module_path[5usize] == 97u8
                                                            && module_path[6usize] == 121u8
                                                            && if 7usize == module_path.len() {
                                                                true
                                                            } else {
                                                                module_path[7usize] == b':'
                                                            }
                                                    } {
                                                        return true;
                                                    }
                                                    false
                                                }
                                                check()
                                            };
                                            CHECK
                                        } {
                                            unsafe { defmt::export::acquire() };
                                            defmt::export::header(&{
                                                defmt::export::make_istr({
                                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: text_buffer_chunks.next()'\\nerror: `{:?}`\",\"disambiguator\":\"5132553037322893469\",\"crate_name\":\"display\"}"]
                                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'unwrap failed: text_buffer_chunks.next()'\\nerror: `{:?}`\",\"disambiguator\":\"5132553037322893469\",\"crate_name\":\"display\"}"]
                                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                                })
                                            });
                                            defmt::export::fmt(arg0);
                                            unsafe { defmt::export::release() }
                                        }
                                    }
                                };
                                defmt::export::panic()
                            }
                        } {
                            let text =
                                Text::with_text_style(text, position, character_style, text_style);
                            text.draw(target2).map_err(|_| ())?;
                            position += Point::new(0, 8);
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod fuel {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct Fuel<D> {
                max_temp: f64,
                min_temp: f64,
                current_temp: f64,
                current_temp_percentage: f64,
                size: Size,
                position: Point,
                bars: i32,
                redraw: bool,
                _marker: core::marker::PhantomData<D>,
            }
            impl<D> Fuel<D>
            where
                D: DrawTarget<Color = Gray4>,
            {
                pub fn new(position: Point, size: Size, min: f64, max: f64, bars: i32) -> Self {
                    Self {
                        position,
                        size,
                        current_temp: 0.0,
                        current_temp_percentage: 0.0,
                        max_temp: max,
                        min_temp: min,
                        bars,
                        redraw: true,
                        _marker: core::marker::PhantomData::default(),
                    }
                }
                pub fn update_temp(&mut self, temp: f64) {
                    if self.current_temp != temp {
                        self.current_temp = temp;
                        self.current_temp_percentage =
                            (temp - self.min_temp) / (self.max_temp - self.min_temp);
                        if self.current_temp > self.max_temp {
                            self.current_temp_percentage = 1.0;
                        } else if self.current_temp < self.min_temp {
                            self.current_temp_percentage = 0.0;
                        }
                        self.redraw = true;
                    }
                }
                pub fn draw(&mut self, target: &mut D) -> Result<(), D::Error> {
                    if self.redraw {
                        let color = Gray4::new(4);
                        let mut style = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::WHITE)
                            .fill_color(Gray4::BLACK)
                            .build();
                        let mut size = self.size;
                        size.width /= 2;
                        size.height -= self.size.width / 2;
                        let mut area = Rectangle::new(
                            self.position + Point::new(self.size.width as i32 / 2, 0),
                            size,
                        );
                        area.draw_styled(&style, target)?;
                        let mut circle_bottom = Circle::with_center(
                            self.position
                                + Point::new(
                                    self.size.width as i32 / 4,
                                    self.size.height as i32 - (self.size.width as i32 / 2),
                                )
                                + Point::new(size.width as i32 - 1, -2),
                            self.size.width,
                        );
                        circle_bottom.draw_styled(&style, target)?;
                        let circle = Circle::with_center(
                            self.position + Point::new(self.size.width as i32 / 2 + 3, 4),
                            self.size.width / 2,
                        );
                        style.fill_color = Some(Gray4::BLACK);
                        let mut circle_box = circle.bounding_box();
                        circle_box.size.width += 2;
                        circle_box.size.height -= 1;
                        circle_box.top_left.x -= 1;
                        circle_box.top_left.y -= 1;
                        target.fill_solid(&circle_box, Gray4::BLACK)?;
                        circle.draw_styled(&style, target)?;
                        style.stroke_color = Some(color);
                        area.size.height -= 4;
                        area.size.width -= 2;
                        area.top_left.x += 1;
                        area.top_left.y += 3;
                        let mut area_clipped = target.clipped(&area.bounding_box());
                        area_clipped.fill_solid(&area, Gray4::BLACK)?;
                        let mut size = self.size;
                        size.width /= 2;
                        size.height = ((self.size.height as f64 - 8.0)
                            * self.current_temp_percentage)
                            .round() as u32;
                        size.width -= 6;
                        style.fill_color = Some(color);
                        let mut position =
                            self.position + Point::new(self.size.width as i32 / 2, 0);
                        position.x += 3;
                        position.y = self.position.y + (self.size.height - 6) as i32
                            - size.height as i32
                            + 2;
                        let mut area_filled = Rectangle::new(position, size);
                        area_filled.draw_styled(&style, target)?;
                        area_filled.size.width = self.size.width;
                        area_filled.top_left.x -= self.size.width as i32 / 2;
                        area_filled.top_left.y -= 1;
                        area_filled.size.height += 1;
                        let mut area_filled = target.clipped(&area_filled);
                        circle_bottom.diameter -= 6;
                        circle_bottom.top_left.y += 3;
                        circle_bottom.top_left.x += 3;
                        circle_bottom.draw_styled(&style, &mut area_filled)?;
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0:.1}°C", self.current_temp))
                            .ok();
                        let character_style = MonoTextStyle::new(&PROFONT_9_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Center)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let mut rotate_target = RotatedDrawTarget::new(target);
                        let text_position = Point::new(20, 26);
                        let text = Text::with_text_style(
                            text.as_str(),
                            text_position,
                            character_style,
                            text_style,
                        );
                        let text_box = Rectangle::with_center(
                            text_position - Point::new(1, 2),
                            Size::new(42, 12),
                        );
                        rotate_target.fill_solid(&text_box, Gray4::BLACK)?;
                        text.draw(&mut rotate_target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod gearbox_gear {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct GearboxGear {
                position: Point,
                gear: &'static str,
                redraw: bool,
                bounding_box: Option<Rectangle>,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for GearboxGear {
                #[inline]
                fn clone(&self) -> GearboxGear {
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<&'static str>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    let _: ::core::clone::AssertParamIsClone<Option<Rectangle>>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for GearboxGear {}
            #[automatically_derived]
            impl ::core::fmt::Debug for GearboxGear {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field4_finish(
                        f,
                        "GearboxGear",
                        "position",
                        &self.position,
                        "gear",
                        &self.gear,
                        "redraw",
                        &self.redraw,
                        "bounding_box",
                        &&self.bounding_box,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::default::Default for GearboxGear {
                #[inline]
                fn default() -> GearboxGear {
                    GearboxGear {
                        position: ::core::default::Default::default(),
                        gear: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                        bounding_box: ::core::default::Default::default(),
                    }
                }
            }
            impl GearboxGear {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        gear: "U",
                        redraw: true,
                        bounding_box: None,
                    }
                }
                pub fn update_gear(&mut self, gear: &'static str) {
                    if self.gear != gear {
                        self.gear = gear;
                        self.redraw = true;
                    }
                }
                pub fn force_redraw(&mut self) {
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0}", self.gear)).ok();
                        let character_style = MonoTextStyle::new(&PROFONT_18_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Center)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let text = Text::with_text_style(
                            text.as_str(),
                            self.position,
                            character_style,
                            text_style,
                        );
                        let new_bounding_box = text.bounding_box();
                        if new_bounding_box.size.width
                            > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0)
                        {
                            self.bounding_box = Some(new_bounding_box);
                        }
                        if let Some(bb) = self.bounding_box {
                            bb.draw_styled(
                                &PrimitiveStyleBuilder::new()
                                    .fill_color(Gray4::BLACK)
                                    .build(),
                                target,
                            )?;
                        }
                        text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod ice_fuel_rate {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct IceFuelRate {
                position: Point,
                ice_fuel_rate: f32,
                vehicle_speed: f32,
                redraw: bool,
                bounding_box: Option<Rectangle>,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for IceFuelRate {
                #[inline]
                fn clone(&self) -> IceFuelRate {
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    let _: ::core::clone::AssertParamIsClone<Option<Rectangle>>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for IceFuelRate {}
            #[automatically_derived]
            impl ::core::fmt::Debug for IceFuelRate {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field5_finish(
                        f,
                        "IceFuelRate",
                        "position",
                        &self.position,
                        "ice_fuel_rate",
                        &self.ice_fuel_rate,
                        "vehicle_speed",
                        &self.vehicle_speed,
                        "redraw",
                        &self.redraw,
                        "bounding_box",
                        &&self.bounding_box,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::default::Default for IceFuelRate {
                #[inline]
                fn default() -> IceFuelRate {
                    IceFuelRate {
                        position: ::core::default::Default::default(),
                        ice_fuel_rate: ::core::default::Default::default(),
                        vehicle_speed: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                        bounding_box: ::core::default::Default::default(),
                    }
                }
            }
            impl IceFuelRate {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        ice_fuel_rate: 0.0,
                        vehicle_speed: 0.0,
                        redraw: true,
                        bounding_box: None,
                    }
                }
                pub fn update_ice_fuel_rate(&mut self, ice_fuel_rate: f32) {
                    if self.ice_fuel_rate != ice_fuel_rate {
                        self.ice_fuel_rate = ice_fuel_rate;
                        self.redraw = true;
                    }
                }
                pub fn update_vehicle_speed(&mut self, vehicle_speed: f32) {
                    if self.vehicle_speed != vehicle_speed {
                        self.vehicle_speed = vehicle_speed;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let mut text: String<16> = String::new();
                        let mut fuel_per_100km = 0.0;
                        if self.vehicle_speed > 0.0 {
                            fuel_per_100km = self.ice_fuel_rate / self.vehicle_speed * 100.0;
                        }
                        text.write_fmt(format_args!("{0:.1}", fuel_per_100km)).ok();
                        let character_style = MonoTextStyle::new(&PROFONT_10_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let draw_text = Text::with_text_style(
                            text.as_str(),
                            self.position,
                            character_style,
                            text_style,
                        );
                        let new_bounding_box = draw_text.bounding_box();
                        if new_bounding_box.size.width
                            > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0)
                        {
                            self.bounding_box = Some(new_bounding_box);
                        }
                        if let Some(bb) = self.bounding_box {
                            bb.draw_styled(
                                &PrimitiveStyleBuilder::new()
                                    .fill_color(Gray4::BLACK)
                                    .build(),
                                target,
                            )?;
                        }
                        draw_text.draw(target)?;
                        text.clear();
                        text.write_fmt(format_args!("l/100")).ok();
                        let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let draw_text = Text::with_text_style(
                            text.as_str(),
                            new_bounding_box.top_left
                                + Point::new(new_bounding_box.size.width as i32 + 4, 8),
                            character_style,
                            text_style,
                        );
                        let new_bounding_box = draw_text.bounding_box();
                        new_bounding_box.draw_styled(
                            &PrimitiveStyleBuilder::new()
                                .fill_color(Gray4::BLACK)
                                .build(),
                            target,
                        )?;
                        draw_text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod icon {
            use alloc::{borrow::Cow, string::ToString as _};
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                image::Image,
                pixelcolor::Gray4,
                prelude::*,
                primitives::{Rectangle, StyledDrawable as _},
            };
            use embedded_iconoir::prelude::IconoirNewIcon as _;
            pub struct Icon<I> {
                position: Point,
                size: u32,
                redraw: bool,
                last_enabled: bool,
                _icon: core::marker::PhantomData<I>,
            }
            #[automatically_derived]
            impl<I: ::core::clone::Clone> ::core::clone::Clone for Icon<I> {
                #[inline]
                fn clone(&self) -> Icon<I> {
                    Icon {
                        position: ::core::clone::Clone::clone(&self.position),
                        size: ::core::clone::Clone::clone(&self.size),
                        redraw: ::core::clone::Clone::clone(&self.redraw),
                        last_enabled: ::core::clone::Clone::clone(&self.last_enabled),
                        _icon: ::core::clone::Clone::clone(&self._icon),
                    }
                }
            }
            #[automatically_derived]
            impl<I: ::core::fmt::Debug> ::core::fmt::Debug for Icon<I> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field5_finish(
                        f,
                        "Icon",
                        "position",
                        &self.position,
                        "size",
                        &self.size,
                        "redraw",
                        &self.redraw,
                        "last_enabled",
                        &self.last_enabled,
                        "_icon",
                        &&self._icon,
                    )
                }
            }
            impl<I> Default for Icon<I> {
                fn default() -> Self {
                    Self {
                        size: 0,
                        position: Point::zero(),
                        redraw: true,
                        last_enabled: true,
                        _icon: core::marker::PhantomData,
                    }
                }
            }
            impl<I: embedded_iconoir::prelude::IconoirIcon> Icon<I> {
                pub fn new(position: Point, enabled: bool) -> Self {
                    Self {
                        position,
                        last_enabled: enabled,
                        ..Default::default()
                    }
                }
                pub fn enabled(&mut self, enabled: bool) {
                    if self.last_enabled == enabled {
                        return;
                    }
                    self.last_enabled = enabled;
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        if self.last_enabled {
                            let icon = I::new(GrayColor::WHITE);
                            let image = Image::new(&icon, self.position);
                            image.draw(target)?;
                            self.size = image.bounding_box().size.width;
                        } else {
                            if self.size != 0 {
                                let style =
                                    embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                                        .stroke_width(0)
                                        .stroke_color(Gray4::BLACK)
                                        .fill_color(Gray4::BLACK)
                                        .build();
                                let bounding_box = Rectangle::new(self.position, Size::new(18, 18));
                                bounding_box.draw_styled(&style, target)?;
                            }
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod motor_electric {
            use core::cell::UnsafeCell;
            use display_interface::DisplayError;
            use embedded_graphics::{
                image::Image,
                pixelcolor::{Gray4, Rgb565},
                prelude::*,
                primitives::*,
            };
            use once_cell::sync::{Lazy, OnceCell};
            use static_cell::StaticCell;
            use tinybmp::Bmp;
            pub struct MotorElectric {
                motor_im: Image<'static, Bmp<'static, Rgb565>>,
                motor_on_im: Image<'static, Bmp<'static, Rgb565>>,
                motor_off_im: Image<'static, Bmp<'static, Rgb565>>,
                on: bool,
                needs_update: bool,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for MotorElectric {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field5_finish(
                        f,
                        "MotorElectric",
                        "motor_im",
                        &self.motor_im,
                        "motor_on_im",
                        &self.motor_on_im,
                        "motor_off_im",
                        &self.motor_off_im,
                        "on",
                        &self.on,
                        "needs_update",
                        &&self.needs_update,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::clone::Clone for MotorElectric {
                #[inline]
                fn clone(&self) -> MotorElectric {
                    MotorElectric {
                        motor_im: ::core::clone::Clone::clone(&self.motor_im),
                        motor_on_im: ::core::clone::Clone::clone(&self.motor_on_im),
                        motor_off_im: ::core::clone::Clone::clone(&self.motor_off_im),
                        on: ::core::clone::Clone::clone(&self.on),
                        needs_update: ::core::clone::Clone::clone(&self.needs_update),
                    }
                }
            }
            impl Default for MotorElectric {
                fn default() -> Self {
                    Self::new(Point::zero())
                }
            }
            impl MotorElectric {
                pub fn new(position: Point) -> Self {
                    static MOTOR_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_bmp = MOTOR_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM\x8a\x1e\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x00;\x00\x00\x00@\x00\x00\x00\x01\x00\x10\x00\x03\x00\x00\x00\x00\x1e\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xf8\x00\x00\xe0\x07\x00\x00\x1f\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00A\x08\xef{\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x96\xb5\x00\x00\xc3\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa2\x10\x14\xa5\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x96\xb5\x00\x00\xc3\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04!\x17\xbe\xff\xffQ\x8c\x82\x10!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08 \x00\x00\x00\xc3\x18\xff\xffq\x8c!\x08!\x08!\x08 \x00\x00\x00\x82\x10\xe3\x18\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x861<\xe7\xff\xffmk!\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00)J\xde\xf7\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\xe3\x18\x861e)e)e)e)e)e)e)e)e)$!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xebZ\xff\xff]\xef\x08B\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00$!y\xce\xde\xf7\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x10\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00A\x08\xcf{\xff\xff8\xc6e)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\xe3\x188\xc6\xff\xff\xb2\x94\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84q\x8c\xff\xff\x10\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00b\x10\x92\x94\xff\xff\x96\xb5\x04!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xdf\xff0\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xc3\x18\x17\xbe\xff\xff,c\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\x10\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18u\xad\xff\xff\xd3\x9c\x82\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 \x00 \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xb6\xb5\xff\xffMk \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\x10\x84\x00\x00\x00\x00\x00\x00\x00\x00E)\x18\xc6\xff\xff\xef{A\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08B\xf3\x9c\x00\x00\xff\xff\x99\xce \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\x96\xb5\xaaR\xaaR\xaaR\xebZ<\xe7\xff\xff,c \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x8aR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00A\x08\x861\x861\x861\x861\x861U\xad\xff\xffb\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 \x00\x00\x00\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffb\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00b\x10\xc3\x18\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xaes\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffb\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xe3\x18\xa61\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa3\x18E)\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\xb2\x94\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xffY\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xffy\xceu\xadu\xadu\xad}\xef\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\xff\xff\x9d\xef\xc79\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xe79\x10\x84\x00\x00\xe79\xff\xff\xbe\xf7\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08B\xff\xff\x9d\xefIJ\xa61\xa61\xa61\xa61\xa61\xa61\xa61\x0cc\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00iJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x8aR\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffb\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00A\x08b\x10\x00\x00\x00\x00\x00\x00\x00\x00IJMkMkMkMkMkMkMkMkMkMk\x861\x00\x00\x00\x00\x00\x00Q\x8c\xff\xffu\xadU\xadU\xadU\xadU\xadU\xadU\xadU\xadU\xadU\xadU\xad0\x84A\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00Q\x8c\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffq\x8cA\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xa3\x18\xc3\x18\xb2\x94\xff\xffq\x8ca\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00b\x10\xd3\x9c\xff\xffQ\x8cA\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10\xf3\x9c\xff\xffQ\x8cA\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10U\xad\xff\xff\x10\x84A\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x104\xa5\xff\xff\xef{!\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc79\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa2\x10U\xad\xff\xff\xcf{\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xc3\x18\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa2\x10u\xad\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa2\x10,c,c,c,c,c,c,c,c,c,c,c,c\x92\x94\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08B\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\xe79\x10\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08B\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00E)\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ0\x84\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff(B\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xef{\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\xe4 \x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xebZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    static MOTOR_ON_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_on_bmp = MOTOR_ON_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM\x02\x06\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x00\x12\x00\x00\x00\x19\x00\x00\x00\x01\x00\x18\x00\x00\x00\x00\x00x\x05\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x04\x08! !! !! !! !\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08\xce\xca\xce\xff\xff\xff\xff\xff\xff\xff\xff\xffkik\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00kik\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff)()\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00151\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xa5\xa2\xa5\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08\xde\xdb\xde\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffRQR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00{y{\xff\xff\xff\xff\xff\xff\xff\xff\xff\xef\xef\xef\xde\xdf\xde\x19\x1c\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00:9:\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x84\x86\x84\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x0c\x10\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff:=:\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x84\x82\x84\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xbd\xbe\xbd\x10\x0c\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00151151151151151151kmk\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffkik\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00{}{\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff)()\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00:9:\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xa5\xa6\xa5\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08\xce\xce\xce\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffRQR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00kik\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xde\xdb\xde\x19\x18\x19\x00\x00\x00\x00\x00\x00\x00\x00)()\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x84\x86\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xb5\xb6\xb5\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xef\xf3\xef\xbd\xc2\xbd\xbd\xc2\xbd\xbd\xc2\xbd\xbd\xc2\xbd\xbd\xc2\xbd\xbd\xc2\xbd151\x00\x00\x00\x00\x00\x00\x00\x00JMJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xd6\xd7\xd6\x08\x0c\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\x18\x19\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffBAB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00{}{\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x94\x96\x94\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00111\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff!\x1c!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08\xc5\xca\xc5\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffZ]Z\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00cec\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xf3\xf7\xc5\xc6\xc5\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00111\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xef\xef\xef\xff\xff\xff:5:\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\xc5\xc6\xc5\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x84\x86\x84\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x10\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    static MOTOR_OFF_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_off_bmp = MOTOR_OFF_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM\x9e\x05\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x00\x11\x00\x00\x00\x19\x00\x00\x00\x01\x00\x18\x00\x00\x00\x00\x00\x14\x05\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00! !1-11-11-1\x10\x0c\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x0c\x101-1!\x1c!! !1-1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x001-1\x10\x10\x10\x08\x04\x081-1\x19\x18\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00)$))()\x00\x00\x00\x10\x14\x101-1\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x101-1\x00\x04\x00\x00\x00\x00)())()\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x001-1\x10\x0c\x10\x00\x00\x00\x08\x08\x081-1\x10\x14\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x001-1! !\x00\x00\x00\x00\x00\x00\x19\x18\x191-1\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x101-1\x00\x00\x00\x00\x00\x00\x00\x00\x001-1! !\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x04\x00\x00\x04\x00\x00\x04\x00\x00\x04\x00\x00\x04\x00\x10\x0c\x101-1\x08\x0c\x08\x00\x00\x00\x00\x00\x00\x10\x0c\x101-1\x10\x0c\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x101-11-11-11-11-11-11-1!\x1c!\x00\x00\x00\x00\x00\x00\x00\x00\x00! !1-1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x001-1)()\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x0c\x10\x08\x0c\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x04\x081-1\x19\x18\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00!$!)()\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x14\x101-1\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x10\x0c\x101-1\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x001-1)$)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x001-1\x19\x14\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\x14\x191-11-11-11-11-11-11-1\x10\x14\x10\x00\x00\x00\x00\x00\x00\x00!\x1c!1-1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x04\x081-1)-)! !! !! !! !! !! !\x00\x00\x00\x00\x00\x00\x00\x08\x08\x081-1\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00)()!$!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x001-1!\x1c!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x0c\x101-1\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x101-1\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x001-1\x19\x14\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x001-1\x10\x10\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\x1c\x191-1\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00! !)-)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x081-1\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x0c\x081-1\x08\x04\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00)-)! !\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x001-1!\x1c!\x10\x14\x10\x10\x14\x10\x10\x14\x10\x10\x14\x10)()1-1\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00! !1-11-11-11-11-11-11-1\x10\x14\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    let motor_im: Image<Bmp<Rgb565>> = Image::new(motor_bmp, position);
                    let motor_on_im: Image<Bmp<Rgb565>> =
                        Image::new(motor_on_bmp, position).translate(Point::new(34, 24));
                    let motor_off_im: Image<Bmp<Rgb565>> =
                        Image::new(motor_off_bmp, position).translate(Point::new(34, 24));
                    Self {
                        motor_im,
                        motor_on_im,
                        motor_off_im,
                        on: false,
                        needs_update: true,
                    }
                }
                pub fn update_on(&mut self, on: bool) {
                    if self.on != on {
                        self.on = on;
                        self.needs_update = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.needs_update {
                        self.motor_im.draw(&mut target.color_converted())?;
                        if self.on {
                            self.motor_on_im.draw(&mut target.color_converted())?;
                        } else {
                            self.motor_off_im.draw(&mut target.color_converted())?;
                        }
                        self.needs_update = false;
                    }
                    Ok(())
                }
            }
        }
        mod motor_ice {
            use display_interface::DisplayError;
            use embedded_graphics::{
                image::Image,
                pixelcolor::{Gray4, Rgb565},
                prelude::*,
                primitives::*,
            };
            use once_cell::sync::OnceCell;
            use static_cell::StaticCell;
            use tinybmp::Bmp;
            pub struct MotorIce {
                motor_im: Image<'static, Bmp<'static, Rgb565>>,
                motor_on_im: Image<'static, Bmp<'static, Rgb565>>,
                motor_off_im: Image<'static, Bmp<'static, Rgb565>>,
                on: bool,
                needs_update: bool,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for MotorIce {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field5_finish(
                        f,
                        "MotorIce",
                        "motor_im",
                        &self.motor_im,
                        "motor_on_im",
                        &self.motor_on_im,
                        "motor_off_im",
                        &self.motor_off_im,
                        "on",
                        &self.on,
                        "needs_update",
                        &&self.needs_update,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::clone::Clone for MotorIce {
                #[inline]
                fn clone(&self) -> MotorIce {
                    MotorIce {
                        motor_im: ::core::clone::Clone::clone(&self.motor_im),
                        motor_on_im: ::core::clone::Clone::clone(&self.motor_on_im),
                        motor_off_im: ::core::clone::Clone::clone(&self.motor_off_im),
                        on: ::core::clone::Clone::clone(&self.on),
                        needs_update: ::core::clone::Clone::clone(&self.needs_update),
                    }
                }
            }
            impl Default for MotorIce {
                fn default() -> Self {
                    Self::new(Point::zero())
                }
            }
            impl MotorIce {
                pub fn new(position: Point) -> Self {
                    static MOTOR_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_bmp = MOTOR_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM\x8a\x1a\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x004\x00\x00\x00@\x00\x00\x00\x01\x00\x10\x00\x03\x00\x00\x00\x00\x1a\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xf8\x00\x00\xe0\x07\x00\x00\x1f\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00iJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x96\xb5\xc3\x18\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00iJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff8\xc6$!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xe3\x18a\x08\x00\x00\x00\x00!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08!\x08A\x08,c\xff\xffY\xceE)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 \x00\xcbZ\xff\xff\xfb\xde\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8aR\xff\xff\xdf\xff\xe8A\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08B\x9e\xf7\xff\xffIJ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa61]\xef\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00e)<\xe7\xff\xff,c \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00$!y\xce\xff\xff\x8es \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xf3\x9ce)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xe3\x18\xb6\xb5\xff\xff\xef{\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61E)\x00\x00\x00\x00\x00\x00\xe4 \xa61\xa61\xa61\xa61\xa61\xa61 \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18U\xad\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10\xaaR\xabZ\xabZ\xabZ\xabZ\xabZ\xabZ\xabZMk\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\xcf{\xaaR\xaaR\xff\xff\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xc3\x18A\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xb6\xb5\xcbZ\xcbZ\xcbZ\xb2\x94\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xaes$!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\x18\xc6\x08B\x08B\x08B\x92\x94\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xffa\x08\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xffu\xadb\x10b\x10b\x10\x8es\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xff\x0cc\xcbZ\xcbZ\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xffu\xad\x82\x10b\x10b\x10\x8es\xff\xff\x861\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xffu\xad\x82\x10\x82\x10b\x10\x8es\xff\xff\x861\x00\x00\x00\x00\x861\x861\x861<\xe7\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xba\xd6\xcf{\xcf{\xcf{\x96\xb5\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xa61\xa2\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\xf3\x9c\xa61\xa61\xa61\xcf{\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xffE)\x82\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\x10\x84E)\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xdb\xde\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa3\x18\x861\x861\x861\x861\x861\x861\x861\x861\x861(B\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\xaaRe)e)e)e)e)\x1c\xe7\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00A\x08U\xad\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff0\x84\x00\x00\x00\x00\x00\x00\xaaR\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffb\x10!\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xebZ\xff\xffU\xad\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\xebZ\x00\x00\x00\x00\x00\x00\x08B\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\x10\x84\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xff\xff\x9d\xef\xe4 \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00Q\x8c\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xb2\x94!\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\xba\xd6\x9d\xef\x04!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 \x00\xb2\x94\xff\xffiJ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00IJ\xff\xff\xf3\x9c \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xc3\x18\x9e\xf7}\xef\x04!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x82\x10A\x08\x00\x00\x00\x00\x00\x00\x00\x00\xa2\x10\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\xa61\x92\x94\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff4\xa5A\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xf3\x9c,c,c,c,c,c,c,c,c,c,c,c,c\x04!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x84E)\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xcbZ\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xb2\x94\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\xcbZ\x861\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\xa61\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\xa3\x18\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x08B\x0cc\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x861\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xa61\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x861\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x861\x82\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x861\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00iJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00iJ\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xaaR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    static MOTOR_ON_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_on_bmp = MOTOR_ON_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM.\x07\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x00\x16\x00\x00\x00\x19\x00\x00\x00\x01\x00\x18\x00\x00\x00\x00\x00\xa4\x06\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08! !! !! !! !! !! !! !! !! !! !! !! !! !\x19\x18\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00!\x1c!\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x84\x82\x84\x00\x00\x00\x00\x00\x00! !111)()\x08\x04\x08\x00\x00\x00\x00\x00\x19\x14\x19\xa5\xa2\xa5\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffZ]Z\x00\x00\x00RQR\xff\xff\xff\xff\xff\xff\xff\xff\xff\x84\x82\x84\x00\x00\x00\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xc5\xc6\xc5\xff\xff\xffsus\xc5\xc2\xc5\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xf7\xfb\xf7\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x19\x14\x19\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xff\xff\x84\x86\x84\x84\x82\x84\xff\xff\xff\xf7\xf7\xf7\x10\x10\x10\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffcec\x00\x00\x00\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xff\xff! !\x19\x1c\x19\x19\x18\x19\x00\x00\x00\x00\x00\x00\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00BAB\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xf7\xfb\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x8c\x8a\x8c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x08\x08\x08\x08\x08\x00\x00\x00\x00\x00\x00\x00\x00B=B\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00JMJZ]Z\x10\x10\x10\x00\x00\x00\x00\x00B=B\xff\xff\xffRQR\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xf7\xf3\xf7\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x9c\x9e\x9c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xffBAB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xef\xf3\xef\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x9c\x9e\x9c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xffBAB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xef\xf3\xef\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x9c\xa2\x9c\xff\xff\xff\x19\x18\x19\x00\x00\x00\x00\x00B=B\xff\xff\xffBAB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xef\xf3\xef\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x10\xe6\xeb\xe6\xde\xdb\xde\x10\x0c\x10\x00\x00\x00\x00\x00B=B\xff\xff\xffBAB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xef\xf3\xef\xff\xfb\xff\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00ZUZ\xff\xff\xffcac\x00\x00\x00\x00\x00\x00\x00\x00:9:\xff\xff\xff\xb5\xb2\xb5\x84\x86\x84\x84\x86\x84\x84\x86\x84\x84\x86\x84\x84\x86\x84\x84\x86\x84\x84\x86\x84\x8c\x8a\x8c\xff\xff\xff\xce\xce\xce\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x0c\x10\xce\xca\xce\xff\xff\xff\x19\x14\x19\x00\x00\x00\x00\x00\x00\x00\x00\x08\x0c\x08\x9c\x9e\x9c\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffBEB\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\x14\x19suskik\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x19\x18\x19\x10\x10\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    static MOTOR_OFF_BMP: OnceCell<Bmp<'static, Rgb565>> = OnceCell::new();
                    let motor_off_bmp = MOTOR_OFF_BMP . get_or_init (| | unsafe { Bmp :: from_slice (b"BM.\x07\x00\x00\x00\x00\x00\x00\x8a\x00\x00\x00|\x00\x00\x00\x16\x00\x00\x00\x19\x00\x00\x00\x01\x00\x18\x00\x00\x00\x00\x00\xa4\x06\x00\x00#.\x00\x00#.\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\x00\x00\xff\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00BGRs\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x01\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\x0c\x0c\x0c\x00\x00\x00\x00\x00\x00\x01\x01\x01\x02\x02\x02\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x12\x12\x12\"\"\"\x15\x15\x15\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0f\x0e\x0f\"\"\"\"\"\"\x05\x06\x05\x00\x00\x00\x04\x04\x04\"\"\"\"\"\"\"\"\"\x0c\x0c\x0c\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\x17\x18\x17\"\"\"\t\t\t\x17\x17\x17\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00 ! \"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\"\"\"\"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\"\"\"\"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\"\"\"\"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\"\"\"\"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\"\"\"\"\"\"\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"\"\"\x0c\x0c\x0c\x0c\x0c\x0c\"\"\"   \x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"\"\"\"\"\"\"\"\"\"\"\"\x06\x07\x06\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"\"\"\x01\x01\x01\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x03\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00 ! \"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0e\r\x0e\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"   \x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\x1d\x1e\x1d\"\"\"\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x04\x03\x05\x06\x05\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x04\x04\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00   \"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x10\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x1f \x1f\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x10\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x1f \x1f\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x12\x10\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x1f \x1f\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x1d\x1e\x1d\x1c\x1b\x1c\x00\x00\x00\x00\x00\x00\x00\x00\x03\x02\x03\"\"\"\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x1f \x1f\"!\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x05\x05\x05\"\"\"\x06\x06\x06\x00\x00\x00\x00\x00\x00\x00\x00\x02\x02\x02\"\"\"\x15\x14\x15\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0c\x0e\r\x0e\"\"\"\x19\x19\x19\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\x19\x19\"\"\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x10\x10\x10\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\t\t\t\x07\x07\x07\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") . unwrap_unchecked () }) ;
                    let motor_im: Image<Bmp<Rgb565>> = Image::new(motor_bmp, position);
                    let motor_on_im: Image<Bmp<Rgb565>> =
                        Image::new(motor_on_bmp, position).translate(Point::new(5, 24));
                    let motor_off_im: Image<Bmp<Rgb565>> =
                        Image::new(motor_off_bmp, position).translate(Point::new(5, 24));
                    Self {
                        motor_im,
                        motor_on_im,
                        motor_off_im,
                        on: false,
                        needs_update: true,
                    }
                }
                pub fn update_on(&mut self, on: bool) -> bool {
                    if self.on != on {
                        self.on = on;
                        self.needs_update = true;
                    }
                    self.needs_update
                }
                pub fn is_redraw(&self) -> bool {
                    self.needs_update
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.needs_update {
                        self.motor_im.draw(&mut target.color_converted())?;
                        if self.on {
                            self.motor_on_im.draw(&mut target.color_converted())?;
                        } else {
                            self.motor_off_im.draw(&mut target.color_converted())?;
                        }
                        self.needs_update = false;
                    }
                    Ok(())
                }
            }
        }
        mod obd2_debug_selector {
            use core::{fmt::Write, str::FromStr as _};
            use defmt::info;
            use display_interface::DisplayError;
            use embassy_time::Instant;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::{
                debug::{DEBUG_CHANNEL_LEN, DEBUG_STRING_LEN},
                tasks::obd2::Obd2Debug,
            };
            pub struct Obd2DebugSelector {
                pids: heapless::FnvIndexMap<&'static str, Obd2Debug, 16>,
                redraw: bool,
            }
            #[automatically_derived]
            impl ::core::default::Default for Obd2DebugSelector {
                #[inline]
                fn default() -> Obd2DebugSelector {
                    Obd2DebugSelector {
                        pids: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                    }
                }
            }
            impl Obd2DebugSelector {
                pub fn new() -> Self {
                    Self {
                        pids: Default::default(),
                        redraw: false,
                    }
                }
                pub fn handle_obd2_debug(&mut self, debug: &Obd2Debug) {
                    if let Some(pid) = self.pids.get_mut(debug.type_id) {
                        pid.data = debug.data.clone();
                    } else {
                        self.pids.insert(debug.type_id, debug.clone()).ok();
                    }
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>, D2: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                    target2: &mut D2,
                ) -> Result<(), ()> {
                    if self.redraw {
                        target.clear(Gray4::BLACK).map_err(|_| ())?;
                        target2.clear(Gray4::BLACK).map_err(|_| ())?;
                        let character_style = MonoTextStyle::new(&PROFONT_7_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let mut position = Point::new(0, 6);
                        for (pid, buffer) in &self.pids {
                            let mut text = String::<64>::new();
                            if let Some(data) = &buffer.data {
                                text.write_fmt(format_args!("{0}: {1:x?}", pid, data)).ok();
                            } else {
                                text.write_fmt(format_args!("{0}: None", pid)).ok();
                            }
                            let text =
                                Text::with_text_style(&text, position, character_style, text_style);
                            text.draw(target).map_err(|_| ())?;
                            position += Point::new(0, 8);
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod position {
            use alloc::{borrow::Cow, string::ToString as _};
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                image::Image,
                pixelcolor::Gray4,
                prelude::*,
                primitives::{Rectangle, StyledDrawable as _},
            };
            use embedded_iconoir::prelude::IconoirNewIcon as _;
            pub struct Position {
                position: Point,
                redraw: bool,
                last_position: bool,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Position {
                #[inline]
                fn clone(&self) -> Position {
                    Position {
                        position: ::core::clone::Clone::clone(&self.position),
                        redraw: ::core::clone::Clone::clone(&self.redraw),
                        last_position: ::core::clone::Clone::clone(&self.last_position),
                    }
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Position {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "Position",
                        "position",
                        &self.position,
                        "redraw",
                        &self.redraw,
                        "last_position",
                        &&self.last_position,
                    )
                }
            }
            impl Default for Position {
                fn default() -> Self {
                    Self {
                        position: Point::zero(),
                        redraw: true,
                        last_position: false,
                    }
                }
            }
            impl Position {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        ..Default::default()
                    }
                }
                pub fn update_last_position(&mut self, last_position: bool) {
                    if self.last_position == last_position {
                        return;
                    }
                    self.last_position = last_position;
                    self.redraw = true;
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        if self.last_position {
                            let icon = embedded_iconoir::icons::size18px::maps::Position::new(
                                GrayColor::WHITE,
                            );
                            let image = Image::new(&icon, self.position);
                            image.draw(target)?;
                        } else {
                            let style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                                .stroke_width(0)
                                .stroke_color(Gray4::BLACK)
                                .fill_color(Gray4::BLACK)
                                .build();
                            let bounding_box = Rectangle::new(self.position, Size::new(18, 18));
                            bounding_box.draw_styled(&style, target)?;
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod power {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct Power {
                position: Point,
                power: f32,
                current: f32,
                redraw: bool,
                bounding_box: Option<Rectangle>,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Power {
                #[inline]
                fn clone(&self) -> Power {
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    let _: ::core::clone::AssertParamIsClone<Option<Rectangle>>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Power {}
            #[automatically_derived]
            impl ::core::fmt::Debug for Power {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field5_finish(
                        f,
                        "Power",
                        "position",
                        &self.position,
                        "power",
                        &self.power,
                        "current",
                        &self.current,
                        "redraw",
                        &self.redraw,
                        "bounding_box",
                        &&self.bounding_box,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::default::Default for Power {
                #[inline]
                fn default() -> Power {
                    Power {
                        position: ::core::default::Default::default(),
                        power: ::core::default::Default::default(),
                        current: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                        bounding_box: ::core::default::Default::default(),
                    }
                }
            }
            impl Power {
                pub fn new(position: Point) -> Self {
                    Self {
                        position,
                        power: 0.0,
                        current: 0.0,
                        redraw: true,
                        bounding_box: None,
                    }
                }
                pub fn update_power(&mut self, power: f32) {
                    if self.power != power {
                        self.power = power;
                        self.redraw = true;
                    }
                }
                pub fn update_current(&mut self, current: f32) {
                    if self.current != current {
                        self.current = current;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0:.2}kW", self.power / 1000.0))
                            .ok();
                        let character_style = MonoTextStyle::new(&PROFONT_12_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Center)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let text = Text::with_text_style(
                            text.as_str(),
                            self.position,
                            character_style,
                            text_style,
                        );
                        let new_bounding_box = text.bounding_box();
                        if new_bounding_box.size.width
                            > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0)
                        {
                            self.bounding_box = Some(new_bounding_box);
                        }
                        if let Some(bb) = self.bounding_box {
                            bb.draw_styled(
                                &PrimitiveStyleBuilder::new()
                                    .fill_color(Gray4::BLACK)
                                    .build(),
                                target,
                            )?;
                        }
                        text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod slider {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            pub struct Slider {
                percentage: f64,
                size: Size,
                position: Point,
                redraw: bool,
            }
            #[automatically_derived]
            impl ::core::default::Default for Slider {
                #[inline]
                fn default() -> Slider {
                    Slider {
                        percentage: ::core::default::Default::default(),
                        size: ::core::default::Default::default(),
                        position: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                    }
                }
            }
            impl Slider {
                pub fn new(position: Point, size: Size) -> Self {
                    Self {
                        position,
                        size,
                        percentage: 0.0,
                        redraw: true,
                    }
                }
                pub fn update_percentage(&mut self, percentage: f64) {
                    if self.percentage != percentage {
                        self.percentage = percentage;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let style = PrimitiveStyleBuilder::new()
                            .stroke_width(1)
                            .stroke_color(Gray4::WHITE)
                            .fill_color(Gray4::BLACK)
                            .build();
                        Rectangle::new(self.position, self.size).draw_styled(&style, target)?;
                        let mut bar_style = style;
                        bar_style.stroke_color = Some(Gray4::new(0x01));
                        bar_style.fill_color = Some(Gray4::new(0x01));
                        let mut bar_size = Size::new(
                            (self.size.width as f64 * self.percentage / 100.0).round() as u32,
                            self.size.height,
                        );
                        if bar_size.width > 0 {
                            let mut bar_style = style;
                            bar_style.stroke_width = 0;
                            bar_style.fill_color = Some(Gray4::new(0x01));
                            bar_size.width = bar_size.width.saturating_sub(4);
                            bar_size.height = bar_size.height.saturating_sub(4);
                            let mut bar_position = self.position;
                            bar_position.x += 2;
                            bar_position.y += 2;
                            Rectangle::new(bar_position, bar_size)
                                .draw_styled(&bar_style, target)?;
                        }
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod temperature {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct Temperature {
                max_temp: f32,
                min_temp: f32,
                current_temp: f32,
                current_temp_percentage: f32,
                size: Size,
                position: Point,
                bars: i32,
                redraw: bool,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Temperature {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    let names: &'static _ = &[
                        "max_temp",
                        "min_temp",
                        "current_temp",
                        "current_temp_percentage",
                        "size",
                        "position",
                        "bars",
                        "redraw",
                    ];
                    let values: &[&dyn ::core::fmt::Debug] = &[
                        &self.max_temp,
                        &self.min_temp,
                        &self.current_temp,
                        &self.current_temp_percentage,
                        &self.size,
                        &self.position,
                        &self.bars,
                        &&self.redraw,
                    ];
                    ::core::fmt::Formatter::debug_struct_fields_finish(
                        f,
                        "Temperature",
                        names,
                        values,
                    )
                }
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Temperature {
                #[inline]
                fn clone(&self) -> Temperature {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<Size>;
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<i32>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Temperature {}
            #[automatically_derived]
            impl ::core::default::Default for Temperature {
                #[inline]
                fn default() -> Temperature {
                    Temperature {
                        max_temp: ::core::default::Default::default(),
                        min_temp: ::core::default::Default::default(),
                        current_temp: ::core::default::Default::default(),
                        current_temp_percentage: ::core::default::Default::default(),
                        size: ::core::default::Default::default(),
                        position: ::core::default::Default::default(),
                        bars: ::core::default::Default::default(),
                        redraw: ::core::default::Default::default(),
                    }
                }
            }
            impl Temperature {
                pub fn new(position: Point, size: Size, min: f32, max: f32, bars: i32) -> Self {
                    Self {
                        position,
                        size,
                        current_temp: 0.0,
                        current_temp_percentage: 0.0,
                        max_temp: max,
                        min_temp: min,
                        bars,
                        redraw: true,
                    }
                }
                pub fn update_temp(&mut self, temp: f32) {
                    if self.current_temp != temp {
                        self.current_temp = temp;
                        self.current_temp_percentage =
                            (temp - self.min_temp) / (self.max_temp - self.min_temp);
                        if self.current_temp > self.max_temp {
                            self.current_temp_percentage = 1.0;
                        } else if self.current_temp < self.min_temp {
                            self.current_temp_percentage = 0.0;
                        }
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let color = Gray4::new(6);
                        let mut style = PrimitiveStyleBuilder::new()
                            .stroke_width(2)
                            .stroke_color(Gray4::WHITE)
                            .fill_color(Gray4::BLACK)
                            .build();
                        let mut size = self.size;
                        size.width /= 2;
                        size.height -= self.size.width / 2;
                        let mut area = Rectangle::new(
                            self.position + Point::new(self.size.width as i32 / 2, 0),
                            size,
                        );
                        area.draw_styled(&style, target)?;
                        let mut circle_bottom = Circle::with_center(
                            self.position
                                + Point::new(
                                    self.size.width as i32 / 4,
                                    self.size.height as i32 - (self.size.width as i32 / 2),
                                )
                                + Point::new(size.width as i32 - 1, -2),
                            self.size.width,
                        );
                        circle_bottom.draw_styled(&style, target)?;
                        let circle = Circle::with_center(
                            self.position + Point::new(self.size.width as i32 / 2 + 3, 4),
                            self.size.width / 2,
                        );
                        style.fill_color = Some(Gray4::BLACK);
                        let mut circle_box = circle.bounding_box();
                        circle_box.size.width += 2;
                        circle_box.size.height -= 1;
                        circle_box.top_left.x -= 1;
                        circle_box.top_left.y -= 1;
                        target.fill_solid(&circle_box, Gray4::BLACK)?;
                        circle.draw_styled(&style, target)?;
                        style.stroke_color = Some(color);
                        area.size.height -= 4;
                        area.size.width -= 2;
                        area.top_left.x += 1;
                        area.top_left.y += 3;
                        let mut area_clipped = target.clipped(&area.bounding_box());
                        area_clipped.fill_solid(&area, Gray4::BLACK)?;
                        let mut size = self.size;
                        size.width /= 2;
                        size.height = ((self.size.height as f32 - 8.0)
                            * self.current_temp_percentage)
                            .round() as u32;
                        size.width -= 6;
                        style.fill_color = Some(color);
                        let mut position =
                            self.position + Point::new(self.size.width as i32 / 2, 0);
                        position.x += 3;
                        position.y = self.position.y + (self.size.height - 6) as i32
                            - size.height as i32
                            + 2;
                        let mut area_filled = Rectangle::new(position, size);
                        area_filled.draw_styled(&style, target)?;
                        area_filled.size.width = self.size.width;
                        area_filled.top_left.x -= self.size.width as i32 / 2;
                        area_filled.top_left.y -= 1;
                        area_filled.size.height += 1;
                        let mut area_filled = target.clipped(&area_filled);
                        circle_bottom.diameter -= 6;
                        circle_bottom.top_left.y += 3;
                        circle_bottom.top_left.x += 3;
                        circle_bottom.draw_styled(&style, &mut area_filled)?;
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0:.0}°C", self.current_temp))
                            .ok();
                        let character_style = MonoTextStyle::new(&PROFONT_10_POINT, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let mut rotate_target = RotatedDrawTarget::new(target);
                        let text_position = Point::new(0, 256 - self.position.x + 5);
                        let text = Text::with_text_style(
                            text.as_str(),
                            text_position,
                            character_style,
                            text_style,
                        );
                        let text_box = Rectangle::with_center(
                            text_position - Point::new(1, 2),
                            Size::new(42, 12),
                        );
                        rotate_target.fill_solid(&text_box, Gray4::BLACK)?;
                        text.draw(&mut rotate_target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod text {
            use alloc::{borrow::Cow, string::ToString as _};
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoFont, MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text as EmbeddedText, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct Text {
                position: Point,
                font: &'static MonoFont<'static>,
                text: Cow<'static, str>,
                selected: bool,
                redraw: bool,
                bounding_box: Option<Rectangle>,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Text {
                #[inline]
                fn clone(&self) -> Text {
                    Text {
                        position: ::core::clone::Clone::clone(&self.position),
                        font: ::core::clone::Clone::clone(&self.font),
                        text: ::core::clone::Clone::clone(&self.text),
                        selected: ::core::clone::Clone::clone(&self.selected),
                        redraw: ::core::clone::Clone::clone(&self.redraw),
                        bounding_box: ::core::clone::Clone::clone(&self.bounding_box),
                    }
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Text {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    let names: &'static _ = &[
                        "position",
                        "font",
                        "text",
                        "selected",
                        "redraw",
                        "bounding_box",
                    ];
                    let values: &[&dyn ::core::fmt::Debug] = &[
                        &self.position,
                        &self.font,
                        &self.text,
                        &self.selected,
                        &self.redraw,
                        &&self.bounding_box,
                    ];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f, "Text", names, values)
                }
            }
            impl Default for Text {
                fn default() -> Self {
                    Self {
                        position: Point::zero(),
                        font: &PROFONT_9_POINT,
                        redraw: true,
                        selected: false,
                        bounding_box: None,
                        text: Cow::Borrowed(""),
                    }
                }
            }
            impl Text {
                pub fn new(
                    position: Point,
                    font: &'static MonoFont,
                    initial_str: Option<&'static str>,
                ) -> Self {
                    let mut ret = Self {
                        position,
                        redraw: true,
                        bounding_box: None,
                        font,
                        text: Cow::Borrowed(""),
                        selected: false,
                    };
                    if let Some(str) = initial_str {
                        ret.update_str(str);
                    }
                    ret
                }
                pub fn update_str(&mut self, str: &'static str) {
                    if self.text != str {
                        self.text = Cow::Borrowed(str);
                        self.redraw = true;
                    }
                }
                pub fn update_string(&mut self, str: &str) {
                    if self.text != str {
                        self.text = Cow::Owned(str.to_string());
                        self.redraw = true;
                    }
                }
                pub fn update_selected(&mut self, selected: bool) {
                    if self.selected != selected {
                        self.selected = selected;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let character_style = MonoTextStyle::new(&self.font, Gray4::WHITE);
                        let mut text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let mut text = EmbeddedText::with_text_style(
                            self.text.as_ref(),
                            self.position,
                            character_style,
                            text_style,
                        );
                        if self.selected {
                            text.character_style.background_color = Some(Gray4::WHITE);
                            text.character_style.text_color = Some(Gray4::BLACK);
                        }
                        let new_bounding_box = text.bounding_box();
                        if new_bounding_box.size.width
                            > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0)
                        {
                            self.bounding_box = Some(new_bounding_box);
                        }
                        if let Some(bb) = self.bounding_box {
                            bb.draw_styled(
                                &PrimitiveStyleBuilder::new()
                                    .fill_color(Gray4::BLACK)
                                    .build(),
                                target,
                            )?;
                        }
                        text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        mod value {
            use core::fmt::Write;
            use display_interface::DisplayError;
            use embedded_graphics::{
                mono_font::{
                    ascii::{FONT_10X20, FONT_6X10, FONT_6X13_BOLD, FONT_9X15_BOLD},
                    MonoFont, MonoTextStyle,
                },
                pixelcolor::Gray4,
                prelude::*,
                primitives::*,
                text::{Alignment, LineHeight, Text, TextStyleBuilder},
            };
            use heapless::String;
            use num_traits::float::FloatCore;
            use profont::*;
            use crate::display::RotatedDrawTarget;
            pub struct Value {
                position: Point,
                value: f32,
                font: &'static MonoFont<'static>,
                unit: &'static str,
                precision: usize,
                redraw: bool,
                bounding_box: Option<Rectangle>,
            }
            #[automatically_derived]
            impl ::core::clone::Clone for Value {
                #[inline]
                fn clone(&self) -> Value {
                    let _: ::core::clone::AssertParamIsClone<Point>;
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<&'static MonoFont<'static>>;
                    let _: ::core::clone::AssertParamIsClone<&'static str>;
                    let _: ::core::clone::AssertParamIsClone<usize>;
                    let _: ::core::clone::AssertParamIsClone<bool>;
                    let _: ::core::clone::AssertParamIsClone<Option<Rectangle>>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Value {}
            #[automatically_derived]
            impl ::core::fmt::Debug for Value {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    let names: &'static _ = &[
                        "position",
                        "value",
                        "font",
                        "unit",
                        "precision",
                        "redraw",
                        "bounding_box",
                    ];
                    let values: &[&dyn ::core::fmt::Debug] = &[
                        &self.position,
                        &self.value,
                        &self.font,
                        &self.unit,
                        &self.precision,
                        &self.redraw,
                        &&self.bounding_box,
                    ];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f, "Value", names, values)
                }
            }
            impl Default for Value {
                fn default() -> Self {
                    Self {
                        position: Point::zero(),
                        value: 0.0,
                        font: &PROFONT_9_POINT,
                        unit: "",
                        precision: 0,
                        redraw: true,
                        bounding_box: None,
                    }
                }
            }
            impl Value {
                pub fn new(
                    position: Point,
                    font: &'static MonoFont,
                    unit: &'static str,
                    precision: usize,
                ) -> Self {
                    Self {
                        position,
                        value: 0.0,
                        unit,
                        redraw: true,
                        bounding_box: None,
                        precision,
                        font,
                    }
                }
                pub fn update_value(&mut self, value: f32) {
                    if self.value != value {
                        self.value = value;
                        self.redraw = true;
                    }
                }
                pub fn draw<D: DrawTarget<Color = Gray4>>(
                    &mut self,
                    target: &mut D,
                ) -> Result<(), D::Error> {
                    if self.redraw {
                        let mut text: String<16> = String::new();
                        text.write_fmt(format_args!("{0:.1$}", self.value, self.precision))
                            .ok();
                        text.write_fmt(format_args!("{0}", self.unit)).ok();
                        let character_style = MonoTextStyle::new(&self.font, Gray4::WHITE);
                        let text_style = TextStyleBuilder::new()
                            .alignment(Alignment::Left)
                            .line_height(LineHeight::Percent(100))
                            .build();
                        let text = Text::with_text_style(
                            text.as_str(),
                            self.position,
                            character_style,
                            text_style,
                        );
                        let new_bounding_box = text.bounding_box();
                        if new_bounding_box.size.width
                            > self.bounding_box.map(|bb| bb.size.width).unwrap_or(0)
                        {
                            self.bounding_box = Some(new_bounding_box);
                        }
                        if let Some(bb) = self.bounding_box {
                            bb.draw_styled(
                                &PrimitiveStyleBuilder::new()
                                    .fill_color(Gray4::BLACK)
                                    .build(),
                                target,
                            )?;
                        }
                        text.draw(target)?;
                        self.redraw = false;
                    }
                    Ok(())
                }
            }
        }
        pub use arrow::{Arrow, ArrowDirection};
        pub use battery::{Battery, BatteryOrientation};
        pub use battery_12v::Battery12V;
        pub use connection::Connection;
        pub use debug::DebugScroll;
        pub use fuel::Fuel;
        pub use gearbox_gear::GearboxGear;
        pub use ice_fuel_rate::IceFuelRate;
        pub use icon::Icon;
        pub use motor_electric::MotorElectric;
        pub use motor_ice::MotorIce;
        pub use obd2_debug_selector::Obd2DebugSelector;
        pub use position::Position;
        pub use power::Power;
        pub use slider::Slider;
        pub use temperature::Temperature;
        pub use text::Text;
        pub use value::Value;
    }
    pub struct RotatedDrawTarget<'a, T>
    where
        T: DrawTarget,
    {
        parent: &'a mut T,
    }
    #[automatically_derived]
    impl<'a, T: ::core::fmt::Debug> ::core::fmt::Debug for RotatedDrawTarget<'a, T>
    where
        T: DrawTarget,
    {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "RotatedDrawTarget",
                "parent",
                &&self.parent,
            )
        }
    }
    impl<'a, T> RotatedDrawTarget<'a, T>
    where
        T: DrawTarget,
    {
        pub fn new(parent: &'a mut T) -> Self {
            Self { parent }
        }
    }
    impl<T> DrawTarget for RotatedDrawTarget<'_, T>
    where
        T: DrawTarget,
    {
        type Color = T::Color;
        type Error = T::Error;
        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            let parent_width = self.parent.bounding_box().size.width as i32;
            self.parent.draw_iter(
                pixels
                    .into_iter()
                    .map(|Pixel(p, c)| Pixel(Point::new(parent_width - p.y, p.x), c)),
            )
        }
    }
    impl<T> Dimensions for RotatedDrawTarget<'_, T>
    where
        T: DrawTarget,
    {
        fn bounding_box(&self) -> Rectangle {
            let parent_bb = self.parent.bounding_box();
            Rectangle::new(
                parent_bb.top_left,
                Size::new(parent_bb.size.height, parent_bb.size.width),
            )
        }
    }
}
mod locks {
    use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
    pub static SPI_BUS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());
}
mod debug {
    use defmt::*;
    use embassy_sync::{
        blocking_mutex::raw::CriticalSectionRawMutex,
        channel::{Channel, TrySendError},
    };
    use heapless::String;
    pub const DEBUG_STRING_LEN: usize = 120;
    pub const DEBUG_CHANNEL_LEN: usize = 16;
    static DEBUG_CHANNEL: Channel<
        CriticalSectionRawMutex,
        String<DEBUG_STRING_LEN>,
        DEBUG_CHANNEL_LEN,
    > = Channel::new();
    pub fn debug(string: String<DEBUG_STRING_LEN>) {
        if let Err(TrySendError::Full(string)) = DEBUG_CHANNEL.try_send(string) {
            let _ = DEBUG_CHANNEL.try_receive();
            DEBUG_CHANNEL.try_send(string).ok();
        }
    }
    pub async fn receive() -> String<DEBUG_STRING_LEN> {
        DEBUG_CHANNEL.receive().await
    }
    pub(crate) use internal_debug;
}
mod dummy_display {
    use embedded_graphics::{
        Pixel,
        pixelcolor::Gray4,
        prelude::{Dimensions, DrawTarget},
        primitives::Rectangle,
    };
    pub struct DummyDisplay;
    impl DummyDisplay {
        pub fn new() -> Self {
            DummyDisplay
        }
        pub async fn init(&mut self, conf: Option<i32>) -> Result<(), ()> {
            Ok(())
        }
        pub fn clear(&mut self) {}
        pub async fn flush(&mut self) -> Result<(), ()> {
            Ok(())
        }
        pub async fn sleep(&mut self, sleep: bool) -> Result<(), ()> {
            Ok(())
        }
        pub async fn set_contrast(&mut self, contrast: u8) -> Result<(), ()> {
            Ok(())
        }
        pub fn get_contrast(&self) -> u8 {
            10
        }
    }
    pub struct DisplayError;
    impl defmt::Format for DisplayError {
        fn format(&self, f: defmt::Formatter) {
            {
                match () {
                    () => {
                        if {
                            const CHECK: bool = {
                                const fn check() -> bool {
                                    let module_path = "display::dummy_display".as_bytes();
                                    if if 7usize > module_path.len() {
                                        false
                                    } else {
                                        module_path[0usize] == 100u8
                                            && module_path[1usize] == 105u8
                                            && module_path[2usize] == 115u8
                                            && module_path[3usize] == 112u8
                                            && module_path[4usize] == 108u8
                                            && module_path[5usize] == 97u8
                                            && module_path[6usize] == 121u8
                                            && if 7usize == module_path.len() {
                                                true
                                            } else {
                                                module_path[7usize] == b':'
                                            }
                                    } {
                                        return true;
                                    }
                                    false
                                }
                                check()
                            };
                            CHECK
                        } {
                            unsafe { defmt::export::acquire() };
                            defmt::export::header(&{
                                defmt::export::make_istr({
                                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"8638609346166957495\",\"crate_name\":\"display\"}"]
                                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"8638609346166957495\",\"crate_name\":\"display\"}"]
                                    static DEFMT_LOG_STATEMENT: u8 = 0;
                                    &DEFMT_LOG_STATEMENT as *const u8 as u16
                                })
                            });
                            unsafe { defmt::export::release() }
                        }
                    }
                };
                defmt::export::panic()
            }
        }
        fn _format_tag() -> defmt::Str {
            {
                defmt::export::make_istr({
                    #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"DisplayError\",\"disambiguator\":\"15941245811031122208\",\"crate_name\":\"display\"}"]
                    #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"DisplayError\",\"disambiguator\":\"15941245811031122208\",\"crate_name\":\"display\"}"]
                    static S: u8 = 0;
                    &S as *const u8 as u16
                })
            }
        }
        fn _format_data(&self) {
            match self {
                Self {} => {}
            }
        }
    }
    impl DrawTarget for DummyDisplay {
        type Color = Gray4;
        type Error = DisplayError;
        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            Ok(())
        }
    }
    impl Dimensions for DummyDisplay {
        fn bounding_box(&self) -> Rectangle {
            ::core::panicking::panic("not yet implemented")
        }
    }
}
mod hal {
    pub fn reset() {}
}
mod tasks {
    pub mod buttons {
        use defmt::Format;
        pub enum Button {
            B0,
            B1,
            B2,
            B3,
            B4,
            B5,
            B6,
            B7,
        }
        impl defmt::Format for Button {
            fn format(&self, f: defmt::Formatter) {
                {
                    match () {
                        () => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::tasks::buttons".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"4494834354432599388\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"4494834354432599388\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            }
            fn _format_tag() -> defmt::Str {
                {
                    defmt::export::make_istr({
                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"B0|B1|B2|B3|B4|B5|B6|B7\",\"disambiguator\":\"17023762970991115318\",\"crate_name\":\"display\"}"]
                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"B0|B1|B2|B3|B4|B5|B6|B7\",\"disambiguator\":\"17023762970991115318\",\"crate_name\":\"display\"}"]
                        static S: u8 = 0;
                        &S as *const u8 as u16
                    })
                }
            }
            fn _format_data(&self) {
                match self {
                    Button::B0 {} => {
                        defmt::export::u8(&0u8);
                    }
                    Button::B1 {} => {
                        defmt::export::u8(&1u8);
                    }
                    Button::B2 {} => {
                        defmt::export::u8(&2u8);
                    }
                    Button::B3 {} => {
                        defmt::export::u8(&3u8);
                    }
                    Button::B4 {} => {
                        defmt::export::u8(&4u8);
                    }
                    Button::B5 {} => {
                        defmt::export::u8(&5u8);
                    }
                    Button::B6 {} => {
                        defmt::export::u8(&6u8);
                    }
                    Button::B7 {} => {
                        defmt::export::u8(&7u8);
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Button {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Button {
            #[inline]
            fn eq(&self, other: &Button) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Button {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Button {
            #[inline]
            fn clone(&self) -> Button {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Button {}
        pub enum Action {
            Pressed(Button),
            Released(Button),
        }
        impl defmt::Format for Action
        where
            Button: defmt::Format,
        {
            fn format(&self, f: defmt::Formatter) {
                {
                    match () {
                        () => {
                            if {
                                const CHECK: bool = {
                                    const fn check() -> bool {
                                        let module_path = "display::tasks::buttons".as_bytes();
                                        if if 7usize > module_path.len() {
                                            false
                                        } else {
                                            module_path[0usize] == 100u8
                                                && module_path[1usize] == 105u8
                                                && module_path[2usize] == 115u8
                                                && module_path[3usize] == 112u8
                                                && module_path[4usize] == 108u8
                                                && module_path[5usize] == 97u8
                                                && module_path[6usize] == 121u8
                                                && if 7usize == module_path.len() {
                                                    true
                                                } else {
                                                    module_path[7usize] == b':'
                                                }
                                        } {
                                            return true;
                                        }
                                        false
                                    }
                                    check()
                                };
                                CHECK
                            } {
                                unsafe { defmt::export::acquire() };
                                defmt::export::header(&{
                                    defmt::export::make_istr({
                                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"1160295824769957655\",\"crate_name\":\"display\"}"]
                                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_error\",\"data\":\"panicked at 'internal error: entered unreachable code'\",\"disambiguator\":\"1160295824769957655\",\"crate_name\":\"display\"}"]
                                        static DEFMT_LOG_STATEMENT: u8 = 0;
                                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                                    })
                                });
                                unsafe { defmt::export::release() }
                            }
                        }
                    };
                    defmt::export::panic()
                }
            }
            fn _format_tag() -> defmt::Str {
                {
                    defmt::export::make_istr({
                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"Pressed({=?})|Released({=?})\",\"disambiguator\":\"9339512007380931501\",\"crate_name\":\"display\"}"]
                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_derived\",\"data\":\"Pressed({=?})|Released({=?})\",\"disambiguator\":\"9339512007380931501\",\"crate_name\":\"display\"}"]
                        static S: u8 = 0;
                        &S as *const u8 as u16
                    })
                }
            }
            fn _format_data(&self) {
                match self {
                    Action::Pressed { 0: arg0 } => {
                        defmt::export::u8(&0u8);
                        defmt::export::fmt(arg0);
                    }
                    Action::Released { 0: arg0 } => {
                        defmt::export::u8(&1u8);
                        defmt::export::fmt(arg0);
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Action {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Action {
            #[inline]
            fn eq(&self, other: &Action) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (Action::Pressed(__self_0), Action::Pressed(__arg1_0)) => {
                            __self_0 == __arg1_0
                        }
                        (Action::Released(__self_0), Action::Released(__arg1_0)) => {
                            __self_0 == __arg1_0
                        }
                        _ => unsafe { ::core::intrinsics::unreachable() },
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Action {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Button>;
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Action {
            #[inline]
            fn clone(&self) -> Action {
                let _: ::core::clone::AssertParamIsClone<Button>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Action {}
        pub fn init() {}
    }
    pub mod ieee802154 {
        use embassy_time::Instant;
        pub fn last_send() -> Option<Instant> {
            None
        }
        pub fn last_receive() -> Option<Instant> {
            None
        }
        pub fn last_position() -> Option<Instant> {
            None
        }
    }
    pub mod obd2 {
        pub async fn obd2_init_wait() {}
        pub struct Obd2Debug {
            pub type_id: &'static str,
            pub data: Option<alloc::vec::Vec<u8>>,
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Obd2Debug {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Obd2Debug {
            #[inline]
            fn eq(&self, other: &Obd2Debug) -> bool {
                self.type_id == other.type_id && self.data == other.data
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Obd2Debug {
            #[inline]
            fn clone(&self) -> Obd2Debug {
                Obd2Debug {
                    type_id: ::core::clone::Clone::clone(&self.type_id),
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
    }
    pub use crate::lcd;
}
mod types {
    use crate::dummy_display::DummyDisplay;
    pub type Display1 = DummyDisplay;
    pub type Display2 = DummyDisplay;
}
static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SERIAL: StaticCell<DefmtSerial> = StaticCell::new();
pub struct DefmtSerial {
    sender: Sender<u8>,
}
impl DefmtSerial {
    pub fn new(sender: Sender<u8>) -> Self {
        DefmtSerial { sender }
    }
}
impl defmt_serial::EraseWrite for DefmtSerial {
    fn write(&mut self, buf: &[u8]) {
        for byte in buf {
            self.sender.send(*byte).unwrap();
        }
    }
    fn flush(&mut self) {
        {
            ::std::io::_print(format_args!("Flushing\n"));
        };
    }
}
fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let bin = args.pop().unwrap();
    {
        ::std::io::_print(format_args!("Hello world!\n"));
    };
    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();
    defmt_serial::defmt_serial(SERIAL.init(DefmtSerial::new(tx)));
    std::thread::spawn(move || {
        let bin = std::fs::read(bin).unwrap();
        {
            ::std::io::_print(format_args!("Read {0} bytes\n", bin.len()));
        };
        let defmt_table = defmt_decoder::Table::parse(&bin).unwrap().unwrap();
        loop {
            let byte = rx.recv().unwrap();
            {
                ::std::io::_print(format_args!("Received byte: {0:?}\n", byte));
            };
            if let Ok(decode) = defmt_table.decode(&[byte]) {
                {
                    ::std::io::_print(format_args!("Decoded: {0:?}\n", decode));
                };
            } else {
                {
                    ::std::io::_print(format_args!("Failed to decode\n"));
                };
            }
        }
    });
    {
        match () {
            () => {
                unsafe {
                    defmt::export::acquire();
                }
                defmt::export::header(&{
                    defmt::export::make_istr({
                        #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_println\",\"data\":\"Hello world!wtffff\",\"disambiguator\":\"15133645062315211016\",\"crate_name\":\"display\"}"]
                        #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_println\",\"data\":\"Hello world!wtffff\",\"disambiguator\":\"15133645062315211016\",\"crate_name\":\"display\"}"]
                        static DEFMT_LOG_STATEMENT: u8 = 0;
                        &DEFMT_LOG_STATEMENT as *const u8 as u16
                    })
                });
                unsafe { defmt::export::release() }
            }
        }
    };
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();
    });
}
#[doc(hidden)]
async fn __run_task() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
        {
            ::std::io::_print(format_args!("Hello, world!\n"));
        };
        match () {
            () => {
                if {
                    const CHECK: bool = {
                        const fn check() -> bool {
                            let module_path = "display".as_bytes();
                            if if 7usize > module_path.len() {
                                false
                            } else {
                                module_path[0usize] == 100u8
                                    && module_path[1usize] == 105u8
                                    && module_path[2usize] == 115u8
                                    && module_path[3usize] == 112u8
                                    && module_path[4usize] == 108u8
                                    && module_path[5usize] == 97u8
                                    && module_path[6usize] == 121u8
                                    && if 7usize == module_path.len() {
                                        true
                                    } else {
                                        module_path[7usize] == b':'
                                    }
                            } {
                                return true;
                            }
                            false
                        }
                        check()
                    };
                    CHECK
                } {
                    unsafe { defmt::export::acquire() };
                    defmt::export::header(&{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"looop defmt\",\"disambiguator\":\"8113616965865383264\",\"crate_name\":\"display\"}"]
                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_info\",\"data\":\"looop defmt\",\"disambiguator\":\"8113616965865383264\",\"crate_name\":\"display\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    });
                    unsafe { defmt::export::release() }
                }
            }
        };
        {
            match () {
                () => {
                    unsafe {
                        defmt::export::acquire();
                    }
                    defmt::export::header(&{
                        defmt::export::make_istr({
                            #[link_section = ".defmt.{\"package\":\"display\",\"tag\":\"defmt_println\",\"data\":\"wtffff\",\"disambiguator\":\"2147858076432396002\",\"crate_name\":\"display\"}"]
                            #[export_name = "{\"package\":\"display\",\"tag\":\"defmt_println\",\"data\":\"wtffff\",\"disambiguator\":\"2147858076432396002\",\"crate_name\":\"display\"}"]
                            static DEFMT_LOG_STATEMENT: u8 = 0;
                            &DEFMT_LOG_STATEMENT as *const u8 as u16
                        })
                    });
                    unsafe { defmt::export::release() }
                }
            }
        };
    }
}
fn run() -> ::embassy_executor::SpawnToken<impl Sized> {
    trait _EmbassyInternalTaskTrait {
        type Fut: ::core::future::Future + 'static;
        fn construct() -> Self::Fut;
    }
    impl _EmbassyInternalTaskTrait for () {
        type Fut = impl core::future::Future + 'static;
        fn construct() -> Self::Fut {
            __run_task()
        }
    }
    const POOL_SIZE: usize = 1;
    static POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();
    unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct()) }
}
