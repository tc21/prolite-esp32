#[derive(Debug)]
pub struct Config {
    pub wifi: WifiConfig
}

#[derive(Debug)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String
}
