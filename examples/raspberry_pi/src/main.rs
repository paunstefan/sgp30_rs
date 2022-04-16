use embedded_hal::prelude::*;
use linux_embedded_hal::{Delay, I2cdev};
use sgp30_rs::Sgp30;

fn main() {
    let mut delay = Delay;
    let dev = I2cdev::new("/dev/i2c-1").unwrap();

    let mut sensor = Sgp30::init(dev);

    sensor.iaq_init(&mut delay).unwrap();
    println!("Sensor initialized!");

    loop {
        let (co2, tvoc) = sensor.measure_iaq(&mut delay).unwrap();
        println!("CO2: {}ppm; TVOC: {}ppb", co2, tvoc);
        delay.delay_ms(1000u32);
    }
}
