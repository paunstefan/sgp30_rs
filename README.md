# sgp30_rs

This is a platform agnostic driver for the SGP30 sensor, using `embedded-hal` traits.

SGP30 is a gas sensor that can measure CO2 and TVOC(Total Volatile Organic Compounds) concentrations in air.

## Features

The SGP30 sensor uses an I2C interface to get the measurements and isssue commands to the sensor. It uses the `0x58` I2C address.

Available commands:

* `iaq_init`: Initializes the sensor to start the air quality measurements.
* `measure_iaq`: Sends the measured values to the user. Should be called at 1 second intervals after the `iaq_init` to ensure proper operation. It will return the CO2(ppm) and TVOC(ppb) 16bit values. It will return 400ppm CO2 and 0ppb TVOC for the first 15 seconds, before calibration.
* `get_iaq_baseline`: Sends the calculated baseline values used for the compensation algorithm and they get optimized over time. Can be saved in non-volatile memory for later use if sensor gets powered off.
* `set_iaq_baseline`: Set the baseline values used by the sensor. You should use the values read in a previous run using the `get_iaq_baseline` command.
* `set_absolute_humidity`: The air quality measurements can get influenced by the humidity in the air, so the sensor can take that into consideration. You can send to the sensor the absolute humidity(g/m3) as an 8bit value. A current limitation of the driver is the lack of support for fractional values.
* `measure_test`: Used for testing the sensor. Should return a value of `0xD400`.
* `get_feature_set`: Returns the current sensor version a pair of product type (0 for the SGP30) and the product version.
* `measure_raw`: Returns the raw values read by the sensor, before applying the calibration algorithms.
* `get_tvoc_inceptive_baseline`: Used as a starting point for the baseline calculation for better accuracy. Only works for TVOC.
* `set_tvoc_baseline`: Sets the value used by the TVOC inceptive baseline described above.
* `get_serial_id`: Returns the serial ID of the sensor.


### CRC
Every 2 bytes read or written to the sensor should be followed by a CRC. The parameters are the following:

* Width: 8bit
* Polynomial: 0x31
* Initialization: 0xFF
* Reflect in: false
* Reflect out: false
* Final XOR: 0x00

## Usage

### References

* [Datasheet](https://sensirion.com/media/documents/984E0DD5/61644B8B/Sensirion_Gas_Sensors_Datasheet_SGP30.pdf)