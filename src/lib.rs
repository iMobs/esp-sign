#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod web;
pub mod wifi;

pub use picoserve::make_static;

pub(crate) const WEB_TASK_POOL_SIZE: usize = 4;
// I think I need one extra for DHCP requests and other background tasks
pub(crate) const WEB_SOCKET_SIZE: usize = WEB_TASK_POOL_SIZE + 1;

#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,

    #[default("")]
    wifi_password: &'static str,
}
