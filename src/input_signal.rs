//speaking of de-bouncing how long is a typical single bounce for a maniacal switch?
//5ms-50ms so maybe timer start with a 10ms period and count upto 5
//
//My thoughts for de-bouncing and this maybe a little over engineered, but this is all a learning journey after all, is as follows.
//
//I will utilize two interrupts.
//The first one is gpio interrupt for both rising and falling edges.
//The second is a timer that would run at some time proportional to a bounce period.
//
//Then use state machine style logic to determine when the signal has stabilized.
//
//For example if the signal is stable and an edge is triggered, id first do logic to notify
//off the state change.  then change the state to waiting for stabilization and reset a
//timer counter.  The timer interrupt would come and look and see if the state is waiting
//for stabilization and increment the counter.  If another edge interrupt occurs the counter
//would be reset.  Eventually after edges stop occuring from the bounce.  The timer would be
//able to increment the value to a threshold value in which the signal could be considered
//stable.  Id probably have to add extra logic around that to detect that when the signal is
//detected as stable that it is still at the logic I expect, but that's the just of it.

use crate::{
    array_vec::ArrayVec,
    default_isr_this_has_to_be_wrong::{enable_interrupt, InterruptPriority},
    log,
};
use crate::{println, timer::*};
use jh7110_hal::gpio::Pad;
use jh7110_pac::{self as pac, Interrupt};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LogicState {
    Low,
    High,
    Unknown,
}

impl From<u64> for LogicState {
    fn from(value: u64) -> Self {
        match value {
            0 => LogicState::Low,
            _ => LogicState::High,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InputSignalState {
    Unknown,
    StableLow,
    StabilizingHigh,
    StableHigh,
    StabilizingLow,
}

#[derive(Copy, Clone, Debug)]
pub struct Signal {
    pin_number: Pad,
    state: InputSignalState,
    stabilization_counter: u8,
    logic_state: LogicState,
    edge_callback: fn(signal: &mut Signal, logic_state: LogicState),
}

impl Signal {
    pub fn new(
        pin_number: Pad,
        edge_callback: fn(signal: &mut Signal, logic_state: LogicState),
    ) -> Self {
        Self {
            pin_number,
            state: InputSignalState::Unknown,
            stabilization_counter: 0,
            logic_state: LogicState::Unknown,
            edge_callback,
        }
    }

    pub fn update_state(&mut self, state: InputSignalState) {
        self.state = state;
        self.stabilization_counter = 0;
    }
}

const NUMBER_GPIO: usize = 63;
const PADS_PER_REGISTER: usize = 32;

static mut SIGNALS: ArrayVec<Signal, NUMBER_GPIO> = ArrayVec::new();

pub fn configure() {
    //Setup input_signal structure list
    //Set length here.  There seems to be an error with initialization.  Refer to below for fix
    // https://docs.rust-embedded.org/embedonomicon/main.html#life-before-main
    unsafe {
        SIGNALS.init();
        match SIGNALS.try_push(Signal::new(Pad::Gpio37, edge_callback)) {
            Err(s) => {
                println!("Failed insert of signal for pin {:?}", s.pin_number);
            }
            Ok(_) => {}
        }
    }

    //run_test();
    println!("Signal {}", core::mem::size_of::<Signal>());

    //Get GPIO
    let pinctrl = unsafe { &*pac::SysPinctrl::ptr() };
    //Setup the output enable function for pin 37 (0)
    pinctrl
        .gpo_doen()
        .gpo_doen9()
        .modify(|_, w| w.doen37().variant(1)); //Set pin as an input (0 output, 1 input)

    //Setup the pad config for pin 37
    //Set to an input with pull-up enabled
    pinctrl.padcfg().gpio37().modify(|_, w| {
        w.ie()
            .set_bit() //enable input
            .ds()
            .variant(0b00) //output strength lowest 2mA
            .pu()
            .set_bit() //enable the pull-up
            .pd()
            .clear_bit() //disable the pull-down
            .slew()
            .clear_bit() //set slew rate to slow (dont care)
            .smt()
            .clear_bit() //disable the schmitt trigger for now (May want to enable later, will help
            //with switch bouncing)
            .pos()
            .clear_bit() //disable active pull down capability
    });

    //Setup GPIO interrupt
    //IS (Interrupt Sense) = 1
    //IBE (Interrupt Both Edges) = 1
    //IEV (Interrupt Event) = Don't care
    let pad = 37u32;
    let mask = 1 << (pad - PADS_PER_REGISTER as u32);
    //Enable GPIO IRQ function.  Note this also is needed just to enable reading of pins
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 55 (Enable IRQ Function)
    pinctrl.ioirq().ioirq0().write(|w| w.gpen0().set_bit());
    //Block 0 - pins 0-31
    //Block 1 - pins 32-63
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 56 Block0 Interrupt Sense (IS) EDGE or LEVEL Trigger
    //pinctrl
    //  .ioirq()
    //  .ioirq1()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 57 Block1 Interrupt Sense (IS) EDGE or LEVEL Trigger
    //Set edge triggering bit corsponding to pin (37 for now)
    pinctrl
        .ioirq()
        .ioirq2()
        .modify(|r, w| w.is1().variant(r.is1().bits() | mask));
    //Not sure what these are for.  Sets a bit to "Do not clear the register" or
    //"Clear the register". What register? This is where the interrupt it cleared
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 58 Block0 Interrupt Clear (IC)
    //pinctrl
    //  .ioirq()
    //  .ioirq3()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 59 Block1 Interrupt Clear (IC)
    //Clear it for now for 37.  Will have to clear it after handeling it
    pinctrl
        .ioirq()
        .ioirq4()
        .modify(|r, w| w.ic1().variant(r.ic1().bits() | mask));
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 60 Block0 Interrupt Both Edges (IBE)
    //pinctrl
    //  .ioirq()
    //  .ioirq5()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 61 Block1 Interrupt Both Edges (IBE)
    pinctrl
        .ioirq()
        .ioirq6()
        .modify(|r, w| w.ibe1().variant(r.ibe1().bits() | mask));
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 62 Block0 Interrupt Event (IEV)
    //pinctrl.ioirq().ioirq7()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 63 Block1 Interrupt Event (IEV)
    //Pisitive/Low trigger or Negative/High trigger
    //I dont think we care about this since we are triggering on both edges, but the
    //linux driver is clearing the bit, so lets do that.
    pinctrl
        .ioirq()
        .ioirq8()
        .modify(|r, w| w.iev1().variant(r.iev1().bits() | !mask));
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 64 Block0 Interrupt Enable (IE)
    //pinctrl.ioirq().ioirq9()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 65 Block1 Interrupt Enable (IE)
    pinctrl
        .ioirq()
        .ioirq10()
        .modify(|r, w| w.ie1().variant(r.ie1().bits() | mask));
    //Dont care about these two right now
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 66 Block0 Raw Interrupt Status (RIS)
    //pinctrl.ioirq().ioirq11()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 67 Block1 Raw Interrupt Status (RIS)
    //pinctrl.ioirq().ioirq12()
    //
    //Dont care about these for now.  They are used to determin what interrupts caused ISR
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 68 Block0 Masked Interrupt Status (MIS)
    //pinctrl.ioirq().ioirq13
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 69 BLOCK1 Masked Interrupt Status (MIS)
    //pinctrl.ioirq().ioirq14()
    //
    //Dont care about these for now.  This is where the value of the signal is read
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 70 Block0 Sync register
    //pinctrl.ioirq().ioirq15()
    //SYS IOMUX CFGSAIF SYSCFG IOIRQ 71 Block1 Sunc register
    //pinctrl.ioirq().ioirq16()

    println!("Setting up crg");
    //Setup timer and timer interrupt
    let sys_crg = unsafe { &*pac::Syscrg::ptr() };
    //Enable the timer Advanced Preriphial BUS clock
    sys_crg.clk_tim().apb().modify(|_, w| w.clk_icg().set_bit());
    //Enable the timer clock
    sys_crg
        .clk_tim()
        .tim01_0()
        .modify(|_, w| w.clk_icg().set_bit());
    //Clear the timer abp reset bit
    sys_crg
        .rst()
        .software_address_selector()
        .rst3()
        .modify(|_, w| w.u0_si5_timer_apb().clear_bit());
    //Clear the timer reset bit
    sys_crg
        .rst()
        .software_address_selector()
        .rst3()
        .modify(|_, w| w.u0_si5_timer_0().clear_bit());

    //Setup the timer
    let t0 = Timer0::new();
    t0.print_debug_info();
    //Mask the interrupt
    t0.set_int_mask(TimerIntMask::Mask);
    match t0.get_int_clear_busy() {
        TimerIntClearBusy::Yes => {
            println!("Timer0 int clear still busy");
        }
        TimerIntClearBusy::No => {
            t0.set_int_status_clear(TimerIntClearStatus::Clear);
            t0.set_load(0x100000);
            //t0.reload_counter();
            t0.set_int_mask(TimerIntMask::Unmask);
            t0.set_enable(TimerEnable::Enable);
        }
    }

    //
    enable_interrupt(Interrupt::SYS_IOMUX, InterruptPriority::Priority7);
    enable_interrupt(Interrupt::TIMER0, InterruptPriority::Priority7);
}

pac::interrupt!(SYS_IOMUX, signal_change_handler);
#[no_mangle]
fn signal_change_handler() {
    println!("GPIO Signal ISR");
    //Read Block0 MIS
    //Read Block1 MIS
    let pinctrl = unsafe { &*pac::SysPinctrl::ptr() };
    let mis0 = pinctrl.ioirq().ioirq13().read().bits();
    let mis1 = pinctrl.ioirq().ioirq14().read().bits();
    let sync0 = pinctrl.ioirq().ioirq15().read().bits();
    let sync1 = pinctrl.ioirq().ioirq16().read().bits();
    println!("mis0: {:#10x}, mis1: {:#10x}", mis0, mis1);
    println!("sync0: {:#10x}, sync1: {:#10x}", sync0, sync1);

    let mis: u64 = (mis1 as u64) << 32 | (mis0 as u64);
    let sync: u64 = (sync1 as u64) << 32 | (sync0 as u64);

    //Check if any of these match out signals, read sync, update signal, do call back
    unsafe {
        for s in SIGNALS.iter_mut() {
            let pin_mask = 1 << (s.pin_number as u64);
            if mis & pin_mask != 0 {
                (s.edge_callback)(s, LogicState::from(sync & pin_mask));
            }
        }
    }

    //Note from TRM:  You can also write 0 and 1 sequentially to clear edge IRQ.
    //Writing just 1 didnt clear and writing 0 just disabled
    //Write to Block0 IC
    pinctrl
        .ioirq()
        .ioirq3()
        .modify(|r, w| w.ic0().variant(r.ic0().bits() & !mis0));
    pinctrl
        .ioirq()
        .ioirq3()
        .modify(|r, w| w.ic0().variant(r.ic0().bits() | mis0));
    //Write to Block1 IC
    pinctrl
        .ioirq()
        .ioirq4()
        .modify(|r, w| w.ic1().variant(r.ic1().bits() & !mis1));
    pinctrl
        .ioirq()
        .ioirq4()
        .modify(|r, w| w.ic1().variant(r.ic1().bits() | mis1));
}

pac::interrupt!(TIMER0, timer_interrupt_handler);
#[no_mangle]
fn timer_interrupt_handler() {
    //Get the timer
    let t0 = Timer0::new();
    //Do the thing
    //Check if any of these match out signals, read sync, update signal, do call back
    unsafe {
        for s in SIGNALS.iter_mut() {
            match s.state {
                InputSignalState::StabilizingLow => {
                    s.stabilization_counter += 1;
                    if s.stabilization_counter == 5 {
                        //read pin to verify we are insync
                        println!("Signal: {:?} Stable Low", s);
                        s.update_state(InputSignalState::StableLow);
                    }
                }
                InputSignalState::StabilizingHigh => {
                    s.stabilization_counter += 1;
                    if s.stabilization_counter == 5 {
                        //Read Pin and verify we are in sync
                        s.update_state(InputSignalState::StableHigh);
                    }
                }
                InputSignalState::Unknown => {
                    //Read pin state and set to stabelizing that direction
                    println!("Unknown State choosing high");
                    s.update_state(InputSignalState::StabilizingHigh);
                }
                _ => { /*Not wure what to do yet*/ }
            }
        }
    }
    //Clear the interrupt status
    t0.set_int_status_clear(TimerIntClearStatus::Clear);
}

fn edge_callback(signal: &mut Signal, logic_state: LogicState) {
    println!(
        "Edge Callback Signal: {:?}, State: {:?}",
        signal, logic_state
    );

    match signal.state {
        InputSignalState::StableLow => {
            signal.update_state(InputSignalState::StabilizingHigh);
            if logic_state == LogicState::Low {
                println!("***UNEXPECTED LOGIC LOW***");
            }
        }
        InputSignalState::StableHigh => {
            signal.update_state(InputSignalState::StabilizingLow);
            if logic_state == LogicState::High {
                println!("***UNEXPECTED LOGIC HIGH***");
            }
        }
        InputSignalState::StabilizingLow => {
            signal.stabilization_counter = 0;
        }
        InputSignalState::StabilizingHigh => {
            signal.stabilization_counter = 0;
        }
        InputSignalState::Unknown => {
            match logic_state {
                LogicState::Low => {
                    signal.update_state(InputSignalState::StabilizingLow);
                }
                LogicState::High => {
                    signal.update_state(InputSignalState::StabilizingHigh);
                }
                LogicState::Unknown => {
                    println!("Unknown Logic State WTF!?");
                }
            };
        }
    }
}

fn run_test() {
    println!("Before insert loop");
    for i in 1..11 {
        println!("Signal Created");
        let s = Signal {
            pin_number: Pad::from(i),
            state: InputSignalState::Unknown,
            stabilization_counter: 0,
            logic_state: LogicState::Unknown,
            edge_callback,
        };

        println!("Before Push");

        unsafe {
            if let Err(_) = SIGNALS.try_push(s) {
                println!("Error Push");
            }
        };
    }
    println!("After insert loop");

    unsafe {
        for s in SIGNALS.iter() {
            println!("Hi: {:?}", s.pin_number);
        }
    }

    println!("After interator");

    println!("Before Mut Interator");
    unsafe {
        for s in SIGNALS.iter_mut() {
            s.pin_number = Pad::from(s.pin_number as u32 + 10);
        }
    }

    println!("After Mut Iterator");

    println!("After mut interator loop");

    unsafe {
        for s in SIGNALS.iter() {
            println!("Hi: {:?}", s.pin_number);
        }
    }

    println!("After mut interator");
}
