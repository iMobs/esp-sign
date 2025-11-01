#![no_std]
#![feature(impl_trait_in_assoc_type)]

// pub mod leds;
pub mod web;
pub mod wifi;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
pub use picoserve::make_static;
use rgb::RGB8;

const WEB_TASK_POOL_SIZE: usize = 4;
// I think I need one extra for DHCP requests and other background tasks
const WEB_SOCKET_SIZE: usize = WEB_TASK_POOL_SIZE + 2;

#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,

    #[default("")]
    wifi_password: &'static str,
}

type RgbChannel = Channel<CriticalSectionRawMutex, RGB8, 8>;
static RGB_CHANNEL: RgbChannel = RgbChannel::new();
