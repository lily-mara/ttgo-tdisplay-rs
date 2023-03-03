use std::time::{Duration, Instant};

use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{draw_target::DrawTarget, image::Image, pixelcolor::Rgb565, prelude::*};
use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::{
    gpio::{AnyIOPin, Gpio0, Gpio16, Gpio18, Gpio19, Gpio23, Gpio4, Gpio5, PinDriver},
    peripheral::PeripheralRef,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriver, SPI2},
    units::FromValueType,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use mipidsi::{
    models::{Model, ST7789},
    Builder, Display, ModelOptions,
};
use tinybmp::Bmp;

// Defines the IMAGE_DATA variable, set up by build.rs
include!(concat!(env!("OUT_DIR"), "/images.rs"));

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    match run() {
        Ok(()) => println!("ok"),
        Err(e) => println!("err {e}"),
    }
}

const TFT_WIDTH: usize = 135;
const TFT_HEIGHT: usize = 240;

/// Implementation of `mipidsi::Model` for the Lilygo ttgo t-display microcontroller's display. This is fundamentally a
struct TDisplayModel;

impl Model for TDisplayModel {
    type ColorFormat = <ST7789 as Model>::ColorFormat;

    fn init<RST, DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        madctl: u8,
        rst: &mut Option<RST>,
    ) -> Result<u8, mipidsi::error::InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs<u32>,
        DI: WriteOnlyDataCommand,
    {
        let mut s = ST7789;
        s.init(di, delay, madctl, rst)
    }

    fn write_pixels<DI, I>(&mut self, di: &mut DI, colors: I) -> Result<(), mipidsi::Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        let mut s = ST7789;
        s.write_pixels(di, colors)
    }

    fn default_options() -> ModelOptions {
        ModelOptions::with_all((135, 240), (135, 240), |_| (52, 40))
    }
}

fn init_display() -> Result<
    Display<impl WriteOnlyDataCommand, impl Model<ColorFormat = Rgb565>, impl OutputPin>,
    &'static str,
> {
    let mut delay = esp_idf_hal::delay::Ets;

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
    let mut display = Builder::with_model(di, TDisplayModel)
        .init(&mut delay, Some(rst))
        .map_err(|_| "init display")?;

    let start = Instant::now();

    display.clear(Rgb565::BLACK).map_err(|_| "clear")?;
    println!("clear {:?}", start.elapsed());

    Ok(display)
}

fn run() -> Result<(), &'static str> {
    let mut display = init_display()?;

    let bmps = IMAGE_DATA
        .into_iter()
        .map(|bytes| Bmp::from_slice(bytes).unwrap())
        .collect::<Vec<_>>();

    let changer =
        PinDriver::input(unsafe { Gpio0::new() }).map_err(|_| "gpio0: create pin driver")?;

    let bmps_len = bmps.len();

    let mut bmp_idx = 0;

    let point = Point::new(0, 0);

    display.clear(Rgb565::WHITE).map_err(|_| "clear")?;
    Image::new(&bmps[0], point)
        .draw(&mut display)
        .map_err(|_| "draw image")?;

    let mut debounce_time = Instant::now();
    let mut held = false;

    loop {
        if changer.is_low() {
            if !held && debounce_time.elapsed() > Duration::from_millis(250) {
                debounce_time = Instant::now();
                held = true;

                bmp_idx = (bmp_idx + 1) % bmps_len;

                Image::new(&bmps[bmp_idx], point)
                    .draw(&mut display)
                    .map_err(|_| "draw image")?;
            }
        } else {
            held = false;
        }
    }

    // let mut rect = Rectangle::new(Point::new(48, 0), Size::new(24, 320));

    // for color in [
    //     Rgb565::CSS_AQUA,
    //     Rgb565::CSS_HOT_PINK,
    //     Rgb565::WHITE,
    //     Rgb565::CSS_HOT_PINK,
    //     Rgb565::CSS_AQUA,
    // ] {
    //     let start = Instant::now();
    //     display.fill_solid(&rect, color).map_err(|_| "draw")?;

    //     println!("color {:?} {:?}", color, start.elapsed());

    //     rect.translate_mut(Point::new(24, 0));
    // }

    // Ok(())
}
