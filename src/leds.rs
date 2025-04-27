use embassy_executor::Spawner;
use esp_hal::{
    gpio::AnyPin,
    rmt::{Channel, Rmt},
    time::Rate,
    Blocking,
};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{brightness, gamma, SmartLedsWrite};

const LED_COUNT: usize = 1; // TODO: Change this to the number of LEDs you have

pub async fn init_leds(rmt: esp_hal::peripherals::RMT, pin: AnyPin, spawner: &Spawner) {
    let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();

    // Annoying that the const LED_COUNT is not available in the macro
    let buffer = smartLedBuffer!(1);
    let neopixel = SmartLedsAdapter::new(rmt.channel0, pin, buffer);

    spawner.must_spawn(led_task(neopixel));
}

#[embassy_executor::task]
async fn led_task(mut neopixel: SmartLedsAdapter<Channel<Blocking, 0>, 25>) {
    loop {
        let rgb = crate::RGB_CHANNEL.receive().await;
        let data = [rgb; LED_COUNT];
        neopixel
            .write(brightness(gamma(data.into_iter()), 0xFF))
            .unwrap();
    }
}
