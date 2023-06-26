use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi as SvcWifi};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use log::{error, info};

pub struct Wifi;

impl Wifi {
    pub fn init(modem: Modem) -> EspWifi<'static> {
        let mut wifi_driver = EspWifi::new(
            modem,
            EspSystemEventLoop::take().expect("Failed to take system event loop"),
            Some(EspDefaultNvsPartition::take().expect("Failed to take default nvs partition")),
        )
        .expect("Failed to create esp wifi device");

        wifi_driver
            .set_configuration(&Configuration::Client(ClientConfiguration {
                // See .cargo/config.toml to set WIFI_SSID and WIFI_PWD env variables
                ssid: env!("WIFI_SSID").into(),
                password: env!("WIFI_PWD").into(),
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            }))
            .expect("Failed to set wifi driver configuration");

        wifi_driver
    }

    pub fn start(wifi_driver: &mut EspWifi<'_>) {
        wifi_driver.start().expect("Failed to start wifi driver");

        loop {
            match wifi_driver.is_started() {
                Ok(true) => {
                    #[cfg(debug_assertions)]
                    info!("Wifi driver started");
                    break;
                }
                Ok(false) => {
                    #[cfg(debug_assertions)]
                    info!("Waiting for wifi driver to start")
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    error!("Error while starting wifi driver: {_e:?}")
                }
            }
        }

        loop {
            match wifi_driver.is_connected() {
                Ok(true) => {
                    #[cfg(debug_assertions)]
                    info!("Wifi is connected");
                    break;
                }
                Ok(false) => {
                    #[cfg(debug_assertions)]
                    info!("Waiting for Wifi connection")
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    error!("Failed to connect wifi driver: {_e:?}")
                }
            }

            if let Err(_e) = wifi_driver.connect() {
                #[cfg(debug_assertions)]
                error!("Error while connecting wifi driver: {_e:?}")
            }

            FreeRtos::delay_ms(1000);
        }
    }
}
