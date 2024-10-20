use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use jh71xx_hal::{delay, gpio, pac};

pub fn configure() {
    //Steal the peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    // configure GPIO 40 as an output
    let gpio40 = gpio::get_gpio(peripherals.sys_pinctrl.padcfg().gpio40());
    let mut gpio40_out = gpio40.into_enabled_output();
    // pull GPIO 40 low
    //
    let mut udelay = delay::u74_mdelay();
    loop {
        gpio40_out.set_low().ok();
        udelay.delay_ms(250);

        gpio40_out.set_high().ok();
        udelay.delay_ms(250);
    }
}
