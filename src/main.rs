#![no_std]
#![no_main]

extern crate panic_halt;

mod init;
mod log;
mod blinky;
mod blinky_pwm;
mod default_isr_this_has_to_be_wrong;

use riscv_rt::{entry, pre_init};

#[export_name = "_mp_hook"]
pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
    //If hart is 1 return true, otherwise spin.
    //TODO: if later on we want to bring another hart online we will have to
    //modify loop to break out.
    match hartid {
        1 => true,
        _ => {
            loop {
                riscv::asm::wfi();
                if riscv::register::mip::read().msoft() {
                    break;
                }
            }
            false
        }
    }
}

#[export_name = "ExceptionHandler"]
fn custom_exception_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    println!("exception {:?}", trap_frame);
    loop {}
}

#[export_name = "DefaultHandler"]
fn custom_default_handler() {
    println!("custom_default_handler()");
    loop {}
}

#[pre_init]
unsafe fn before_main() {
    //    //Setup global things
    init::setup_clocks();
    init::setup_gpio();
    //init::setup_ddr();
}

#[repr(usize)]
#[derive(Debug)]
pub enum Harts {
    Hart0,
    Hart1,
    Hart2,
    Hart3,
    Hart4,
    Unknown,
}

impl From<usize> for Harts {
    fn from(value: usize) -> Self {
        match value {
            0 => Harts::Hart0,
            1 => Harts::Hart1,
           2 => Harts::Hart2,
            3 => Harts::Hart3,
            4 => Harts::Hart4,
            _ => Harts::Unknown,
        }
    }
}

#[entry]
fn main() -> ! {
    log::init();

    let hart_id = Harts::from(riscv::register::mhartid::read());
    match hart_id {
        Harts::Hart0 => init::print_ids(),
        Harts::Hart1 => {
            init::print_boot_mode();
            init::print_ids();
            unsafe {
                init::setup_ddr();
            }
        }
        Harts::Hart2 => init::print_ids(),
        Harts::Hart3 => init::print_ids(),
        Harts::Hart4 => init::print_ids(),
        _ => {
            println!("fml");

            //println!("Unknown hart {:?} booted.", hart_id);
        }
    }

    //Setup core local things
    unsafe {
        init::setup_mstatus();
        init::setup_features();
    };

    match hart_id {
        Harts::Hart1 => {
            blinky::configure();
            blinky_pwm::configure();
            println!("back in main about to spin after setting up blinky");
        }
        _ => {}
    }

    loop {}
}
