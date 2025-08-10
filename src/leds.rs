use embassy_executor::Spawner;
use esp_hal::{
    gpio::AnyPin,
    rmt::{ConstChannelAccess, Rmt, Tx},
    time::Rate,
};
use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
use rgb::RGB8;
use smart_leds::{brightness, gamma, SmartLedsWriteAsync};

// TODO: Change this to the right number of LEDs
const LED_COUNT: usize = 10;
const LED_BUFFER_SIZE: usize = buffer_size_async(LED_COUNT);

pub async fn init_leds(rmt: esp_hal::peripherals::RMT<'_>, pin: AnyPin<'_>, spawner: &Spawner) {
    let rmt = defmt::unwrap!(Rmt::new(rmt, Rate::from_mhz(80))).into_async();

    let buffer = [0; LED_BUFFER_SIZE];
    let neopixel = SmartLedsAdapterAsync::new(rmt.channel0, pin, buffer);

    spawner.must_spawn(led_task(neopixel));
}

type Neopixel = SmartLedsAdapterAsync<ConstChannelAccess<Tx, 0>, LED_BUFFER_SIZE>;

#[embassy_executor::task]
async fn led_task(mut neopixel: Neopixel) {
    let mut data = [RGB8::new(0xD1, 0xDD, 0xF1); LED_COUNT];

    loop {
        defmt::unwrap!(
            neopixel
                .write(brightness(gamma(data.into_iter()), 0xFF))
                .await
        );

        let rgb = crate::RGB_CHANNEL.receive().await;
        data = [rgb; LED_COUNT];
    }
}
