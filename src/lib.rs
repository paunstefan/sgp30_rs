//! Generic no_std driver for the SGP30 gas sensor
//!
//! ## Usage:
//!
//!
//!
//! ## References:
//! * [Datasheet](https://sensirion.com/media/documents/984E0DD5/61644B8B/Sensirion_Gas_Sensors_Datasheet_SGP30.pdf)

#![no_std]
#![deny(missing_docs, trivial_casts, trivial_numeric_casts)]
use embedded_hal::blocking::{delay, i2c};
use simple_crc::simple_crc8;

const I2C_ADDRESS: u8 = 0x58;

/// Sensor struct.
/// Encapsulates the I2C bus type.
#[derive(Debug)]
pub struct Sgp30<I>
where
    I: i2c::Read + i2c::Write,
{
    bus: I,
}

/// Error type for the driver
#[derive(Debug, Eq, PartialEq)]
pub enum Error<E> {
    /// Encapsulation for I2C errors.
    I2c(E),
    /// Error variant for invalid CRC values.
    InvalidCrc,
    /// Fixed point conversion out of range
    FixedPointError,
}

impl<I, E> Sgp30<I>
where
    I: i2c::Read<Error = E> + i2c::Write<Error = E>,
    E: core::fmt::Debug,
{
    /// Create the sensor structure
    /// Be careful, this does not initialize the sensor
    /// that can be done using the `iaq_init` function
    pub fn init(i2c: I) -> Self {
        Sgp30 { bus: i2c }
    }

    /// Get back ownership of the I2C bus
    pub fn destroy(self) -> I {
        self.bus
    }

    /// Used after sensor power up to initialize the measurements
    pub fn iaq_init(&mut self, delay: &mut impl delay::DelayMs<u16>) -> Result<(), Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x03])
            .map_err(Error::I2c)?;

        delay.delay_ms(10);

        Ok(())
    }

    /// Read the measured values.
    /// Should be used at 1 second intervals to keep calibration.
    /// Returns a pair of CO2 ppm and TVOC ppb.
    pub fn measure_iaq(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
    ) -> Result<(u16, u16), Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x08])
            .map_err(Error::I2c)?;

        delay.delay_ms(12);

        let mut buffer = [0u8; 6];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] || calculate_crc(&buffer[3..5]) != buffer[5] {
            return Err(Error::InvalidCrc);
        }

        let co2 = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let tvoc = u16::from_be_bytes(buffer[3..5].try_into().unwrap());

        Ok((co2, tvoc))
    }

    /// Baseline values used for calibration.
    /// Returns pair of CO2 and TVOC baselines.
    pub fn get_iaq_baseline(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
    ) -> Result<(u16, u16), Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x15])
            .map_err(Error::I2c)?;

        delay.delay_ms(10);

        let mut buffer = [0u8; 6];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] || calculate_crc(&buffer[3..5]) != buffer[5] {
            return Err(Error::InvalidCrc);
        }

        let co2_baseline = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let tvoc_baseline = u16::from_be_bytes(buffer[3..5].try_into().unwrap());

        Ok((co2_baseline, tvoc_baseline))
    }

    /// Set the baseline values to use after sensor power up.
    pub fn set_iaq_baseline(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
        baseline: (u16, u16),
    ) -> Result<(), Error<E>> {
        let co2_baseline = baseline.0.to_be_bytes();
        let tvoc_baseline = baseline.1.to_be_bytes();

        let mut buffer = [0u8; 8];
        buffer[0..2].copy_from_slice(&[0x20, 0x1E]);

        buffer[2..4].copy_from_slice(&co2_baseline);
        buffer[4] = calculate_crc(&co2_baseline);
        buffer[5..7].copy_from_slice(&tvoc_baseline);
        buffer[7] = calculate_crc(&tvoc_baseline);

        self.bus.write(I2C_ADDRESS, &buffer).map_err(Error::I2c)?;

        delay.delay_ms(10);

        Ok(())
    }

    /// Special baseline value used for better accuracy of TVOC measurements.
    pub fn get_tvoc_inceptive_baseline(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
    ) -> Result<u16, Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0xB3])
            .map_err(Error::I2c)?;

        delay.delay_ms(10);

        let mut buffer = [0u8; 3];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] {
            return Err(Error::InvalidCrc);
        }

        let tvoc_baseline = u16::from_be_bytes(buffer[0..2].try_into().unwrap());

        Ok(tvoc_baseline)
    }

    /// Set the TVOC inceptive baseline.
    pub fn set_tvoc_baseline(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
        baseline: u16,
    ) -> Result<(), Error<E>> {
        let tvoc_baseline = baseline.to_be_bytes();

        let mut buffer = [0u8; 5];
        buffer[0..2].copy_from_slice(&[0x20, 0x77]);

        buffer[2..4].copy_from_slice(&tvoc_baseline);
        buffer[4] = calculate_crc(&tvoc_baseline);

        self.bus.write(I2C_ADDRESS, &buffer).map_err(Error::I2c)?;

        delay.delay_ms(10);

        Ok(())
    }

    /// Returns the raw measurement value, as it is before compensation algorithms.
    pub fn measure_raw(&mut self, delay: &mut impl delay::DelayMs<u16>) -> Result<u16, Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x50])
            .map_err(Error::I2c)?;

        delay.delay_ms(25);

        let mut buffer = [0u8; 3];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] {
            return Err(Error::InvalidCrc);
        }

        let data = u16::from_be_bytes(buffer[0..2].try_into().unwrap());

        Ok(data)
    }

    /// Used for testing. Should return `0xD400`
    pub fn measure_test(&mut self, delay: &mut impl delay::DelayMs<u16>) -> Result<bool, Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x32])
            .map_err(Error::I2c)?;

        delay.delay_ms(220);

        let mut buffer = [0u8; 3];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] {
            return Err(Error::InvalidCrc);
        }

        let mut ret = true;

        if buffer[0] != 0xD4 || buffer[1] != 0x00 {
            ret = false;
        }

        Ok(ret)
    }

    /// Sets the absolute humidity value in g/m3.
    pub fn set_absolute_humidity(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
        humidity: f32,
    ) -> Result<(), Error<E>> {
        let number = match f32_to_fixed_point(humidity) {
            None => return Err(Error::FixedPointError),
            Some(n) => n.to_be_bytes(),
        };

        let mut buffer = [0u8; 5];
        buffer[0..2].copy_from_slice(&[0x20, 0x61]);
        buffer[2] = number[0];
        buffer[3] = number[1];
        buffer[4] = calculate_crc(&buffer[2..4]);
        self.bus.write(I2C_ADDRESS, &buffer).map_err(Error::I2c)?;

        delay.delay_ms(10);

        Ok(())
    }

    /// Returns product type and product version of the sensor.
    pub fn get_feature_set(
        &mut self,
        delay: &mut impl delay::DelayMs<u16>,
    ) -> Result<(u8, u8), Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x20, 0x2F])
            .map_err(Error::I2c)?;

        delay.delay_ms(10);

        let mut buffer = [0u8; 3];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2] {
            return Err(Error::InvalidCrc);
        }

        let product_type = buffer[0] >> 4;
        let product_version = buffer[1];

        Ok((product_type, product_version))
    }

    /// Returns the serial ID of the sensor.
    pub fn get_serial_id(&mut self, delay: &mut impl delay::DelayUs<u16>) -> Result<u64, Error<E>> {
        self.bus
            .write(I2C_ADDRESS, &[0x36, 0x82])
            .map_err(Error::I2c)?;

        delay.delay_us(500);

        let mut buffer = [0u8; 9];
        self.bus
            .read(I2C_ADDRESS, &mut buffer)
            .map_err(Error::I2c)?;

        if calculate_crc(&buffer[0..2]) != buffer[2]
            || calculate_crc(&buffer[3..5]) != buffer[5]
            || calculate_crc(&buffer[6..8]) != buffer[8]
        {
            return Err(Error::InvalidCrc);
        }

        let mut serial_id_be = [0u8; 8];

        serial_id_be[2..4].copy_from_slice(&buffer[0..2]);
        serial_id_be[4..6].copy_from_slice(&buffer[3..5]);
        serial_id_be[6..=7].copy_from_slice(&buffer[6..8]);

        let serial = u64::from_be_bytes(serial_id_be);

        Ok(serial)
    }
}

/// Calculates CRC according to the datasheet
///
/// Polynomial = 0x31
/// Initialization = 0xFF
/// Reflect in = false
/// Reflect out = false
/// Final XOR = 0x00
fn calculate_crc(data: &[u8]) -> u8 {
    simple_crc8(data, 0x31, 0xFF, false, false, 0x00)
}

/// Convert f32 number to 8.8bit fixed point representation.
fn f32_to_fixed_point(value: f32) -> Option<u16> {
    const MULTIPLE: f32 = 1.0 / 256.0;

    let integer_part: u16 = value as u16;
    let fractional_part: u16 = ((value - integer_part as f32) / MULTIPLE) as u16;

    if integer_part > 0xFF || fractional_part > 0xFF {
        return None;
    }

    Some((integer_part << 8) | fractional_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc() {
        assert_eq!(calculate_crc(&[0xBE, 0xEF]), 0x92);
    }

    #[test]
    fn test_conversion_good() {
        assert_eq!(f32_to_fixed_point(0.0).unwrap(), 0x0);
        assert_eq!(f32_to_fixed_point(15.5).unwrap(), 0x0F80);
    }

    #[test]
    fn test_conversion_bad() {
        assert_eq!(f32_to_fixed_point(555.0), None);
    }
}
