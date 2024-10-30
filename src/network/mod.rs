use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{modem::Modem, peripheral::Peripheral},
    sys::{nvs_flash_init, EspError},
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

pub fn establish_wifi_connection(
    ssid: &str,
    password: &str,
    modem: impl Peripheral<P = Modem> + 'static,
) -> Result<EspWifi<'static>, EspError> {
    unsafe {
        // "otherwise you get this warning: Call nvs_flash_init before starting WiFi/BT"
        nvs_flash_init();
    }

    // this code copied from https://github.com/esp-rs/std-training/blob/main/common/lib/wifi/src/lib.rs
    let system_loop = EspSystemEventLoop::take()?;

    let mut auth_method = AuthMethod::WPA2Personal;

    if ssid.is_empty() {
        info!("[network] wifi ssid is empty")
    }

    if password.is_empty() {
        auth_method = AuthMethod::None;
        info!("[network] wifi password is empty");
    }

    let mut esp_wifi = EspWifi::new(modem, system_loop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, system_loop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("[network] starting wifi...");

    wifi.start()?;

    info!("[network] scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "[network] found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "[network] configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("[network] could not parse the given ssid into wifi config"),
        password: password
            .try_into()
            .expect("[network] could not parse the given password into wifi config"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    info!("[network] connecting wifi...");

    wifi.connect()?;

    info!("[network] waiting for dhcp lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("[network] wifi dhcp info: {:?}", ip_info);

    Ok(esp_wifi)
}
