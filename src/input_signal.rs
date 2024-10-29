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

enum InputSignalState {
    Unknown,
    StableLow,
    StabilizingHigh,
    StableHigh,
    StabilizingLow,
}

pub fn configure() {
    //Steal the peripherals
    //let peripherals = unsafe { pac::Peripherals::steal() };
    // configure GPIO 40 as an output
    //let gpio40 = gpio::get_gpio(peripherals.sys_pinctrl.padcfg().gpio40());
    //
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
}
