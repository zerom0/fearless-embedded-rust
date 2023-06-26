use ds18b20::{Ds18b20, Resolution};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::PinDriver,
    prelude::Peripherals,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::{error, info, warn};
use one_wire_bus::{OneWire, OneWireError};
use serde::Serialize;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Initialize one-wire-bus on GPIO3
    let peripherals = Peripherals::take().unwrap();
    let driver = PinDriver::input_output_od(peripherals.pins.gpio3).unwrap();
    let mut one_wire_bus = OneWire::new(driver).unwrap();

    loop {
        // Temperature measurement on one-wire-bus
        match measure_temperature(&mut one_wire_bus) {
            Ok(measurement) => send(&measurement).unwrap(),
            Err(MeasurementError::NoDeviceFound) => warn!("No device found on one-wire-bus"),
            Err(err) => error!("{:?}", err),
        }

        FreeRtos::delay_ms(10_000); // Wait until the data is sent
    }
}

fn send(measurement: &Measurement) -> std::io::Result<()> {
    let message = serde_json::to_string_pretty(&measurement)?;
    info!("{}", message);
    Ok(())
}

#[derive(Serialize)]
struct Measurement {
    device_id: String,
    temperature: f32,
}

fn measure_temperature<P, E>(
    one_wire_bus: &mut OneWire<P>,
) -> Result<Measurement, MeasurementError<E>>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
{
    ds18b20::start_simultaneous_temp_measurement(one_wire_bus, &mut Ets)?;

    Resolution::Bits12.delay_for_measurement_time(&mut FreeRtos);

    if let Some((device_address, _)) = one_wire_bus.device_search(None, false, &mut Ets)? {
        let sensor = Ds18b20::new::<E>(device_address)?;
        let sensor_data = sensor.read_data(one_wire_bus, &mut Ets)?;
        return Ok(Measurement {
            device_id: format!("{:?}", device_address),
            temperature: sensor_data.temperature,
        });
    }

    Err(MeasurementError::NoDeviceFound)
}

// When performing a measurement it can happen that no device was found on the one-wire-bus
// in addition to the bus errors. Therefore we extend the error cases for proper error handling.
#[derive(Debug)]
enum MeasurementError<E> {
    OneWireError(OneWireError<E>),
    NoDeviceFound,
}

impl<E> From<OneWireError<E>> for MeasurementError<E> {
    fn from(value: OneWireError<E>) -> Self {
        MeasurementError::OneWireError(value)
    }
}
