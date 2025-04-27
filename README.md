# esp-sign

A WiFi-connected LED sign controller using Rust

# Game Plan

I want to 3D print a neon-like sign that uses Neopixel LEDs to create a glowing effect.
The sign will be controlled by a small ESP32 board that connects to WiFi and serves a simple web API to control the LEDs.
Since I already have Home Assistant turning on and off stuff in my office, it would be nice to control the sign with it as well.
I want to create neon flickering effects and other animations, maybe even come up with a scripting language to control each letter and section of the sign that can be uploaded via the web API.

# Hurdles

I still have a lot to learn when it comes to Fusion 360 and CAD software, but all this really is is taking a SVG and extruding it into a 3D model.
I also want this to be about a half meter wide and I only have an Ender 3 S1, so I will have to figure out how to print it in sections and assemble it.

# Features

- Controls WS2812B LEDs (Neopixels)
- Connects to WiFi and serve a simple web API to control the LEDs
  - The idea is to control this with Home Assistant to turn the sign on and off with my existing automations.

# Software

- Written in Rust using the [esp-rs](https://github.com/esp-rs) ecosystem and [embassy](https://github.com/embassy-rs/embassy) for async programming.
- Uses [esp-hal-smartled](https://github.com/esp-rs/esp-hal-community/tree/main/esp-hal-smartled) and [smart-leds](https://github.com/smart-leds-rs/smart-leds) to control the WS2812B LEDs.
- In order to not leak my WiFi in the code, I'm using [toml-cfg](https://github.com/jamesmunns/toml-cfg) so I can set the WiFi credentials at compile time and .gitignore the file.
- Uses [picoserve](https://github.com/sammhicks/picoserve) to create an axum like web server to control the LEDs.

# Hardware

Right now I'm testing this on the [esp-rust-board](https://github.com/esp-rs/esp-rust-board) which I got off of Amazon.
I plan on getting a roll of Neopixels to use with it, but for now I'm just using the single on-board WS2812B LED to test the code and light diffusion of the filament I'm using.
If I get further along I plan on moving to a smaller board that doesn't have all all the bells and whistles of the esp-rust-board (I don't need an IMU, for example).

# Reference Material

- [Adafruit NeoPixel Überguide](https://learn.adafruit.com/adafruit-neopixel-uberguide)
- It's amazing what people [get away with selling on etsy](https://www.etsy.com/uk/listing/867821190/cyberpunk-2077-illuminable-afterlife). This is basically what I want to create and put on my wall.
