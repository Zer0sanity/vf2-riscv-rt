use core::ptr;

use crate::println;
use jh7110_pac::{self as pac};

#[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::MachineExternal)]
fn machine_external_isr() {
    //TODO Maybe move this external and thread safe with a mutex to ensure thread safty
    //when multipal cores are running.  Not sure if I need to accunt for interrupt
    //priorities or not
    //Claim the interrupt
    let hart_id = HartId::from(riscv::register::mhartid::read());
    let interrupt_number = plic_claim(&hart_id, ExecutionMode::Machine);
    //println!("Global interrupt number: {}", interrupt_number);
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
        plic_complete(&hart_id, ExecutionMode::Machine, interrupt_number);
    }
}

#[repr(u8)]
#[derive(Debug)]
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

#[repr(u8)]
#[derive(Debug)]
enum ExecutionMode {
    Machine,
    Supervisor,
}

#[repr(u8)]
#[derive(Debug, Clone)]
enum HartId {
    Hart0,
    Hart1,
    Hart2,
    Hart3,
    Hart4,
}

impl From<usize> for HartId {
    fn from(value: usize) -> Self {
        match value {
            0 => HartId::Hart0,
            1 => HartId::Hart1,
            2 => HartId::Hart2,
            3 => HartId::Hart3,
            4 => HartId::Hart4,
            _ => HartId::Hart0,
        }
    }
}

pub fn enable_interrupt(interrupt_number: pac::Interrupt, priority: InterruptPriority) {
    plic_set_interrupt_priority(interrupt_number, priority);
    //NOTE:  Pending bit can be cleared by enabeling the interrupt and then claiming it

    //Enable the interrupt
    plic_enable_interrupt(HartId::Hart1, ExecutionMode::Machine, interrupt_number);

    //Set the prority threshold (for now just or it with the prority value so at least
    //this one will fire)
    plic_set_interrupt_priority_threshold(HartId::Hart1, ExecutionMode::Machine, 1)
}

//PLIC base address
const PLIC_BASE: u32 = 0x0C00_0000;

//Priority (offset into this by interrupt_number * 4)
const PLIC_PRIORITY: u32 = PLIC_BASE;

//Interrupt Pending
//Register offset address = (interrupt_number/32) * 4
//Bit offset = interrupt_number % 32
const PLIC_PENDING: u32 = PLIC_BASE + 0x1000;

//Enterrupt enables by hart and mode
//Register offset address = (interrupt_number/32) * 4
//Bit offset = interrupt_number % 32
const PLIC_HART0_MMODE_ENABLES: u32 = PLIC_BASE + 0x2000;
//Hart 1
const PLIC_HART1_MMODE_ENABLES: u32 = PLIC_BASE + 0x2080;
const PLIC_HART1_SMODE_ENABLES: u32 = PLIC_BASE + 0x2100;
//Hart 2
const PLIC_HART2_MMODE_ENABLES: u32 = PLIC_BASE + 0x2180;
const PLIC_HART2_SMODE_ENABLES: u32 = PLIC_BASE + 0x2200;
//Hart 3
const PLIC_HART3_MMODE_ENABLES: u32 = PLIC_BASE + 0x2280;
const PLIC_HART3_SMODE_ENABLES: u32 = PLIC_BASE + 0x2300;
//Hart 4
const PLIC_HART4_MMODE_ENABLES: u32 = PLIC_BASE + 0x2380;
const PLIC_HART4_SMODE_ENABLES: u32 = PLIC_BASE + 0x2400;

//Priority/Claim registers by hart and mode
//Hart0
const PILC_HART0_MMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_0000;
const PILC_HART0_MMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_0004;
//Hart1
const PILC_HART1_MMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_1000;
const PILC_HART1_MMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_1004;
const PILC_HART1_SMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_2000;
const PILC_HART1_SMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_2004;
//Hart2
const PILC_HART2_MMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_3000;
const PILC_HART2_MMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_3004;
const PILC_HART2_SMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_4000;
const PILC_HART2_SMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_4004;
//Hart3
const PILC_HART3_MMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_5000;
const PILC_HART3_MMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_5004;
const PILC_HART3_SMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_6000;
const PILC_HART3_SMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_6004;
//Hart4
const PILC_HART4_MMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_7000;
const PILC_HART4_MMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_7004;
const PILC_HART4_SMODE_PRIORITY_THRESHOLD: u32 = PLIC_BASE + 0x20_8000;
const PILC_HART4_SMODE_CLAIM_COMPLETE: u32 = PLIC_BASE + 0x20_8004;

pub fn clear_interrupt_enable_all() {
    for i in 1..5 {
        unsafe {
            ptr::write_volatile((PLIC_HART0_MMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART1_MMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART1_SMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART2_MMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART2_SMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART3_MMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART3_SMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART4_MMODE_ENABLES + 4 * i) as *mut u32, 0);
            ptr::write_volatile((PLIC_HART4_SMODE_ENABLES + 4 * i) as *mut u32, 0);
        }
    }
}

pub fn clear_interrupt_priotiry_all() {
    for i in 1..137 {
        unsafe {
            ptr::write_volatile((PLIC_PRIORITY + i * 4) as *mut u32, 0);
        }
    }
}
pub fn print_interrupt_enable() {
    println!("Interrupt Enable");
    unsafe {
        for i in 1..5 {
            let mut val = ptr::read_volatile((PLIC_HART0_MMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 0, i, val);
            val = ptr::read_volatile((PLIC_HART1_MMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 1, i, val);
            val = ptr::read_volatile((PLIC_HART1_SMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 1, i, val);
            val = ptr::read_volatile((PLIC_HART2_MMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 2, i, val);
            val = ptr::read_volatile((PLIC_HART2_SMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 2, i, val);
            val = ptr::read_volatile((PLIC_HART3_MMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 3, i, val);
            val = ptr::read_volatile((PLIC_HART3_SMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 3, i, val);
            val = ptr::read_volatile((PLIC_HART4_MMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 4, i, val);
            val = ptr::read_volatile((PLIC_HART4_SMODE_ENABLES + 4 * i) as *const u32);
            println!("Hart: {}, RegNum: {}, Value: {:#10x}", 4, i, val);
        }
    }
}

pub fn print_priority_interrupt_info() {
    for i in 1..137 {
        unsafe {
            let val = ptr::read_volatile((PLIC_PRIORITY + i * 4) as *const u32);
            println!("Interrupt: {}, Priority: {}", i, val);
        }
    }
}

fn plic_set_interrupt_priority(interrupt_number: pac::Interrupt, priority: InterruptPriority) {
    let priority_register = PLIC_PRIORITY + 4 * interrupt_number as u32;
    unsafe {
        ptr::write_volatile(priority_register as *mut u32, priority as u32);
    }
}

fn plic_enable_interrupt(
    hart: HartId,
    execution_mode: ExecutionMode,
    interrupt_number: pac::Interrupt,
) {
    //Register offset address = (interrupt_number/32) * 4
    //Bit offset = interrupt_number % 32
    //
    let register_offset = (interrupt_number as u32 / 32) * 4;
    let bit_offset = interrupt_number as u32 % 32;
    let base = match hart {
        HartId::Hart0 => PLIC_HART0_MMODE_ENABLES,
        HartId::Hart1 => match execution_mode {
            ExecutionMode::Machine => PLIC_HART1_MMODE_ENABLES,
            ExecutionMode::Supervisor => PLIC_HART1_SMODE_ENABLES,
        },
        HartId::Hart2 => match execution_mode {
            ExecutionMode::Machine => PLIC_HART2_MMODE_ENABLES,
            ExecutionMode::Supervisor => PLIC_HART2_SMODE_ENABLES,
        },
        HartId::Hart3 => match execution_mode {
            ExecutionMode::Machine => PLIC_HART3_MMODE_ENABLES,
            ExecutionMode::Supervisor => PLIC_HART3_SMODE_ENABLES,
        },
        HartId::Hart4 => match execution_mode {
            ExecutionMode::Machine => PLIC_HART4_MMODE_ENABLES,
            ExecutionMode::Supervisor => PLIC_HART4_SMODE_ENABLES,
        },
    };
    set_bit(base + register_offset, bit_offset);
}

fn plic_set_interrupt_priority_threshold(
    hart: HartId,
    execution_mode: ExecutionMode,
    threshold: u32,
) {
    let reg = match hart {
        HartId::Hart0 => PILC_HART0_MMODE_PRIORITY_THRESHOLD,
        HartId::Hart1 => match execution_mode {
            ExecutionMode::Machine => PILC_HART1_MMODE_PRIORITY_THRESHOLD,
            ExecutionMode::Supervisor => PILC_HART1_SMODE_PRIORITY_THRESHOLD,
        },
        HartId::Hart2 => match execution_mode {
            ExecutionMode::Machine => PILC_HART2_MMODE_PRIORITY_THRESHOLD,
            ExecutionMode::Supervisor => PILC_HART2_SMODE_PRIORITY_THRESHOLD,
        },
        HartId::Hart3 => match execution_mode {
            ExecutionMode::Machine => PILC_HART3_MMODE_PRIORITY_THRESHOLD,
            ExecutionMode::Supervisor => PILC_HART3_SMODE_PRIORITY_THRESHOLD,
        },
        HartId::Hart4 => match execution_mode {
            ExecutionMode::Machine => PILC_HART4_MMODE_PRIORITY_THRESHOLD,
            ExecutionMode::Supervisor => PILC_HART4_SMODE_PRIORITY_THRESHOLD,
        },
    };
    unsafe {
        ptr::write_volatile(reg as *mut u32, threshold & 0b111);
    }
}

fn plic_claim(hart: &HartId, execution_mode: ExecutionMode) -> u32 {
    let reg = match hart {
        HartId::Hart0 => PILC_HART0_MMODE_CLAIM_COMPLETE,
        HartId::Hart1 => match execution_mode {
            ExecutionMode::Machine => PILC_HART1_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART1_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart2 => match execution_mode {
            ExecutionMode::Machine => PILC_HART2_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART2_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart3 => match execution_mode {
            ExecutionMode::Machine => PILC_HART3_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART3_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart4 => match execution_mode {
            ExecutionMode::Machine => PILC_HART4_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART4_SMODE_CLAIM_COMPLETE,
        },
    };
    unsafe { ptr::read_volatile(reg as *const u32) }
}

fn plic_complete(hart: &HartId, execution_mode: ExecutionMode, interrupt_number: u32) {
    let reg = match hart {
        HartId::Hart0 => PILC_HART0_MMODE_CLAIM_COMPLETE,
        HartId::Hart1 => match execution_mode {
            ExecutionMode::Machine => PILC_HART1_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART1_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart2 => match execution_mode {
            ExecutionMode::Machine => PILC_HART2_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART2_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart3 => match execution_mode {
            ExecutionMode::Machine => PILC_HART3_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART3_SMODE_CLAIM_COMPLETE,
        },
        HartId::Hart4 => match execution_mode {
            ExecutionMode::Machine => PILC_HART4_MMODE_CLAIM_COMPLETE,
            ExecutionMode::Supervisor => PILC_HART4_SMODE_CLAIM_COMPLETE,
        },
    };
    unsafe {
        ptr::write_volatile(reg as *mut u32, interrupt_number);
    }
}

fn set_bit(reg: u32, bit: u32) {
    unsafe {
        let regval = ptr::read_volatile(reg as *const u32);
        ptr::write_volatile(reg as *mut u32, regval | 1 << bit);
    }
}

pub fn print_pending_interrupt_info() {
    for i in 1..137 {
        let reg_offset: u32 = 4 * (i / 32);
        let bit_mask: u32 = 1 << (i % 32);
        unsafe {
            let val = ptr::read_volatile((PLIC_PENDING + reg_offset) as *const u32);
            let set = (val & bit_mask) > 0;
            println!("Interrupt: {}, Pending: {}", i, set);
        }
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
//pac::interrupt!(SYS_IOMUX, default_handler);
pac::interrupt!(HOST0, default_handler);
pac::interrupt!(PERIPHERAL0, default_handler);
pac::interrupt!(OTG0, default_handler);
pac::interrupt!(PMU, default_handler);
