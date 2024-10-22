use crate::println;
use embedded_hal::digital::OutputPin;
use jh7110_hal::gpio;
use jh7110_pac as pac;
use riscv::interrupt::machine::Interrupt;
static mut PIN_IS_HIGH: bool = false;

pub fn configure() {
    //Steal the peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    // configure GPIO 40 as an output
    let gpio40 = gpio::get_gpio(peripherals.sys_pinctrl.padcfg().gpio40());

    let mut gpio40_out = gpio40.into_enabled_output();

    unsafe {
        let mtimecmp = 0x0200_4008 as *mut u64;
        let mtime = 0x0200_bff8 as *const u64;
        //quarter second .25s/(1/4MHz) = 1000000
        mtimecmp.write_volatile(mtime.read_volatile() + 1_000_000);
        riscv::register::mie::set_mtimer();
        riscv::register::mstatus::set_mie();
        gpio40_out.set_low().ok();
        PIN_IS_HIGH = false;
    }
}

const PERIOD: u32 = 20000;
static mut DUTY_CYCLE: u32 = 0;

#[riscv_rt::core_interrupt(Interrupt::MachineTimer)]
fn machine_timer_isr() {
    println!("Machine Timer ISR");
    let peripherals = unsafe { pac::Peripherals::steal() };
    // configure GPIO 40 as an output
    let gpio40 = gpio::get_gpio(peripherals.sys_pinctrl.padcfg().gpio40());
    let mut gpio40_out = gpio40.into_enabled_output();

    unsafe {
        match PIN_IS_HIGH {
            false => {
                let _ = gpio40_out.set_high();
                PIN_IS_HIGH = true;
            }
            true => {
                let _ = gpio40_out.set_low();
                PIN_IS_HIGH = false;
            }
        }
    }

    unsafe {
        let mtimecmp = 0x0200_4008 as *mut u64;
        let mtime = 0x0200_bff8 as *const u64;
        mtimecmp.write_volatile(mtime.read_volatile() + 1_000_000);
    }

    if peripherals.pwm.ctrl().read().en().bit_is_set() {
        unsafe {
            DUTY_CYCLE = (DUTY_CYCLE + 5000) % PERIOD;
            peripherals
                .pwm
                .hrc()
                .modify(|_, w| w.hrc().variant(DUTY_CYCLE));
            println!("Duty Cycle: {DUTY_CYCLE}");
        }
    } else {
        println!("PWM Disabled :(");
    }
}
