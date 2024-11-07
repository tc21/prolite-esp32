use esp_idf_svc::{hal::delay::Delay, sys::EspError};
use prolite::ScreenBuffer;

use crate::{
    config::*,
    gpio::{ControlPins, ToGpioLevel},
};

pub fn display_screen(
    buffer: &ScreenBuffer,
    delay_driver: &Delay,
    control_pins: &mut ControlPins,
) -> Result<(), EspError> {
    let wait_clock_delay = || delay_driver.delay_us(CLOCK_DELAY_US);

    control_pins.clk.set_low()?;
    wait_clock_delay();

    for row in 0..ScreenBuffer::HEIGHT {
        let (row_0_level, row_1_level, row_2_level) = CONTROL_SIGNALS_BY_ROW[row];

        control_pins.screen.set_high()?;
        // control_pins.row_0.set_low();
        // control_pins.row_1.set_low();
        // control_pins.row_2.set_low();

        for col in 0..ScreenBuffer::WIDTH {
            let pixel = buffer.0[row][80 - col - 1];
            control_pins.r.set_level(pixel.red.to_gpio_level())?;
            control_pins.g.set_level(pixel.green.to_gpio_level())?;

            control_pins.clk.set_high()?;
            wait_clock_delay();
            control_pins.clk.set_low()?;
            wait_clock_delay();
        }


        control_pins.row_0.set_level(row_0_level)?;
        control_pins.row_1.set_level(row_1_level)?;
        control_pins.row_2.set_level(row_2_level)?;

        wait_clock_delay();

        control_pins.screen.set_low()?;
        delay_driver.delay_us(ROW_DELAY_US);
        control_pins.screen.set_high()?;

        wait_clock_delay();
    }

    Ok(())
}
