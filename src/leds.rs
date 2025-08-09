use embassy_executor::Spawner;
use esp_hal::{
    gpio::AnyPin,
    rmt::{ConstChannelAccess, Rmt, Tx},
    time::Rate,
};
use esp_hal_smartled::SmartLedsAdapter;
use rgb::RGB8;
use smart_leds::{brightness, gamma, SmartLedsWrite};

// TODO: Change this to the right number of LEDs
const LED_COUNT: usize = 10;
// Copied from smartLedBuffer! macro
// The size we're assigning here is calculated as following
//  (
//   Nr. of LEDs
//   * channels (r,g,b -> 3)
//   * pulses per channel 8)
//  ) + 1 additional pulse for the end delimiter
const LED_BUFFER_SIZE: usize = LED_COUNT * 24 + 1;

pub async fn init_leds(rmt: esp_hal::peripherals::RMT<'_>, pin: AnyPin<'_>, spawner: &Spawner) {
    let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();

    let buffer = [0; LED_BUFFER_SIZE];
    let neopixel = SmartLedsAdapter::new(rmt.channel0, pin, buffer);

    spawner.must_spawn(led_task(neopixel));
}

type Neopixel = SmartLedsAdapter<ConstChannelAccess<Tx, 0>, LED_BUFFER_SIZE>;

#[embassy_executor::task]
async fn led_task(mut neopixel: Neopixel) {
    let mut data = [RGB8::new(0xD1, 0xDD, 0xF1); LED_COUNT];
    neopixel
        .write(brightness(gamma(data.into_iter()), 0xFF))
        .unwrap();

    loop {
        let rgb = crate::RGB_CHANNEL.receive().await;
        data = [rgb; LED_COUNT];
        neopixel
            .write(brightness(gamma(data.into_iter()), 0xFF))
            .unwrap();
    }
}
