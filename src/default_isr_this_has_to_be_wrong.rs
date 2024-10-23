use crate::println;
use jh7110_pac as pac;
use riscv::interrupt::machine::Interrupt;

#[riscv_rt::core_interrupt(Interrupt::MachineExternal)]
fn machine_external_isr() {
    let hart = riscv::register::mhartid::read();
    //TODO Maybe move this external and thread safe with a mutex to ensure thread safty
    //when multipal cores are running.  Not sure if I need to accunt for interrupt
    //priorities or not
    let plic = unsafe { pac::Plic::steal() };

    let interrupt_number = plic.threshold_claim(hart).claim_complete().read().bits();

    if interrupt_number != 0 {
        let v: &pac::Vector = &pac::__EXTERNAL_INTERRUPTS[interrupt_number as usize];
        unsafe {
            if v._reserved != 0 {
                (v._handler)();
            } else {
                println!("reserved interrupt hit. wait what?");
            }
        }
        plic.threshold_claim(hart)
            .claim_complete()
            .write(|w| w.complete().variant(interrupt_number))
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
//pac::interrupt!(UART0, uart0);  // Assuming you want a specific handler for UART0
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
pac::interrupt!(PTC0, default_handler);
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
