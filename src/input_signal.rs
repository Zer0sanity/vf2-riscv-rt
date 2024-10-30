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

use jh7110_hal::gpio::Pad;
use jh7110_pac::{self as pac, gmac0::phyif_ctrl_status::FalsecardetW};

use crate::println;

enum InputSignalState {
    Unknown,
    StableLow,
    StabilizingHigh,
    StableHigh,
    StabilizingLow,
}

struct Signal {
    pub pin_number: u32,
    pub state: InputSignalState,
    pub rising_edge_callback: fn(),
    pub falling_edge_callback: fn(),
}

struct Signals {
    pub num_signals: usize,
    pub signals: [Signal; 1],
}

impl Signals {
    fn new() -> Self {
        Signals {
            num_signals: 1,
            signals: [Signal {
                pin_number: 37,
                state: InputSignalState::Unknown,
                rising_edge_callback: || {
                    println!("Rising");
                },
                falling_edge_callback: || {
                    println!("Falling");
                },
            }],
        }
    }

    fn add(signal: Signal) {}
}

static mut SIGNALS: Option<Signals> = None;

pub fn configure() {
    //GPIO37

    //Setup input_signal structure list
    //Get GPIO
    //Set to an input with pull-up enabled
    //Setup timer interrupt
    //Setup GPIO interrupt
    //IS (Interrupt Sense) = 1
    //IBE (Interrupt Both Edges) = 1
    //IEV (Interrupt Event) = Don't care
    //

    //let mut gpio40_out = gpio40.into_enabled_output();

    //I dont think this is needed.  dout should be ignored.
    //Setup the output function for pin 37 (0)
    //p.sys_pinctrl
    //    .gpo_dout()
    //    .gpo_dout9()
    //    .modify(|_, w| w.dout37().variant(0)); //Set output signal to 0

    let pinctrl = unsafe { &*pac::SysPinctrl::ptr() };
    //Setup the output enable function for pin 37 (0)
    pinctrl
        .gpo_doen()
        .gpo_doen9()
        .modify(|_, w| w.doen37().variant(1)); //Set pin as an input (0 output, 1 input)
                                               //Setup the pad config for pin 37
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

    let s = Signals::new();
    unsafe {
        SIGNALS.replace(s);
    }

    let pad = 37u32;
    let pad_per_reg = 32u32;
    //Enable GPIO IRQ function.  Note this also is needed just to enable reading of pins
    pinctrl.ioirq().ioirq0().write(|w| w.gpen0().set_bit());
    //Set edge triggering
    pinctrl
        .ioirq()
        .ioirq2()
        .write(|w| w.is1().variant(1 << (pad_per_reg - pad)));

    let mut lastvalue = false;
    loop {
        unsafe {
            if let Some(ref signals) = SIGNALS {
                for s in 0..signals.num_signals {
                    let signal = &signals.signals[s];
                    let pad = signal.pin_number;
                    let mut value = false;
                    if pad < pad_per_reg {
                        value = (pinctrl.ioirq().ioirq15().read().bits() >> pad) & 0x1 != 0;
                    } else if pad < u32::from(Pad::Gpio63) {
                        let idx = pad.saturating_sub(pad_per_reg);
                        value = (pinctrl.ioirq().ioirq16().read().bits() >> idx) & 0x1 != 0;
                    }

                    if value != lastvalue {
                        match value {
                            true => {
                                (signal.rising_edge_callback)();
                            }
                            false => {
                                (signal.falling_edge_callback)();
                            }
                        }
                        lastvalue = value;
                    }
                }
            }
        }

        //        let idx = pad.saturating_sub(pad_per_reg);
        //        //let value = pinctrl.ioirq().ioirq16().read().bits();
        //        let value = (pinctrl.ioirq().ioirq16().read().bits() >> idx) & 0x1;
        //        if value != lastvalue {
        //            println!("Pin State: {}", value);
        //            lastvalue = value;
        //        }

        //if pad < pad_per_reg {
        //    let value = (pinctrl.ioirq().ioirq15().read().bits() >> pad) & 0x1 != 0;
        //    println!("Pin Value: {}", value);
        //} else if pad < u32::from(Pad::Gpio63) {
        //    let idx = pad.saturating_sub(pad_per_reg);
        //    let value = (pinctrl.ioirq().ioirq16().read().bits() >> idx) & 0x1 != 0;
        //    println!("Pin Value: {}", value);
        //} else {
        //    println!("Error");
        //}
    }
}