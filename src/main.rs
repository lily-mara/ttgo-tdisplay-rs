use std::time::Instant;

use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::{
    gpio::{AnyIOPin, Gpio16, Gpio18, Gpio19, Gpio23, Gpio4, Gpio5, PinDriver},
    peripheral::PeripheralRef,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriver, SPI2},
    units::{FromValueType, Hertz},
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use mipidsi::{models::Model, Builder, Display};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    match run() {
        Ok(()) => println!("ok"),
        Err(e) => println!("err {e}"),
    }
}

const TFT_WIDTH: usize = 240;
const TFT_HEIGHT: usize = 320;

const BLUE: Rgb565 = Rgb565::new(91, 206, 250);
const PINK: Rgb565 = Rgb565::new(245, 169, 184);
const WHITE: Rgb565 = Rgb565::new(255, 255, 255);

fn init_display() -> Result<
    Display<impl WriteOnlyDataCommand, impl Model<ColorFormat = Rgb565>, impl OutputPin>,
    &'static str,
> {
    let mut delay = esp_idf_hal::delay::FreeRtos;

    let sclk = unsafe { Gpio18::new() };

    let spi = unsafe { SPI2::new() };
    let sdo = unsafe { Gpio19::new() };
    let cs = unsafe { Gpio5::new() };

    let driver = SpiDriver::new(
        spi,
        sclk,
        sdo,
        None::<PeripheralRef<AnyIOPin>>,
        Dma::Channel1(TFT_WIDTH * TFT_HEIGHT * 2 + 8),
    )
    .map_err(|_| "create spi driver")?;

    let mut bl = PinDriver::input_output_od(unsafe { Gpio4::new() })
        .map_err(|_| "gpio16: create pin driver")?;

    bl.set_high().map_err(|_| "backlight")?;

    let spi_config = Config::new().baudrate(24.MHz().into());

    let device_driver = SpiDeviceDriver::new(driver, Some(cs), &spi_config)
        .map_err(|_| "create spi device driver")?;

    let dc = PinDriver::input_output_od(unsafe { Gpio16::new() })
        .map_err(|_| "gpio16: create pin driver")?;
    let rst = PinDriver::input_output_od(unsafe { Gpio23::new() })
        .map_err(|_| "gpio23: create pin driver")?;

    let di = SPIInterfaceNoCS::new(device_driver, dc);
    let mut display = Builder::st7789(di)
        .init(&mut delay, Some(rst))
        .map_err(|_| "init display")?;

    let start = Instant::now();

    display.clear(Rgb565::BLACK).map_err(|_| "clear")?;
    println!("clear {:?}", start.elapsed());

    Ok(display)
}

fn run() -> Result<(), &'static str> {
    let mut display = init_display()?;

    let mut rect = Rectangle::new(Point::new(48, 0), Size::new(24, 320));

    for color in [
        Rgb565::CSS_AQUA,
        Rgb565::CSS_HOT_PINK,
        Rgb565::WHITE,
        Rgb565::CSS_HOT_PINK,
        Rgb565::CSS_AQUA,
    ] {
        let start = Instant::now();
        display.fill_solid(&rect, color).map_err(|_| "draw")?;

        println!("color {:?} {:?}", color, start.elapsed());

        rect.translate_mut(Point::new(24, 0));
    }

    Ok(())
}
