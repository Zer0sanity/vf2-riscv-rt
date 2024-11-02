#![allow(unused)]
use core::{marker::PhantomData, ptr};

use crate::println;

#[repr(u8)]
#[derive(Debug)]
pub enum TimerIntStatus {
    None = 0,
    Pending = 1,
}

impl TimerIntStatus {
    fn from_u32(value: u32) -> TimerIntStatus {
        match value {
            0 => TimerIntStatus::None,
            _ => TimerIntStatus::Pending,
        }
    }
}

///Timer control for Continuous run or Single (One-Shot) run.
#[repr(u8)]
#[derive(Debug)]
pub enum TimerControl {
    Continuous = 0,
    Single = 1,
}

impl TimerControl {
    fn from_u32(value: u32) -> TimerControl {
        match value {
            0 => TimerControl::Continuous,
            _ => TimerControl::Single,
        }
    }
}

///Timer enable/disable flag
#[repr(u8)]
#[derive(Debug)]
pub enum TimerEnable {
    Disable = 0,
    Enable = 1,
}

impl TimerEnable {
    fn from_u32(value: u32) -> TimerEnable {
        match value {
            0 => TimerEnable::Disable,
            _ => TimerEnable::Enable,
        }
    }
}

/// Timer interrupt clear busy flag
#[repr(u8)]
#[derive(Debug)]
pub enum TimerIntClearBusy {
    No = 0,
    Yes = 1,
}

impl TimerIntClearBusy {
    fn from_u32(value: u32) -> TimerIntClearBusy {
        match value {
            0 => TimerIntClearBusy::No,
            _ => TimerIntClearBusy::Yes,
        }
    }
}

/// Timer interrupt clear status flag
#[repr(u8)]
#[derive(Debug)]
pub enum TimerIntClearStatus {
    None = 0,
    Clear = 1,
}

impl TimerIntClearStatus {
    fn from_u32(value: u32) -> TimerIntClearStatus {
        match value {
            0 => TimerIntClearStatus::None,
            _ => TimerIntClearStatus::Clear,
        }
    }
}

/// Timer interrupt mask flag
#[repr(u8)]
#[derive(Debug)]
pub enum TimerIntMask {
    Unmask = 0,
    Mask = 1,
}

impl TimerIntMask {
    fn from_u32(value: u32) -> TimerIntMask {
        match value {
            0 => TimerIntMask::Unmask,
            _ => TimerIntMask::Mask,
        }
    }
}

pub trait Timer {
    /// Timer Bias (basically an offset), must be implimented by struct
    /// implimentation
    const CHANNEL: u32;

    /// Define timer register offsets
    const INT_STATUS_REG_OFFSET: u32 = 0x00;
    const CONTROL_REG_OFFSET: u32 = 0x04;
    const LOAD_REG_OFFSET: u32 = 0x08;
    const ENABLE_REG_OFFSET: u32 = 0x10;
    const RELOAD_REG_OFFSET: u32 = 0x14;
    const VALUE_REG_OFFSET: u32 = 0x18;
    const INT_STATUS_CLEAR_REG_OFFSET: u32 = 0x20;
    const INT_STATUS_CLEAR_BUSY_BIT: u8 = 1;
    const INT_MASK_REG_OFFSET: u32 = 0x24;

    /// System memory map start address of timer device registers
    const TIMER_REG_BASE: u32 = 0x1305_0000;
    const CHANNEL_LENGTH: u32 = 0x40;
    const CHANNEL_REG_BASE: u32 = Self::TIMER_REG_BASE + Self::CHANNEL_LENGTH * Self::CHANNEL;

    //Define the actual registers
    const INT_STATUS_REG: u32 = Self::TIMER_REG_BASE;
    const CONTROL_REG: u32 = Self::CHANNEL_REG_BASE + Self::CONTROL_REG_OFFSET;
    const LOAD_REG: u32 = Self::CHANNEL_REG_BASE + Self::LOAD_REG_OFFSET;
    const ENABLE_REG: u32 = Self::CHANNEL_REG_BASE + Self::ENABLE_REG_OFFSET;
    const RELOAD_REG: u32 = Self::CHANNEL_REG_BASE + Self::RELOAD_REG_OFFSET;
    const VALUE_REG: u32 = Self::CHANNEL_REG_BASE + Self::VALUE_REG_OFFSET;
    const INT_STATUS_CLEAR_REG: u32 = Self::CHANNEL_REG_BASE + Self::INT_STATUS_CLEAR_REG_OFFSET;
    const INT_MASK_REG: u32 = Self::CHANNEL_REG_BASE + Self::INT_MASK_REG_OFFSET;

    /// Reads the timers interrupt status for if an interrupt is pending or not
    fn get_int_status(&self) -> TimerIntStatus {
        // Read the value
        let mut value: u32 = unsafe { ptr::read_volatile(Self::INT_STATUS_REG as *const u32) };
        // Mask off the other channels
        value &= 1 << Self::CHANNEL;
        // Shift the channel down
        value >>= Self::CHANNEL;
        // Convert to the enum
        TimerIntStatus::from_u32(value)
    }

    /// GET/SET timers control for running in Continuous or Single (One-Shot) mode
    fn get_control(&self) -> TimerControl {
        let value: u32 = unsafe { ptr::read_volatile(Self::CONTROL_REG as *const u32) };
        TimerControl::from_u32(value)
    }

    fn set_control(&self, control: TimerControl) {
        unsafe { ptr::write_volatile(Self::CONTROL_REG as *mut u32, control as u32) };
    }

    /// GET/SET the load value.  The initial value to be loaded into the counter
    /// and is also used as the reloaded value
    fn get_load(&self) -> u32 {
        unsafe { ptr::read_volatile(Self::LOAD_REG as *const u32) }
    }
    ///Set value to be loaded into the counter
    fn set_load(&self, load: u32) {
        unsafe { ptr::write_volatile(Self::LOAD_REG as *mut u32, load as u32) };
    }
    /// GET/SET Timer enable/disable flag
    fn get_enable(&self) -> TimerEnable {
        let value: u32 = unsafe { ptr::read_volatile(Self::ENABLE_REG as *const u32) };
        TimerEnable::from_u32(value)
    }

    fn set_enable(&self, enable: TimerEnable) {
        unsafe { ptr::write_volatile(Self::ENABLE_REG as *mut u32, enable as u32) };
    }

    /// Reload preset value (load) to counter
    fn reload_counter(&self) {
        unsafe { ptr::write_volatile(Self::RELOAD_REG as *mut u32, 0x1) };
    }

    /// GET Timer counter value
    fn get_counter(&self) -> u32 {
        unsafe { ptr::read_volatile(Self::VALUE_REG as *const u32) }
    }

    /// GET Timer interrupt clear busy flag
    fn get_int_clear_busy(&self) -> TimerIntClearBusy {
        let mut value: u32 =
            unsafe { ptr::read_volatile(Self::INT_STATUS_CLEAR_REG as *const u32) };
        println!("Int Clear Status: {value}");
        value >>= Self::INT_STATUS_CLEAR_BUSY_BIT;
        TimerIntClearBusy::from_u32(value)
    }

    /// GET/SET Timer clear status flag
    fn get_int_status_clear(&self) -> TimerIntClearStatus {
        let mut value: u32 =
            unsafe { ptr::read_volatile(Self::INT_STATUS_CLEAR_REG as *const u32) };
        value &= 0x1;
        TimerIntClearStatus::from_u32(value)
    }

    fn set_int_status_clear(&self, status: TimerIntClearStatus) {
        unsafe { ptr::write_volatile(Self::INT_STATUS_CLEAR_REG as *mut u32, status as u32) };
    }

    /// GET/SET Timer interrupt mask
    fn get_int_mask(&self) -> TimerIntMask {
        let value: u32 = unsafe { ptr::read_volatile(Self::INT_MASK_REG as *const u32) };
        TimerIntMask::from_u32(value)
    }
    fn set_int_mask(&self, mask: TimerIntMask) {
        unsafe { ptr::write_volatile(Self::INT_MASK_REG as *mut u32, mask as u32) }
    }

    /// Prints timer information
    fn print_debug_info(&self) {
        println!("TIMER_REG_BASE:       {:#10x}", Self::TIMER_REG_BASE);
        println!("CHANNEL_LENGTH:       {:#10x}", Self::CHANNEL_LENGTH);
        println!("CHANNEL_REG_BASE:     {:#10x}", Self::CHANNEL_REG_BASE);

        println!(
            "INT_STATUS_REG        {:#10x}: {:#10X}",
            Self::INT_STATUS_REG,
            unsafe { ptr::read_volatile(Self::INT_STATUS_REG as *const u32) }
        );
        println!(
            "CONTROL_REG           {:#10x}: {:#10X}",
            Self::CONTROL_REG,
            unsafe { ptr::read_volatile(Self::CONTROL_REG as *const u32) }
        );
        println!(
            "LOAD_REG              {:#10x}: {:#10X}",
            Self::LOAD_REG,
            unsafe { ptr::read_volatile(Self::LOAD_REG as *const u32) }
        );
        println!(
            "ENABLE_REG            {:#10x}: {:#10X}",
            Self::ENABLE_REG,
            unsafe { ptr::read_volatile(Self::ENABLE_REG as *const u32) }
        );
        println!(
            "RELOAD_REG            {:#10x}: {:#10X}",
            Self::RELOAD_REG,
            unsafe { ptr::read_volatile(Self::RELOAD_REG as *const u32) }
        );
        println!(
            "VALUE_REG             {:#10x}: {:#10X}",
            Self::VALUE_REG,
            unsafe { ptr::read_volatile(Self::VALUE_REG as *const u32) }
        );
        println!(
            "INT_STATUS_CLEAR_REG  {:#10x}: {:#10X}",
            Self::INT_STATUS_CLEAR_REG,
            unsafe { ptr::read_volatile(Self::INT_STATUS_CLEAR_REG as *const u32) }
        );
        println!(
            "INT_MASK_REG          {:#10x}: {:#10X}",
            Self::INT_MASK_REG,
            unsafe { ptr::read_volatile(Self::INT_MASK_REG as *const u32) }
        );
        println!();
    }
}

/// Struct to access Timer0
pub struct Timer0 {
    /// I dont know why I should put this here
    _marker: PhantomData<*const ()>,
}

impl Timer0 {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl Timer for Timer0 {
    const CHANNEL: u32 = 0;
}

/// Struct to access Timer1
struct Timer1 {
    /// I dont know why I should put this here
    _marker: PhantomData<*const ()>,
}

impl Timer for Timer1 {
    const CHANNEL: u32 = 1;
}

/// Struct to access Timer2
struct Timer2 {
    /// I dont know why I should put this here
    _marker: PhantomData<*const ()>,
}

impl Timer for Timer2 {
    const CHANNEL: u32 = 2;
}

/// Struct to access Timer3
struct Timer3 {
    /// I dont know why I should put this here
    _marker: PhantomData<*const ()>,
}

impl Timer for Timer3 {
    const CHANNEL: u32 = 3;
}
