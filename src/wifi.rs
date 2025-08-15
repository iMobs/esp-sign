use defmt::{debug, error, info, unwrap};
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Timer;
use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};

pub async fn init_wifi(
    esp_wifi_ctrl: &'static EspWifiController<'static>,
    wifi: esp_hal::peripherals::WIFI<'static>,
    mut rng: esp_hal::rng::Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    let (controller, interfaces) = unwrap!(esp_wifi::wifi::new(esp_wifi_ctrl, wifi));
    let wifi_interface = interfaces.sta;

    let mut dhcp_config = embassy_net::DhcpConfig::default();
    let hostname = unwrap!("esp32.local".try_into());
    dhcp_config.hostname = Some(hostname);
    let net_config = embassy_net::Config::dhcpv4(dhcp_config);
    let net_seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        crate::make_static!(
            StackResources<{ crate::WEB_SOCKET_SIZE }>,
            StackResources::new()
        ),
        net_seed,
    );

    spawner.must_spawn(connection(controller));
    spawner.must_spawn(net_task(runner));

    wait_for_connection(stack).await;

    stack
}

/// Connection task to manage wifi connection.
/// It will keep trying to connect to the configured wifi network.
/// It will wait for disconnection events and retry.
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    const SSID: &str = crate::CONFIG.wifi_ssid;
    const PASSWORD: &str = crate::CONFIG.wifi_password;

    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.capabilities());

    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after_secs(5).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            unwrap!(controller.set_configuration(&client_config));
            debug!("Starting wifi");
            unwrap!(controller.start_async().await);
            debug!("Wifi started!");
        }
        debug!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => debug!("Wifi connected!"),
            Err(e) => {
                error!("Failed to connect to wifi: {:?}", e);
                Timer::after_secs(5).await
            }
        }
    }
}

/// Network task to run the embassy net stack.
/// It will wait for network events and handle them, just needs to run in the background.
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

async fn wait_for_connection(stack: Stack<'_>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after_millis(500).await;
    }

    debug!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after_millis(500).await;
    }
}
