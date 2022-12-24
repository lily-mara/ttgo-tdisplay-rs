use std::{thread::sleep, time::Duration};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565, prelude::WebColors};
use esp_idf_hal::{
    delay::Ets,
    gpio::{AnyIOPin, Gpio16, Gpio18, Gpio19, Gpio23, Gpio5, PinDriver},
    peripheral::PeripheralRef,
    spi::{config::Config, SpiDeviceDriver, SpiDriver, SPI2},
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use mipidsi::Builder;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    match run() {
        Ok(()) => println!("ok"),
        Err(e) => println!("err {e}"),
    }
}

fn run() -> Result<(), &'static str> {
    let mut delay = Ets;

    let sclk = unsafe { Gpio18::new() };

    let spi = unsafe { SPI2::new() };
    let sdo = unsafe { Gpio19::new() };
    let cs = unsafe { Gpio5::new() };

    let driver = SpiDriver::new(
        spi,
        sclk,
        sdo,
        None::<PeripheralRef<AnyIOPin>>,
        esp_idf_hal::spi::Dma::Auto(0),
    )
    .map_err(|_| "create spi driver")?;

    let device_driver = SpiDeviceDriver::new(driver, Some(cs), &Config::new())
        .map_err(|_| "create spi device driver")?;

    let dc = PinDriver::input_output_od(unsafe { Gpio16::new() })
        .map_err(|_| "gpio16: create pin driver")?;
    let rst = PinDriver::input_output_od(unsafe { Gpio23::new() })
        .map_err(|_| "gpio23: create pin driver")?;

    let di = SPIInterfaceNoCS::new(device_driver, dc);
    let mut display = Builder::st7789(di)
        .init(&mut delay, Some(rst))
        .map_err(|_| "init display")?;

    display
        .clear(Rgb565::CSS_DARK_RED)
        .map_err(|_| "clear display")?;

    sleep(Duration::from_secs(100));

    Ok(())
}
