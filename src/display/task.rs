use defmt::info;
use display_interface_spi::SPIInterface;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    dma::*,
    gpio::*,
    peripherals::*,
    spi::{master::dma::SpiDma, FullDuplexMode},
};
use sh1122::{async_display::buffered_graphics::AsyncBufferedGraphicsMode, AsyncDisplay};

use super::widgets::*;
use crate::types::*;

#[embassy_executor::task]
pub async fn run4(mut display1: Sh1122<10>, mut display2: Sh1122<1>) {
    // Get a region covering the entire display area, and clear it by writing all zeros.

    display1.init(None).await.unwrap();
    display2.init(None).await.unwrap();

    display1.clear();
    display2.clear();
    let mut battery_state = 32.0;

    let mut battery2 = Battery::new(
        Point::new(9, 1),
        Size::new(128, 62),
        BatteryOrientation::HorizontalRight,
        Some(Size::new(8, 32)),
        4,
        true,
    );
    let mut arrow_direction = ArrowDirection::Reverse;
    let mut arrow =
        Arrow::new(Point { x: 9 + 128, y: 64 / 2 - 9 }, Size { width: 54, height: 16 }, 14, arrow_direction);

    let mut motor_electric = MotorElectric::new(Point::new(256 - 60, 0));
    let mut motor_ice = MotorIce::new(Point::new(0, 0));

    let mut ice_temperature = Temperature::new(Point::new(256 - 21, 0), Size::new(16, 64), 0.0, 130.0, 4);
    let mut battery_12 = Battery12V::new(Point::new(256 - 41 - 22, 31));

    let mut power = Power::new(Point::new(128 + 36, 14));

    let mut i = 0;
    let mut direction = true;
    let mut updates = 0;
    let mut now = Instant::now();
    let mut now2 = Instant::now();
    let mut voltage = 12.0;

    battery2.draw(&mut display2).unwrap();
    arrow.draw(&mut display2).unwrap();
    power.draw(&mut display2).unwrap();
    ice_temperature.draw(&mut display1).unwrap();
    battery_12.draw(&mut display1).unwrap();

    motor_electric.update_on(true);
    motor_electric.draw(&mut display2).unwrap();

    motor_ice.update_on(true);
    motor_ice.draw(&mut display1).unwrap();

    battery2.update_voltage(360.1);
    battery2.update_temp(35.2);

    let mut speed = 0.0;
    arrow.update_speed(speed);

    display1.flush().await.unwrap();
    display2.flush().await.unwrap();

    let mut ice_temp = 20.0;
    loop {
        Timer::after(Duration::from_millis(10)).await;
        if i >= 256 - 64 && direction {
            direction = false;
        } else if i == 0 && !direction {
            direction = true;
        }

        if direction {
            i += 1;
        } else {
            i -= 1;
        }
        updates += 1;
        if now.elapsed().as_millis() > 10000 {
            info!("fps: {}", updates / 10);
            updates = 0;
            now = Instant::now();
        }

        if now2.elapsed().as_millis() > 100 {
            now2 = Instant::now();
            battery_state += 2.0;
            if battery_state > 100.0 {
                battery_state = 0.0;
                speed += 20.0;
                esp_println::println!("setting speed to: {:?}", speed);
                if speed > 100.0 {
                    if let ArrowDirection::Forward = arrow_direction {
                        arrow_direction = ArrowDirection::Reverse;
                        motor_electric.update_on(true);
                        motor_ice.update_on(false);
                    } else {
                        arrow_direction = ArrowDirection::Forward;
                        motor_electric.update_on(false);
                        motor_ice.update_on(true);
                    }
                    speed = 20.0;
                    arrow.update_direction(arrow_direction);
                }
                arrow.update_speed(speed);
            }
            ice_temp += 1.0;
            if ice_temp >= 140.0 {
                ice_temp = 0.0;
            }

            voltage += 0.05;
            if voltage > 15.0 {
                voltage = 8.0;
            }

            ice_temperature.update_temp(ice_temp);
            ice_temperature.draw(&mut display1).unwrap();
            motor_electric.draw(&mut display2).unwrap();
            motor_ice.draw(&mut display1).unwrap();
            arrow.draw(&mut display2).unwrap();
            battery2.update_percentage(battery_state);
            battery_12.update_voltage(voltage);
            battery_12.draw(&mut display1).unwrap();

            battery2.draw(&mut display2).unwrap();
            display2.flush().await.unwrap();
            display1.flush().await.unwrap();
        }
    }
}
