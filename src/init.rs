use embedded_hal::delay::DelayNs;
use jh71xx_hal::{clocks, ddr, delay, pac, pll, register::feature_disable};
use riscv::register::{marchid, mhartid, mimpid, mstatus, mvendorid};

use crate::println;

//use crate::println;

//#[no_mangle]
//#[allow(non_snake_case)]
//#[cfg(not(any(feature = "rt", feature = "rts")))]
//fn DefaultInterruptHandler() {}

//#[no_mangle]
//#[allow(non_snake_case)]
//#[cfg(not(any(feature = "rt", feature = "rts")))]
//fn DefaultExceptionHandler() {}

#[inline]
pub unsafe fn setup_mstatus() {
    // clear MSTATUS register
    mstatus::clear_uie();
    mstatus::clear_sie();
    mstatus::clear_mie();
    mstatus::clear_mprv();
    mstatus::clear_sum();
    mstatus::clear_mxr();
    mstatus::clear_tvm();
    mstatus::clear_tw();
    mstatus::clear_tsr();
    mstatus::set_ube(mstatus::Endianness::LittleEndian);
    mstatus::set_spp(mstatus::SPP::User);
    mstatus::set_mpp(mstatus::MPP::User);
    mstatus::set_fs(mstatus::FS::Off);
    mstatus::set_sbe(mstatus::Endianness::LittleEndian);
    mstatus::set_mbe(mstatus::Endianness::LittleEndian);
}

#[inline]
pub unsafe fn setup_features() {
    feature_disable::clear_all();
}

#[inline]
pub unsafe fn setup_clocks() {
    //Steal the peripherals
    let p = pac::Peripherals::steal();
    //Select pll frequency
    let mut pll = pll::Pll::new(p.sys_syscon);
    pll.set_pll0(pll::Freq::pll0_1ghz())
        .set_pll2(pll::Freq::pll2_1188mhz());
    /* DDR controller related clk init */
    let mut clock_syscrg = clocks::ClockSyscrg::new(p.syscrg);
    let mut clock_aoncrg = clocks::ClockAoncrg::new(p.aoncrg);
    // select CPU root clock
    clock_syscrg.select_cpu_root(clocks::ClkCpuRootMuxSel::ClkPll0);
    // select Bus root clock
    clock_syscrg.select_bus_root(clocks::ClkBusRootMuxSel::ClkPll2);
    // initialize peripheral clocks
    clock_syscrg.select_peripheral_root(clocks::ClkPeripheralMuxSel::ClkPll2);
    clock_syscrg.set_noc_stg_axi(clocks::ClkNocIcg::Enable);
    clock_aoncrg.select_aon_apb(clocks::ClkAonApbMuxSel::ClkOscDiv4);
    clock_syscrg.select_qspi(clocks::ClkQspiMuxSel::ClkQspiRefSrc);
}

#[inline]
pub unsafe fn setup_gpio() {
    let p = pac::Peripherals::steal();
    p.sys_syscon.sys_syscfg3().modify(|_, w| {
        w.vout0_remap_awaddr_gpio0().clear_bit();
        w.vout0_remap_awaddr_gpio1().clear_bit();
        w.vout0_remap_awaddr_gpio2().clear_bit();
        w.vout0_remap_awaddr_gpio3().clear_bit()
    });
    // TX/RX are GPIOs 5 and 6
    p.sys_pinctrl.gpo_doen().gpo_doen1().modify(|_, w| {
        w.doen5().variant(0);
        w.doen6().variant(0b1)
    });

    p.sys_pinctrl
        .gpo_dout()
        .gpo_dout1()
        .modify(|_, w| w.dout5().variant(20));

    p.sys_pinctrl
        .gpi()
        .gpi3()
        .modify(|_, w| w.uart_sin_0().variant(6));
}

#[inline]
pub unsafe fn setup_ddr() {
    //Steal the peripherals
    let p = pac::Peripherals::steal();
    //First reset the syscrg clock
    let mut clock_syscrg = clocks::ClockSyscrg::new(p.syscrg);
    clock_syscrg.reset_apb0();
    //Get a ddr preipheral for configuration
    let mut dram = ddr::Ddr::new(p.dmc_ctrl, p.dmc_phy, clock_syscrg.release(), p.sys_syscon);
    //Get a delay
    let mut udelay = delay::u74_mdelay();
    //TODO see why its being configured like this.  Not sure why its selecting
    //the bus clock between configuration and delays
    dram.select_bus_clock(clocks::ClkDdrBusMuxSel::ClkOscDiv2);
    dram.set_pll1(pll::Freq::pll1_ddr2133_1066mhz());
    udelay.delay_ns(500);
    dram.select_bus_clock(clocks::ClkDdrBusMuxSel::ClkPll1Div2);
    udelay.delay_ns(200);
    // init the clocks.
    dram.init_osc(&mut udelay);
    dram.init_apb(&mut udelay);
    dram.init_axi(&mut udelay);
    // init the OMC PHY.
    dram.phy_train();
    dram.phy_util();
    dram.phy_start();
    dram.select_bus_clock(clocks::ClkDdrBusMuxSel::ClkOscDiv2);
    // init the OMC (Orbit Memory Controller).
    dram.omc_init();
}

/// Prints the boot mode of the device.
pub fn print_boot_mode() {
    let p = unsafe { pac::AonPinctrl::steal() };
    // lowest two bits only; 0: SPI, 1: MMC2, 2: MMC1, 3: UART
    let mode_str = match p.ioirq_status_in_sync2().read().ioirq().bits() & 0b11 {
        0 => "SPI",
        1 => "MMC2",
        2 => "MMC1",
        3 => "UART",
        mode => {
            println!("boot mode: unknown ({mode:#010x})");
            "unknown"
        }
    };
    println!("boot mode: {mode_str}");
}

/// Gets the vendor name of the device from the `vendorid`.
#[inline]
pub fn vendorid_to_name<'a>(vendorid: usize) -> &'a str {
    match vendorid {
        0x0489 => "SiFive",
        _ => "unknown",
    }
}

/// Gets the implementation ID from the device.
///
/// <https://sifive.cdn.prismic.io/sifive/2dd11994-693c-4360-8aea-5453d8642c42_u74mc_core_complex_manual_21G3.pdf>
#[inline]
pub fn impid_to_name<'a>(vendorid: usize) -> &'a str {
    match vendorid {
        0x0421_0427 => "21G1.02.00 / llama.02.00-general",
        _ => "unknown",
    }
}

/// Print RISC-V core information:
///
/// - vendor
/// - arch
/// - implementation
/// - hart ID
#[inline]
pub fn print_ids() {
    let vid = mvendorid::read().map(|r| r.bits()).unwrap_or(0);
    let aid = marchid::read().map(|r| r.bits()).unwrap_or(0);
    let iid = mimpid::read().map(|r| r.bits()).unwrap_or(0);

    // TODO: This prints 8000000000000007, but should be 80000007.
    // See U74-MC core complex manual 21G3.
    let archid = (aid >> 32) as u32 | aid as u32;
    println!("RISC-V arch {archid:#010x}");
    let vendor_name = vendorid_to_name(vid);
    println!("RISC-V core vendor: {vendor_name} ({vid:#06x})");
    let imp_name = impid_to_name(iid);
    println!("RISC-V implementation: {imp_name} ({iid:#010x})");
    let hart_id = mhartid::read();
    println!("RISC-V hart ID {hart_id}");
}
