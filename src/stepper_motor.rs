/*
SIGNAL  GPIO    PIN     COLOR
-----------------------------
EN      44      40      RED
MS1     61      38      BLUE
MS2     36      36      BLACK
UART1   NA      NA      NA
UART1   63      35      GREEN
CLK     GND     9       NA
STEP    59      33      WHITE
DIR     39      31      YELLOW
*/

use embedded_hal::digital::OutputPin;
use jh7110_hal::{
    gpio::{self, GpenFunction, GpioCfg, GpoFunction},
    pac,
};
use jh7110_pac::Interrupt;

use crate::{
    default_isr_this_has_to_be_wrong::{enable_interrupt, InterruptPriority},
    println,
    timer::*,
};

static mut PIN_IS_HIGH: bool = false;
pub fn init() {
    let p = unsafe { pac::Peripherals::steal() };
    init_gpio(
        p.sys_pinctrl.padcfg().gpio44(), /*en*/
        p.sys_pinctrl.padcfg().gpio61(), /*ms1*/
        p.sys_pinctrl.padcfg().gpio36(), /*ms2*/
        p.sys_pinctrl.padcfg().gpio63(), /*uart*/
        //p.sys_pinctrl.padcfg().gpioXX(), /*clk*/
        p.sys_pinctrl.padcfg().gpio59(), /*step*/
        p.sys_pinctrl.padcfg().gpio39(), /*dir*/
    );
}

fn init_gpio<
    GpioEn: GpioCfg,
    GPIOMs1: GpioCfg,
    GpioMs2: GpioCfg,
    GpioUart: GpioCfg,
    GpioStep: GpioCfg,
    GpioDir: GpioCfg,
>(
    en: &GpioEn,
    ms1: &GPIOMs1,
    ms2: &GpioMs2,
    uart: &GpioUart,
    //clk: &GPIO,
    step: &GpioStep,
    dir: &GpioDir,
) {
    let mut ms1 = gpio::get_gpio(ms1).into_enabled_output();
    let _ = ms1.set_low();

    let mut ms2 = gpio::get_gpio(ms2).into_enabled_output();
    let _ = ms2.set_low();

    let mut en = gpio::get_gpio(en).into_enabled_output();
    let _ = en.set_low();

    let mut uart = gpio::get_gpio(uart).into_enabled_output();
    let _ = uart.set_low();
    //let clk = gpio::get_gpio(clk);
    //let mut step = gpio::get_gpio(step).into_enabled_output();
    //let _ = step.set_low();

    let mut dir = gpio::get_gpio(dir).into_enabled_output();
    let _ = dir.set_low();

    // println!("Setting up crg step");
    // //Setup timer and timer interrupt
    // let sys_crg = unsafe { &*pac::Syscrg::ptr() };
    // //Enable the timer Advanced Preriphial BUS clock
    // sys_crg.clk_tim().apb().modify(|_, w| w.clk_icg().set_bit());
    // //Enable the timer clock
    // sys_crg
    //     .clk_tim()
    //     .tim01_1()
    //     .modify(|_, w| w.clk_icg().set_bit());
    // //Clear the timer abp reset bit
    // sys_crg
    //     .rst()
    //     .software_address_selector()
    //     .rst3()
    //     .modify(|_, w| w.u0_si5_timer_apb().clear_bit());
    // //Clear the timer reset bit
    // sys_crg
    //     .rst()
    //     .software_address_selector()
    //     .rst3()
    //     .modify(|_, w| w.u0_si5_timer_1().clear_bit());

    // //Setup the timer
    // let t1 = Timer1::new();
    // //Mask the interrupt
    // t1.set_int_mask(TimerIntMask::Mask);
    // match t1.get_int_clear_busy() {
    //     TimerIntClearBusy::Yes => {
    //         println!("Timer1 int clear still busy");
    //     }
    //     TimerIntClearBusy::No => {
    //         t1.set_int_status_clear(TimerIntClearStatus::Clear);
    //         //So the apb is at 24MHz and I want this to fire every 10ms.
    //         //24000000*.01=240000
    //         t1.set_load(24000);
    //         //t0.reload_counter();
    //         t1.set_int_mask(TimerIntMask::Unmask);
    //         t1.set_enable(TimerEnable::Enable);
    //     }
    // }
    //enable_interrupt(Interrupt::TIMER1, InterruptPriority::Priority7);

    let p = unsafe { pac::Peripherals::steal() };
    //Enable the pwm apb clock
    p.syscrg.clk_pwm_apb().modify(|_, w| w.clk_icg().set_bit());
    //Reset the device
    p.syscrg
        .rst()
        .software_address_selector()
        .rst3()
        .modify(|_, w| w.u0_pwm_apb().clear_bit());
    //Set the low and high counter values
    p.pwm1.lrc().modify(|_, w| w.lrc().variant(5_000_000u32));
    p.pwm1.hrc().modify(|_, w| w.hrc().variant(5_000_000u32));
    //Setup the control register
    p.pwm1.ctrl().modify(|_, w| {
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
            .set_bit() //enable interrupts
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
        .modify(|_, w| w.dout59().variant(GpoFunction::U0_PWM_8CH_PTC_PWM_1));
    //Setup the output enable function for pin 56 (GPOEN_SYS_PWM0_CHANNEL0 9)
    p.sys_pinctrl
        .gpo_doen()
        .gpo_doen14()
        .modify(|_, w| w.doen59().variant(GpenFunction::U0_PWM_8CH_PTC_OE_N_1));
    //Setup the pad config for pin 56
    p.sys_pinctrl.padcfg().gpio59().modify(|_, w| {
        w.ie()
            .clear_bit() //input disabled
            .ds()
            .variant(0b11) //output strength 12mA
            .pu()
            .clear_bit() //disable the pull-up
            .pd()
            .clear_bit() //disable the pull-down
            .slew()
            .set_bit() //set slwe rate to fast (Im assuming I want fast transitions)
            .smt()
            .clear_bit() //disable the schmitt trigger (Dont care for output)
            .pos()
            .clear_bit() //disable active pull down capability
    });

    enable_interrupt(Interrupt::PTC1, InterruptPriority::Priority7)

    //
}

pac::interrupt!(PTC1, step_pwm_interrupt_handler);
#[no_mangle]
fn step_pwm_interrupt_handler() {
    let p = unsafe { pac::Peripherals::steal() };
    //Set the low and high counter values
    let mut current_hrc = p.pwm1.hrc().read().bits();
    if current_hrc == 0 {
        current_hrc = 5_000_000u32;
    } else {
        current_hrc -= 1;
    }
    p.pwm1.hrc().modify(|_, w| w.hrc().variant(current_hrc));
}
//pac::interrupt!(TIMER1, step_timer_interrupt_handler);
//#[no_mangle]
//fn step_timer_interrupt_handler() {
//    let pinctrl = unsafe { &*pac::SysPinctrl::ptr() };
//    let gpio59 = gpio::get_gpio(pinctrl.padcfg().gpio59());
//    let mut gpio59_out = gpio59.into_enabled_output();
//
//    unsafe {
//        match PIN_IS_HIGH {
//            false => {
//                let _ = gpio59_out.set_high();
//                PIN_IS_HIGH = true;
//            }
//            true => {
//                let _ = gpio59_out.set_low();
//                PIN_IS_HIGH = false;
//            }
//        }
//    }
//
//    //Clear the interrupt status
//    let t1 = Timer1::new();
//    t1.set_int_status_clear(TimerIntClearStatus::Clear);
//}
