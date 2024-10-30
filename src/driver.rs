use std::{thread, time::Duration};

use esp_idf_svc::{
    hal::{
        self,
        gpio::{Level, PinDriver},
    },
    sys::EspError,
};

use crate::protocol;

pub fn display_screen(
    screen: &protocol::drive::Screen,
    control_pins: &mut ControlPins,
) -> Result<(), EspError> {
    control_pins.clk.set_low()?;
    thread::sleep(CLOCK_CYCLE_DELAY);

    for row in 0..6 {
        let (row_0_level, row_1_level, row_2_level) = ROW_CONTROL[row];

        control_pins.row_0.set_level(row_0_level)?;
        control_pins.row_1.set_level(row_1_level)?;
        control_pins.row_2.set_level(row_2_level)?;

        for col in 0..80 {
            let pixel = screen[row][80 - col - 1];
            control_pins.r.set_level(pixel.red.to_gpio_level())?;
            control_pins.g.set_level(pixel.green.to_gpio_level())?;

            control_pins.clk.set_high()?;
            thread::sleep(CLOCK_CYCLE_DELAY);
            control_pins.clk.set_low()?;
            thread::sleep(CLOCK_CYCLE_DELAY);
        }
    }

    Ok(())
}

const CLOCK_CYCLE_DELAY: Duration = Duration::from_nanos(64);

const ROW_CONTROL: [(Level, Level, Level); 7] = [
    (Level::High, Level::High, Level::High),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::High, Level::Low),
];

pub struct ControlPins {
    // I haven't been able to figure out how to configure these pins dynamically
    pub r: PinDriver<'static, hal::gpio::Gpio2, hal::gpio::Output>,
    pub g: PinDriver<'static, hal::gpio::Gpio4, hal::gpio::Output>,
    pub row_0: PinDriver<'static, hal::gpio::Gpio15, hal::gpio::Output>,
    pub row_1: PinDriver<'static, hal::gpio::Gpio16, hal::gpio::Output>,
    pub row_2: PinDriver<'static, hal::gpio::Gpio17, hal::gpio::Output>,
    pub clk: PinDriver<'static, hal::gpio::Gpio18, hal::gpio::Output>,
}
