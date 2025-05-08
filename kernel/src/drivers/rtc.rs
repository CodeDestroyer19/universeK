//! Real-Time Clock (RTC) driver
//! Provides date and time functionality

use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::{Port, PortWriteOnly};
use crate::errors::KernelError;
use crate::serial_println;

// CMOS/RTC ports
const CMOS_ADDR_PORT: u16 = 0x70;
const CMOS_DATA_PORT: u16 = 0x71;

// RTC registers
const RTC_SECONDS: u8 = 0x00;
const RTC_MINUTES: u8 = 0x02;
const RTC_HOURS: u8 = 0x04;
const RTC_DAY_OF_MONTH: u8 = 0x07;
const RTC_MONTH: u8 = 0x08;
const RTC_YEAR: u8 = 0x09;
const RTC_CENTURY: u8 = 0x32; // May be different on some hardware

// Status registers
const RTC_STATUS_A: u8 = 0x0A;
const RTC_STATUS_B: u8 = 0x0B;
const RTC_STATUS_C: u8 = 0x0C;
const RTC_STATUS_D: u8 = 0x0D;

// Status register bit flags
const RTC_UIP: u8 = 0x80; // Update in progress flag (Status A)
const RTC_DM: u8 = 0x04;  // Data Mode: 0 = BCD, 1 = Binary (Status B)
const RTC_24H: u8 = 0x02; // Hour Format: 0 = 12h, 1 = 24h (Status B)
const RTC_DST: u8 = 0x01; // Daylight Savings Time enable (Status B)

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl DateTime {
    pub fn new() -> Self {
        Self {
            second: 0,
            minute: 0,
            hour: 0,
            day: 1,
            month: 1,
            year: 2000,
        }
    }
    
    pub fn format(&self) -> alloc::string::String {
        use alloc::format;
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
            self.year, self.month, self.day,
            self.hour, self.minute, self.second)
    }
}

struct RtcDriver {
    addr_port: PortWriteOnly<u8>,
    data_port: Port<u8>,
}

impl RtcDriver {
    fn new() -> Self {
        Self {
            addr_port: PortWriteOnly::new(CMOS_ADDR_PORT),
            data_port: Port::new(CMOS_DATA_PORT),
        }
    }
    
    fn init(&mut self) -> Result<(), KernelError> {
        serial_println!("Initializing RTC driver");
        
        // Read status registers for debugging
        let status_b = self.read_register(RTC_STATUS_B);
        
        let hour_format = if status_b & RTC_24H != 0 { "24-hour" } else { "12-hour" };
        let data_mode = if status_b & RTC_DM != 0 { "binary" } else { "BCD" };
        
        serial_println!("RTC configured with {} format, {} data mode", hour_format, data_mode);
        
        // Read the current time
        let now = self.read_datetime();
        serial_println!("Current RTC time: {}", now.format());
        
        Ok(())
    }
    
    fn read_register(&mut self, register: u8) -> u8 {
        unsafe {
            // Disable NMI (high bit set) while reading
            self.addr_port.write(register | 0x80);
            self.data_port.read()
        }
    }
    
    fn write_register(&mut self, register: u8, value: u8) {
        unsafe {
            // Disable NMI (high bit set) while writing
            self.addr_port.write(register | 0x80);
            self.data_port.write(value);
        }
    }
    
    fn bcd_to_binary(&self, value: u8) -> u8 {
        // Convert from BCD to binary
        // Example: 0x42 (BCD for 42) = 4*10 + 2 = 42 (binary)
        ((value >> 4) * 10) + (value & 0x0F)
    }
    
    fn read_datetime(&mut self) -> DateTime {
        // Wait until RTC is not updating
        while self.read_register(RTC_STATUS_A) & RTC_UIP != 0 {
            // Spin until update is complete
        }
        
        // Get the RTC values
        let seconds = self.read_register(RTC_SECONDS);
        let minutes = self.read_register(RTC_MINUTES);
        let hours = self.read_register(RTC_HOURS);
        let day = self.read_register(RTC_DAY_OF_MONTH);
        let month = self.read_register(RTC_MONTH);
        let year = self.read_register(RTC_YEAR);
        
        // Get century if available
        let century = self.read_register(RTC_CENTURY);
        
        // Read status register B
        let status_b = self.read_register(RTC_STATUS_B);
        
        // Convert BCD to binary if needed
        let (seconds, minutes, hours, day, month, year, century) = if status_b & RTC_DM == 0 {
            // BCD mode, convert to binary
            (
                self.bcd_to_binary(seconds),
                self.bcd_to_binary(minutes),
                self.bcd_to_binary(hours & 0x7F), // Remove AM/PM bit if present
                self.bcd_to_binary(day),
                self.bcd_to_binary(month),
                self.bcd_to_binary(year),
                if century != 0 { self.bcd_to_binary(century) } else { 20 } // Default to 21st century
            )
        } else {
            // Binary mode, use as-is
            (seconds, minutes, hours & 0x7F, day, month, year, if century != 0 { century } else { 20 })
        };
        
        // Convert 12-hour to 24-hour if needed
        let hours = if status_b & RTC_24H == 0 && hours & 0x80 != 0 {
            // 12-hour mode with PM bit set
            (hours & 0x7F) + 12
        } else {
            // 24-hour mode or AM
            hours
        };
        
        // Construct the full year
        let full_year = (century as u16 * 100) + year as u16;
        
        DateTime {
            second: seconds,
            minute: minutes,
            hour: hours,
            day,
            month,
            year: full_year,
        }
    }
}

lazy_static! {
    static ref RTC: Mutex<RtcDriver> = Mutex::new(RtcDriver::new());
}

/// Initialize the RTC driver
pub fn init() -> Result<(), KernelError> {
    RTC.lock().init()
}

/// Get the current date and time from the RTC
pub fn get_datetime() -> DateTime {
    RTC.lock().read_datetime()
}

/// Sleep for a given number of seconds using the RTC
pub fn sleep(seconds: u32) {
    let start = get_datetime();
    let mut elapsed = 0;
    
    while elapsed < seconds {
        let now = get_datetime();
        
        // Simple elapsed time calculation
        // This is basic and doesn't handle day/month/year boundaries well
        if now.minute != start.minute || now.hour != start.hour {
            // Minutes changed, recalculate fully
            elapsed = seconds; // Just exit for now
        } else if now.second >= start.second {
            elapsed = (now.second - start.second) as u32;
        } else {
            // Handle second wrapping around from 59 to 0
            elapsed = (60 + now.second - start.second) as u32;
        }
        
        // Use HLT to pause the CPU
        x86_64::instructions::hlt();
    }
} 