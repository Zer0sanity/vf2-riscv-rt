use core::fmt;
use embedded_hal_nb::serial::{ErrorType, Read, Write};
use jh7110_hal::{pac, uart};
use nb::block;

/// Convenience alias for the [`Uart`](jh71xx_hal::uart::Uart) implementation for the [`Uart0`](jh71xx_hal::pac::Uart0) peripheral.
pub type Uart0 = uart::Uart<pac::Uart0>;

/// Convenience wrapper to implement traits on the [`Uart`](jh71xx_hal::uart::Uart) type.
pub struct Logger(pub Uart0);

impl ErrorType for Logger {
    type Error = uart::Error;
}

impl Read for Logger {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.0.read()
    }
}

impl Write for Logger {
    fn write(&mut self, val: u8) -> nb::Result<(), Self::Error> {
        self.0.write(val)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.flush()
    }
}

static LOGGER: spin::Mutex<Option<Logger>> = spin::Mutex::new(None);

/// Set the globally available logger that enables the macros.
pub fn init() {
    //Steal the peri
    let p = unsafe { pac::Peripherals::steal() };
    let logger = Logger(Uart0::new_with_config(
        p.uart0,
        uart::TIMEOUT_US,
        uart::Config {
            data_len: uart::DataLength::Eight,
            stop: uart::Stop::One,
            parity: uart::Parity::None,
            baud_rate: uart::BaudRate::B115200,
            clk_hz: uart::CLK_OSC,
        },
    ));

    //critical_section::with(|_| {
    //    LOGGER.replace(logger);
    //});
    LOGGER.lock().replace(logger);
}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for &byte in s.as_bytes() {
            block!(self.write(byte)).ok();
        }

        block!(self.flush()).ok();

        Ok(())
    }
}

/// Inner implementation of the local `println` macro.
///
/// From [`oreboot`](https://github.com/oreboot/oreboot/blob/37a5e71b3095922aedbe4c40fe2a7a68595a3198/src/lib/log/src/lib.rs)
pub fn print(args: fmt::Arguments) {
    use fmt::Write;

    //critical_section::with(|cs| {
    //    if let Some(&mut l) = LOGGER.borrow(cs) {
    //        l.write_fmt(args)
    //    }
    //});

    if let Some(l) = LOGGER.lock().as_mut() {
        l.write_fmt(args).ok();
    }
}

/// Serial implementation of the `print` macro from `core`.
///
/// From [`oreboot`](https://github.com/oreboot/oreboot/blob/37a5e71b3095922aedbe4c40fe2a7a68595a3198/src/lib/log/src/lib.rs)
#[macro_export]
macro_rules! print {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::log::print(core::format_args!($fmt $(, $($arg)+)?));
    }
}

/// Serial implementation of the `println` macro from `core`.
///
/// From [`oreboot`](https://github.com/oreboot/oreboot/blob/37a5e71b3095922aedbe4c40fe2a7a68595a3198/src/lib/log/src/lib.rs)
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::log::print(core::format_args!($fmt $(, $($arg)+)?));
        $crate::print!("\r\n");
    }
}
