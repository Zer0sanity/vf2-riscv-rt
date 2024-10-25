use crate::println;
use jh7110_pac::{self as pac};
use riscv::{interrupt::machine::Interrupt, register::mhartid};
//use riscv_rt::interrupts;

#[riscv_rt::core_interrupt(Interrupt::MachineExternal)]
fn machine_external_isr() {
    let hart = riscv::register::mhartid::read();
    //TODO Maybe move this external and thread safe with a mutex to ensure thread safty
    //when multipal cores are running.  Not sure if I need to accunt for interrupt
    //priorities or not
    let plic = unsafe { pac::Plic::steal() };
    //Claim the interrupt
    let interrupt_number = plic.threshold_claim(hart).claim_complete().read().bits();
    println!("Global interrupt number: {}", interrupt_number);
    if interrupt_number != 0 {
        //Load irs entry from the table
        let v: &pac::Vector = &pac::__EXTERNAL_INTERRUPTS[interrupt_number as usize];
        unsafe {
            //Check if its set and execute handler
            if v._reserved != 0 {
                (v._handler)();
            } else {
                println!("reserved interrupt hit. wait what?");
            }
        }
        //Complete the interrupt
        plic.threshold_claim(hart)
            .claim_complete()
            .write(|w| w.complete().variant(interrupt_number));
    }
}

#[repr(u8)]
pub enum InterruptPriority {
    Disabled = 0,
    Priority1 = 1,
    Priority2 = 2,
    Priority3 = 3,
    Priority4 = 4,
    Priority5 = 5,
    Priority6 = 6,
    Priority7 = 7,
}

impl From<u32> for InterruptPriority {
    fn from(value: u32) -> Self {
        match value {
            0 => InterruptPriority::Disabled,
            1 => InterruptPriority::Priority1,
            2 => InterruptPriority::Priority2,
            3 => InterruptPriority::Priority3,
            4 => InterruptPriority::Priority4,
            5 => InterruptPriority::Priority5,
            6 => InterruptPriority::Priority6,
            7 => InterruptPriority::Priority7,
            _ => InterruptPriority::Disabled,
        }
    }
}

pub fn enable_interrupt(interrupt_number: pac::Interrupt, priority: InterruptPriority) {
    let plic = unsafe { pac::Plic::steal() };
    let interrupt_number = interrupt_number as usize;
    let priority = priority as u32;
    //Set the interrupt prority
    println!(
        "Setting priority for interrupt {} to {}",
        interrupt_number, priority
    );
    plic.priority(interrupt_number)
        .write(|w| w.priority().variant(priority));

    //NOTE:  Pending bit can be cleared by enabeling the interrupt and then claiming it

    //Enable the interrupt
    let hart = mhartid::read();
    let register_offset = interrupt_number / 32;
    let enable_mask = 1 << (interrupt_number % 32);
    println!(
        "Enabeling interrupt {}, register {}, mask {:#10x}",
        interrupt_number, register_offset, enable_mask
    );
    plic.enable(hart)
        .enable_bits(register_offset)
        .modify(|r, w| w.enable().variant(r.enable().bits() | enable_mask));

    //Set the prority threshold (for now just or it with the prority value so at least
    //this one will fire)
    println!(
        "Setting interrupt threshold for hart {} to {}",
        hart, priority
    );
    plic.threshold_claim(hart)
        .threshold()
        .modify(|r, w| w.threshold().variant(r.threshold().bits() | priority));
}

pub fn clear_interrupt_enable_all() {
    let plic = unsafe { pac::Plic::steal() };
    while let Some(hart_enable_regs) = plic.enable_iter().next() {
        while let Some(hart_enable_reg) = hart_enable_regs.enable_bits_iter().next() {
            hart_enable_reg.reset();
            println!(
                "Cleared Interrupt Enable: {}",
                hart_enable_reg.read().bits()
            );
        }
    }
}

pub fn clear_interrupt_priotiry_all() {
    let plic = unsafe { pac::Plic::steal() };
    while let Some(priority_reg) = plic.priority_iter().next() {
        priority_reg.reset();
        println!("Reset Interrupt Priority {}", priority_reg.read().bits());
    }
}
pub fn print_interrupt_enable() {
    let plic = unsafe { pac::Plic::steal() };
    let mut hart = 0;
    while let Some(hart_enable_regs) = plic.enable_iter().next() {
        let mut reg_number = 0;
        while let Some(hart_enable_reg) = hart_enable_regs.enable_bits_iter().next() {
            println!(
                "Hart: {}, RegNum: {}, Value: {}",
                hart,
                reg_number,
                hart_enable_reg.read().bits()
            );
            reg_number += 1;
        }
        hart += 1;
    }
}

pub fn print_priority_interrupt_info() {
    let plic = unsafe { pac::Plic::steal() };
    let mut reg_num = 0;
    println!("Intrrupt Priority");
    while let Some(priority_reg) = plic.priority_iter().next() {
        println!("RegNum: {}, Value: {}", reg_num, priority_reg.read().bits());
        reg_num += 1;
    }
}

pub fn print_pending_interrupt_info() {
    let plic = unsafe { pac::Plic::steal() };
    let mut reg_num = 0;
    while let Some(pending_reg) = plic.pending_iter().next() {
        println!("RegNum: {}, Value: {}", reg_num, pending_reg.read().bits());
        reg_num += 1;
    }
}

#[no_mangle]
fn default_handler() {
    // This function will handle any unassigned interrupts
    println!("Default interrupt handler");
}

pac::interrupt!(L2PM, default_handler);
pac::interrupt!(L2PM1, default_handler);
pac::interrupt!(L2PM2, default_handler);
pac::interrupt!(L2PM3, default_handler);
pac::interrupt!(ETH_LPI0, default_handler);
pac::interrupt!(ETH_WAKE_IRQ0, default_handler);
pac::interrupt!(MACIRQ0, default_handler);
pac::interrupt!(QSPI0, default_handler);
pac::interrupt!(CRYPTO, default_handler);
pac::interrupt!(SDMA, default_handler);
pac::interrupt!(TRNG, default_handler);
pac::interrupt!(UART0, default_handler); // Assuming you want a specific handler for UART0
pac::interrupt!(UART1, default_handler);
pac::interrupt!(UART2, default_handler);
pac::interrupt!(I2C0, default_handler);
pac::interrupt!(I2C1, default_handler);
pac::interrupt!(I2C2, default_handler);
pac::interrupt!(SPI0, default_handler);
pac::interrupt!(SPI1, default_handler);
pac::interrupt!(SPI2, default_handler);
pac::interrupt!(UART3, default_handler);
pac::interrupt!(UART4, default_handler);
pac::interrupt!(UART5, default_handler);
pac::interrupt!(I2C3, default_handler);
pac::interrupt!(I2C4, default_handler);
pac::interrupt!(I2C5, default_handler);
pac::interrupt!(I2C6, default_handler);
pac::interrupt!(SPI3, default_handler);
pac::interrupt!(SPI4, default_handler);
pac::interrupt!(SPI5, default_handler);
pac::interrupt!(SPI6, default_handler);
//pac::interrupt!(PTC0, default_handler);
pac::interrupt!(PTC1, default_handler);
pac::interrupt!(PTC2, default_handler);
pac::interrupt!(PTC3, default_handler);
pac::interrupt!(PTC4, default_handler);
pac::interrupt!(PTC5, default_handler);
pac::interrupt!(PTC6, default_handler);
pac::interrupt!(PTC7, default_handler);
pac::interrupt!(WDOG, default_handler);
pac::interrupt!(TIMER0, default_handler);
pac::interrupt!(TIMER1, default_handler);
pac::interrupt!(TIMER2, default_handler);
pac::interrupt!(TIMER3, default_handler);
pac::interrupt!(DMA, default_handler);
pac::interrupt!(MMC0, default_handler);
pac::interrupt!(MMC1, default_handler);
pac::interrupt!(ETH_LPI1, default_handler);
pac::interrupt!(ETH_WAKE_IRQ1, default_handler);
pac::interrupt!(MACIRQ1, default_handler);
pac::interrupt!(AON_IOMUX, default_handler);
pac::interrupt!(SYS_IOMUX, default_handler);
pac::interrupt!(HOST0, default_handler);
pac::interrupt!(PERIPHERAL0, default_handler);
pac::interrupt!(OTG0, default_handler);
pac::interrupt!(PMU, default_handler);
