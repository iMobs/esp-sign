#![no_std]

#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,

    #[default("")]
    wifi_password: &'static str,
}

#[macro_export]
macro_rules! make_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
