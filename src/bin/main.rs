#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::clock::CpuClock;
// use esp_hal::gpio::Pin;
// use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_sign::make_static;
use panic_rtt_target as _;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new();
    let init = make_static!(
        esp_radio::Controller::<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );

    let stack = esp_sign::wifi::init_wifi(init, peripherals.WIFI, rng, &spawner).await;

    esp_sign::web::init_web(stack, &spawner).await;
    // esp_sign::leds::init_leds(peripherals.RMT, peripherals.GPIO2.degrade(), &spawner).await;

    let mut status_led = esp_hal::gpio::Output::new(
        peripherals.GPIO8,
        esp_hal::gpio::Level::Low,
        esp_hal::gpio::OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
    );

    loop {
        status_led.set_low();
        Timer::after_millis(100).await;
        status_led.set_high();
        Timer::after_millis(900).await;
    }
}
