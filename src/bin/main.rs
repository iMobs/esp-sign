#![no_std]
#![no_main]

use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_sign::make_static;
use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};
use panic_rtt_target as _;

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let init = make_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK).unwrap()
    );

    let (controller, interfaces) = esp_wifi::wifi::new(init, peripherals.WIFI).unwrap();
    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        make_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.must_spawn(connection(controller));
    spawner.must_spawn(net_task(runner));

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

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    // TODO: Increase this to the number of LEDs you have
    let buffer = esp_hal_smartled::smartLedBuffer!(1);
    let mut neopixel = esp_hal_smartled::SmartLedsAdapter::new(
        rmt.channel0,
        // TODO: Change this to the GPIO pin you have connected your NeoPixel to
        peripherals.GPIO2,
        buffer,
    );

    {
        use smart_leds::{brightness, gamma, SmartLedsWrite, RGB8};

        let data = [RGB8 {
            // #d1dd6d
            r: 0xD1,
            g: 0xDD,
            b: 0x6D,
        }];
        neopixel
            .write(brightness(gamma(data.into_iter()), 0xFF))
            .unwrap();
    }

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}

/// Connection task to manage wifi connection.
/// It will keep trying to connect to the configured wifi network.
/// It will wait for disconnection events and retry.
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
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
                ssid: esp_sign::CONFIG.wifi_ssid.try_into().unwrap(),
                password: esp_sign::CONFIG.wifi_password.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            debug!("Starting wifi");
            controller.start_async().await.unwrap();
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
