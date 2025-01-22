use crate::gcr::clocks::{Clock, SystemClock};

pub const FLASH_BASE:       u32 = 0x1000_0000;
pub const FLASH_SIZE:       u32 = 0x0008_0000;
pub const FLASH_END:        u32 = FLASH_BASE + FLASH_SIZE;
pub const FLASH_PAGE_SIZE:  u32 = 0x2000;

// TODO:
// - Correctly implement write_128 (can we even use u128)?
// - Implement write_32
// - Return Result instead of FlashStatus
// - Write tests
// - Document

pub enum FlashStatus {
    Ok,
    InvalidAddress,
    AccessViolation,
    NeedsErase,
}

pub struct Flc {
    flc: crate::pac::Flc,
}

impl Flc {
    // #[link_section = ".flashprog"]
    // #[inline(never)]
    pub fn new(flc: crate::pac::Flc, sys_clk: &Clock<SystemClock>) -> Self {
        let s = Self { flc };
        while s.is_busy() {}
        // Calculate FLC divisor such that FLC frequency is 1 MHz
        let flc_div = sys_clk.frequency / 1_000_000;
        // Set FLC divisor
        s.flc.clkdiv().modify(|_, w| unsafe {
            w.clkdiv().bits(flc_div as u8)
        });
        // Clear stale interrupts
        if s.flc.intr().read().af().bit_is_set() {
            s.flc.intr().write(|w| w.af().clear_bit());
        }
        return s;
    }

    /// Check if the flash controller is busy.
    // #[link_section = ".flashprog"]
    // #[inline(never)]
    #[inline]
    pub fn is_busy(&self) -> bool {
        let ctrl = self.flc.ctrl().read();
        ctrl.pend().is_busy() ||
        ctrl.pge().bit_is_set() ||
        ctrl.me().bit_is_set() ||
        ctrl.wr().bit_is_set()
    }

    /// Unlock the flash controller to allow write or erase operations.
    #[inline]
    fn unlock_flash(&self) {
        self.flc.ctrl().modify(|_, w| w.unlock().unlocked());
        while self.flc.ctrl().read().unlock().is_locked() {}
    }

    /// Lock the flash controller to prevent write or erase operations.
    #[inline]
    fn lock_flash(&self) {
        self.flc.ctrl().modify(|_, w| w.unlock().locked());
        while self.flc.ctrl().read().unlock().is_unlocked() {}
    }

    /// Set the target address for a write or erase operation.
    fn set_address(&self, address: u32) -> FlashStatus {
        // Ensure that the address is within the flash memory range and aligned to a page boundary
        if address < FLASH_BASE || address >= FLASH_END || address & (FLASH_PAGE_SIZE - 1) != 0 {
            return FlashStatus::InvalidAddress;
        }
        // Convert to physical address
        let phys_addr = address & (FLASH_SIZE - 1);
        // Safety: We have validated the address already
        self.flc.addr().write(|w| unsafe {
            w.addr().bits(phys_addr)
        });
        FlashStatus::Ok
    }

    /// Commit a write operation.
    #[link_section = ".flashprog"]
    // #[inline(never)]
    #[inline]
    fn commit_write(&self) {
        self.flc.ctrl().modify(|_, w| w.wr().set_bit());
        while self.flc.ctrl().read().wr().bit_is_set() {}
        while self.is_busy() {}
    }

    /// Commit a page erase operation.
    #[link_section = ".flashprog"]
    // #[inline(never)]
    #[inline]
    fn commit_erase(&self) {
        self.flc.ctrl().modify(|_, w| w.pge().set_bit());
        while self.flc.ctrl().read().pge().bit_is_set() {}
        while self.is_busy() {}
    }

    /// Write a 128-bit word to flash memory.
    ///
    /// # Safety
    /// Writes are only successful if the target address is already erased or
    /// the bits that change between the existing value at the target address
    /// and the new value are only 1 -> 0 transitions.
    ///
    /// Executing from flash memory while writing to flash memory can result
    /// in a crash or undefined behavior. The user should ensure that the
    /// `.flashprog` section is correctly placed in SRAM during the linking
    /// process, and that code in the `.flashprog` section can execute from
    /// SRAM.
    #[link_section = ".flashprog"]
    #[inline(never)]
    pub unsafe fn write_128(&self, address: u32, data: u128) -> FlashStatus {
        // Target address must be 128-bit aligned
        if address & 0b1111 != 0 {
            return FlashStatus::InvalidAddress;
        }
        // Ensure that the flash controller is not busy
        while self.is_busy() {}
        // Unlock the flash controller
        self.unlock_flash();
        // Set the address to write to
        match self.set_address(address) {
            FlashStatus::Ok => {},
            FlashStatus::InvalidAddress => {
                self.lock_flash();
                return FlashStatus::InvalidAddress;
            },
            _ => {}, // ????
        }
        // Set the data to write
        // self.flc.data().write(|w| unsafe {
        //     w.data().bits(data)
        // });
        // Commit the write operation
        self.commit_write();
        // Lock the flash controller
        self.lock_flash();
        // Check for access violation
        if self.flc.intr().read().af().bit_is_set() {
            self.flc.intr().write(|w| w.af().clear_bit());
            return FlashStatus::AccessViolation;
        }
        FlashStatus::Ok
    }

    #[link_section = ".flashprog"]
    #[inline(never)]
    pub unsafe fn write_32(&self, address: u32, data: u32) -> FlashStatus {
        // Target address must be 32-bit aligned
        if address & 0b11 != 0 {
            return FlashStatus::InvalidAddress;
        }
        FlashStatus::InvalidAddress
    }

    /// Erases a page in flash memory.
    ///
    /// # Safety
    /// Care must be taken to not erase the page containing the executing code.
    #[link_section = ".flashprog"]
    #[inline(never)]
    pub unsafe fn erase_page(&self, address: u32) -> FlashStatus {
        // Ensure that the flash controller is not busy
        while self.is_busy() {}
        // Unlock the flash controller
        self.unlock_flash();
        // Set the address to erase
        match self.set_address(address) {
            FlashStatus::Ok => {},
            FlashStatus::InvalidAddress => {
                self.lock_flash();
                return FlashStatus::InvalidAddress;
            },
            _ => {}, // ????
        }
        // Set erase page code
        self.flc.ctrl().modify(|_, w| w.erase_code().erase_page());
        // Commit the erase operation
        self.commit_erase();
        // Lock the flash controller
        self.lock_flash();
        // Check for access violation
        if self.flc.intr().read().af().bit_is_set() {
            self.flc.intr().write(|w| w.af().clear_bit());
            return FlashStatus::AccessViolation;
        }
        FlashStatus::Ok
    }

    /// Protects a page in flash memory from write or erase operations.
    /// Effective until the next external or power-on reset.
    pub fn disable_page_write(&self, address: u32) -> FlashStatus {
        // Ensure that the flash controller is not busy
        while self.is_busy() {}
        // Convert to page number
        let page_num_bit = (address >> 13) & 63;
        // Check for invalid page number
        if page_num_bit >= 64 {
            return FlashStatus::InvalidAddress;
        }
        // Lock based on page number
        if page_num_bit < 32 {
            let write_lock_bit = 1 << page_num_bit;
            self.flc.welr0().write(|w| unsafe {
                w.bits(write_lock_bit)
            });
            while self.flc.welr0().read().bits() & write_lock_bit == write_lock_bit {}
        } else {
            let write_lock_bit = 1 << (page_num_bit - 32);
            self.flc.welr1().write(|w| unsafe {
                w.bits(write_lock_bit)
            });
            while self.flc.welr1().read().bits() & write_lock_bit == write_lock_bit {}
        }
        FlashStatus::Ok
    }

    /// Protects a page in flash memory from read operations.
    /// Effective until the next external or power-on reset.
    pub fn disable_page_read(&self, address: u32) -> FlashStatus {
        // Ensure that the flash controller is not busy
        while self.is_busy() {}
        // Convert to page number
        let page_num_bit = (address >> 13) & 63;
        // Check for invalid page number
        if page_num_bit >= 64 {
            return FlashStatus::InvalidAddress;
        }
        // Lock based on page number
        if page_num_bit < 32 {
            let read_lock_bit = 1 << page_num_bit;
            self.flc.rlr0().write(|w| unsafe {
                w.bits(read_lock_bit)
            });
            while self.flc.rlr0().read().bits() & read_lock_bit == read_lock_bit {}
        } else {
            let read_lock_bit = 1 << (page_num_bit - 32);
            self.flc.rlr1().write(|w| unsafe {
                w.bits(read_lock_bit)
            });
            while self.flc.rlr1().read().bits() & read_lock_bit == read_lock_bit {}
        }
        FlashStatus::Ok
    }
}