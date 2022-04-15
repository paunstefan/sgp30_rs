use embedded_hal_mock::{delay, i2c};
use sgp30_rs::*;
use simple_crc::simple_crc8;

const I2C_ADDRESS: u8 = 0x58;

fn calculate_crc(data: &[u8]) -> u8 {
    simple_crc8(data, 0x31, 0xFF, false, false, 0x00)
}

#[test]
fn test_feature_set_good() {
    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x2F]),
        i2c::Transaction::read(I2C_ADDRESS, vec![0x00, 0x22, calculate_crc(&[0x00, 0x22])]),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let (typ, version) = sensor.get_feature_set(&mut delay).unwrap();
    assert_eq!(typ, 0);
    assert_eq!(version, 0x22);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_feature_set_bad_crc() {
    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x2F]),
        i2c::Transaction::read(I2C_ADDRESS, vec![0x00, 0x22, !calculate_crc(&[0x00, 0x22])]),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let version = sensor.get_feature_set(&mut delay);
    assert_eq!(Err(Error::InvalidCrc), version);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_sensor_init() {
    let expectations = vec![i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x03])];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    sensor.iaq_init(&mut delay).unwrap();

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_measure_aq() {
    let co2_be = 400u16.to_be_bytes();
    let tvoc_be = 0u16.to_be_bytes();

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x08]),
        i2c::Transaction::read(
            I2C_ADDRESS,
            vec![
                co2_be[0],
                co2_be[1],
                calculate_crc(&co2_be),
                tvoc_be[0],
                tvoc_be[1],
                calculate_crc(&tvoc_be),
            ],
        ),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let (co2, tvoc) = sensor.measure_iaq(&mut delay).unwrap();
    assert_eq!(co2, 400);
    assert_eq!(tvoc, 0);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_get_baseline() {
    let co2_be = 123u16.to_be_bytes();
    let tvoc_be = 321u16.to_be_bytes();

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x15]),
        i2c::Transaction::read(
            I2C_ADDRESS,
            vec![
                co2_be[0],
                co2_be[1],
                calculate_crc(&co2_be),
                tvoc_be[0],
                tvoc_be[1],
                calculate_crc(&tvoc_be),
            ],
        ),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let (co2, tvoc) = sensor.get_iaq_baseline(&mut delay).unwrap();
    assert_eq!(co2, 123);
    assert_eq!(tvoc, 321);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_set_baseline() {
    let co2_be = 123u16.to_be_bytes();
    let tvoc_be = 321u16.to_be_bytes();

    let expectations = vec![i2c::Transaction::write(
        I2C_ADDRESS,
        vec![
            0x20,
            0x1E,
            co2_be[0],
            co2_be[1],
            calculate_crc(&co2_be),
            tvoc_be[0],
            tvoc_be[1],
            calculate_crc(&tvoc_be),
        ],
    )];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    sensor.set_iaq_baseline(&mut delay, (123, 321)).unwrap();

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_get_inceptive_baseline() {
    let tvoc_be = 321u16.to_be_bytes();

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0xB3]),
        i2c::Transaction::read(
            I2C_ADDRESS,
            vec![tvoc_be[0], tvoc_be[1], calculate_crc(&tvoc_be)],
        ),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let tvoc = sensor.get_tvoc_inceptive_baseline(&mut delay).unwrap();
    assert_eq!(tvoc, 321);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_set_inceptive_baseline() {
    let tvoc_be = 321u16.to_be_bytes();

    let expectations = vec![i2c::Transaction::write(
        I2C_ADDRESS,
        vec![0x20, 0x77, tvoc_be[0], tvoc_be[1], calculate_crc(&tvoc_be)],
    )];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    sensor.set_tvoc_baseline(&mut delay, 321).unwrap();

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_measure_raw() {
    let raw_be = 321u16.to_be_bytes();

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x50]),
        i2c::Transaction::read(
            I2C_ADDRESS,
            vec![raw_be[0], raw_be[1], calculate_crc(&raw_be)],
        ),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let data = sensor.measure_raw(&mut delay).unwrap();
    assert_eq!(data, 321);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_measure_test() {
    let data = [0xD4, 0x00];

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x20, 0x32]),
        i2c::Transaction::read(I2C_ADDRESS, vec![data[0], data[1], calculate_crc(&data)]),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let res = sensor.measure_test(&mut delay).unwrap();
    assert!(res);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_get_serial() {
    let serial = 0xAABBCCDDEEFFu64;
    let serial_be = serial.to_be_bytes();

    let mut data = vec![0u8; 9];
    data[0..2].copy_from_slice(&serial_be[2..4]);
    data[2] = calculate_crc(&serial_be[2..4]);
    data[3..5].copy_from_slice(&serial_be[4..6]);
    data[5] = calculate_crc(&serial_be[4..6]);
    data[6..8].copy_from_slice(&serial_be[6..=7]);
    data[8] = calculate_crc(&serial_be[6..=7]);

    let expectations = vec![
        i2c::Transaction::write(I2C_ADDRESS, vec![0x36, 0x82]),
        i2c::Transaction::read(I2C_ADDRESS, data),
    ];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    let res = sensor.get_serial_id(&mut delay).unwrap();
    assert_eq!(res, serial);

    let mut mock = sensor.destroy();
    mock.done();
}

#[test]
fn test_set_humidity() {
    let humidity = 50u8;

    let expectations = vec![i2c::Transaction::write(
        I2C_ADDRESS,
        vec![0x20, 0x61, humidity, 0, calculate_crc(&[humidity, 0])],
    )];

    let mock = i2c::Mock::new(&expectations);

    let mut sensor = Sgp30::init(mock);

    let mut delay = delay::MockNoop::new();

    sensor.set_absolute_humidity(&mut delay, humidity).unwrap();

    let mut mock = sensor.destroy();
    mock.done();
}
