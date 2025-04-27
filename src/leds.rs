use embassy_executor::Spawner;
use esp_hal::{gpio::AnyPin, rmt::Rmt, time::Rate};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{brightness, gamma, SmartLedsWrite, RGB8};

const LED_COUNT: usize = 1; // TODO: Change this to the number of LEDs you have

pub async fn init_leds(rmt: esp_hal::peripherals::RMT, pin: AnyPin, spawner: &Spawner) {
    let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();

    // Annoying that the const LED_COUNT is not available in the macro
    let buffer = smartLedBuffer!(1);
    let mut neopixel = SmartLedsAdapter::new(rmt.channel0, pin, buffer);

    // TODO: Change this into a task that can be called from the web interface
    let _ = spawner;
    {
        let data = [RGB8::new(
            // #d1dd6d
            0xD1, 0xDD, 0x6D,
        ); LED_COUNT];
        neopixel
            .write(brightness(gamma(data.into_iter()), 0xFF))
            .unwrap();
    }
}
