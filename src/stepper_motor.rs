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
};

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
    _step: &GpioStep,
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
    p.pwm1.lrc().modify(|_, w| w.lrc().variant(2_000_000u32));
    p.pwm1.hrc().modify(|_, w| w.hrc().variant(1_000_000u32));
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

    unsafe {
        STEP_COUNTER = 0;
        MOVE_COMMAND.num_steps = 100;
        MOVE_COMMAND.direction = MotorDirection::Forward;
        let d = p.sys_pinctrl.padcfg().gpio39();
        let d = gpio::get_gpio(d);
        let mut d = d.into_enabled_output();
        d.set_pin(bool::from(MOVE_COMMAND.direction));
    }

    enable_interrupt(Interrupt::PTC1, InterruptPriority::Priority7)

    //
}

#[derive(Clone, Copy, Debug)]
enum MotorDirection {
    Forward,
    Retrograde,
}

impl From<MotorDirection> for bool {
    fn from(dir: MotorDirection) -> bool {
        match dir {
            MotorDirection::Forward => true,
            MotorDirection::Retrograde => false,
        }
    }
}

struct StepMove {
    num_steps: usize,
    direction: MotorDirection,
}

static mut MOVE_COMMAND: StepMove = StepMove {
    num_steps: 100,
    direction: MotorDirection::Forward,
};

static mut STEP_COUNTER: usize = 0;

pac::interrupt!(PTC1, step_pwm_interrupt_handler);
#[no_mangle]
fn step_pwm_interrupt_handler() {
    let p = unsafe { pac::Pwm1::steal() };
    //Print clock values to see interrupt
    let cnt = p.cntr().read().bits();
    let hrc = p.hrc().read().bits();
    let lrc = p.lrc().read().bits();
    let _ctrl = p.ctrl().read().bits();
    //println!(
    //    "cnt: {}, hrc: {}, lrc: {}, ctrl: {:#10X}",
    //    cnt, hrc, lrc, ctrl
    //);

    //See if the interrupt is for the second half of the step signal
    //Try to find a better way to detect this.  We seem to get an interrupt
    //after the match when in free running mode or a match to lrc in single mode.
    //So, for example in free running mode after the lrc match the counter would
    //roll over
    //Free-Run mode
    //HRC match -> cnt: 12000039, hrc: 12000000, lrc: 24000000, ctrl:       0x69
    //LRC match -> cnt: 22, hrc: 12000000, lrc: 24000000, ctrl:       0x69
    //Single mode
    //HRC match -> cnt: 12000058, hrc: 12000000, lrc: 24000000, ctrl:       0x79
    //LRC match -> cnt: 24000000, hrc: 12000000, lrc: 24000000, ctrl:       0x79
    if cnt < hrc || cnt == lrc {
        unsafe {
            STEP_COUNTER += 1;
            match MOVE_COMMAND.num_steps {
                0 => {
                    MOVE_COMMAND.num_steps = 100;
                    MOVE_COMMAND.direction = match MOVE_COMMAND.direction {
                        MotorDirection::Forward => MotorDirection::Retrograde,
                        MotorDirection::Retrograde => MotorDirection::Forward,
                    };
                    //Clear the counter and put it in reset.  Clearing counter is
                    //probasbly not needed.
                    p.cntr().modify(|_, w| w.cntr().variant(0));
                    p.ctrl().modify(|_, w| w.cntrrst().set_bit());
                    println!("End of move. {}", STEP_COUNTER);
                    //Setup to revirse direction
                    let d = pac::SysPinctrl::steal();
                    let d = d.padcfg().gpio39();
                    let d = gpio::get_gpio(d);
                    let mut d = d.into_enabled_output();
                    d.set_pin(bool::from(MOVE_COMMAND.direction));
                    println!("Direction: {:?}", MOVE_COMMAND.direction);
                    p.ctrl()
                        .modify(|_, w| w.single().clear_bit().cntrrst().clear_bit())
                }
                1 => {
                    //This is the last period of the move command so enable one shot
                    p.ctrl().modify(|_, w| w.single().set_bit());
                    MOVE_COMMAND.num_steps -= 1;
                    println!("Last Step {}", STEP_COUNTER);
                }
                _ => {
                    //Decrement the number of steps
                    MOVE_COMMAND.num_steps -= 1;
                    println!("Stepping {}", STEP_COUNTER);
                }
            }
        }
    }
    //Clear the interrupt
    p.ctrl().modify(|_, w| w.int().clear_bit());
}
