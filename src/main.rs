#![no_std]
#![no_main]

extern crate panic_halt;

mod blinky;
mod init;
mod log;

use riscv_rt::{entry, pre_init};

#[export_name = "_mp_hook"]
pub extern "Rust" fn mp_hook(hartid: usize) -> bool {
    match hartid {
        1 => true,
        _ => false,
    }

    // if hartid == 1 {
    //     true
    // } else {
    //     false
    // }
    // hartid == 1
}

#[no_mangle]
fn MachineEnvCall(trap_frame: &riscv_rt::TrapFrame) -> ! {
    loop {}
}

#[no_mangle]
fn ExceptionHandler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    loop {}
}

#[no_mangle]
fn DefaultHandler() {
    // ...
}

#[pre_init]
unsafe fn before_main() {
    init::setup_mstatus();
    init::setup_features();
    init::setup_clocks();
    init::setup_gpio();
    init::setup_ddr();
    //let dp = pac::Peripherals::take().unwrap();

    //let (sys_syscon, mut clock_syscrg, _clock_aoncrg) =
    //    init::configure_clocks(dp.sys_syscon, dp.syscrg, dp.aoncrg);
    //let (sys_syscon, _sys_pinctrl) = init::configure_gpios(sys_syscon, dp.sys_pinctrl);

    // AXI cfg0, clk_apb_bus, clk_apb0, clk_apb12
    //clock_syscrg.reset_apb0();

    //let _ddr = init::configure_dram(dp.dmc_ctrl, dp.dmc_phy, clock_syscrg.release(), sys_syscon);
}

#[entry]
fn main() -> ! {
    log::init();
    println!("main");
    blinky::configure();

    loop {}
}
