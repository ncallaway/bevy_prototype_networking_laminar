use laminar::Config;
use std::time::Duration;

pub enum Transport {
    Laminar(LaminarConfig),
}

pub struct LaminarConfig {
    pub idle_connection_timeout: Duration,
    pub heartbeat_interval: Option<Duration>,
    pub max_packets_in_flight: u16,
}

impl Default for LaminarConfig {
    fn default() -> Self {
        LaminarConfig {
            idle_connection_timeout: Duration::from_millis(5000),
            heartbeat_interval: Some(Duration::from_millis(1000)),
            max_packets_in_flight: 1024,
        }
    }
}

impl From<LaminarConfig> for Config {
    fn from(cfg: LaminarConfig) -> Self {
        Config {
            idle_connection_timeout: cfg.idle_connection_timeout,
            heartbeat_interval: cfg.heartbeat_interval,
            max_packets_in_flight: cfg.max_packets_in_flight,
            ..Default::default()
        }
    }
}
