use hal::gpio::{GpenFunction, GpoFunction};
use jh7110_hal as hal;
use jh7110_pac::{self as pac};

use crate::println;

pub fn configure() {
    let p = unsafe { pac::Peripherals::steal() };

    //Enable the pwm apb clock
    p.syscrg.clk_pwm_apb().modify(|_, w| w.clk_icg().set_bit());
    //Reset the device
    p.syscrg
        .rst()
        .software_address_selector()
        .rst3()
        .modify(|_, w| w.u0_pwm_apb().none());
    //Set the low and high counter values
    p.pwm.lrc().modify(|_, w| w.lrc().variant(20000));
    p.pwm.hrc().modify(|_, w| w.hrc().variant(5000));
    //Setup the control register
    p.pwm.ctrl().modify(|_, w| {
        w.en()
            .set_bit() //enable rptc counter incrementation
            .eclk()
            .clear_bit() //clear use external clock
            .nec()
            .clear_bit() //clear bit, has no effect when using internal clock
            .oe()
            .set_bit() //enable the output deiver for driving the pin
            .single()
            .clear_bit() //disable one shot.  when set counter will stop after LRC is hit
            .inte()
            .clear_bit() //disable interrupts for now
            .int()
            .clear_bit() //clear the interrupt pending bit
            .cntrrst()
            .clear_bit() //clear bit to take counter out of reset
            .capte()
            .clear_bit() //clear the capture function
    });
    //Setup the output function for pin 56 (GPOUT_SYS_PWM_CHANNEL0 24)
    p.sys_pinctrl
        .gpo_dout()
        .gpo_dout14()
        .modify(|_, w| w.dout56().variant(GpoFunction::U0_PWM_8CH_PTC_PWM_0));
    //Setup the output enable function for pin 56 (GPOEN_SYS_PWM0_CHANNEL0 9)
    p.sys_pinctrl
        .gpo_doen()
        .gpo_doen14()
        .modify(|_, w| w.doen56().variant(GpenFunction::U0_PWM_8CH_PTC_OE_N_0));
    //Setup the pad config for pin 56
    p.sys_pinctrl.padcfg().gpio56().modify(|_, w| {
        w.ie()
            .clear_bit() //disable interrupts
            .ds()
            .variant(0b11) //configure deive strength to use function?
            .pu()
            .clear_bit() //disable the pull-up
            .pd()
            .clear_bit() //disable the pull-down
            .slew()
            .clear_bit() //set sluw rate to slow (i dont know what effec this has)
            .smt()
            .clear_bit() //disable the schmitt trigger
            .pos()
            .clear_bit() //disable active pull down capability
    });
}

pac::interrupt!(UART0, uart0);
#[no_mangle]
fn uart0() {
    // UART0 interrupt handler is running in an interrupt-free context,
    // and should thus have exclusive access to peripheral memory.
    println!("HI");
}
