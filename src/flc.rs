//! # Flash Controller (FLC)
use crate::gcr::clocks::{Clock, SystemClock};

/// Base address of the flash memory.
pub const FLASH_BASE: u32 = 0x1000_0000;
/// Size of the flash memory.
pub const FLASH_SIZE: u32 = 0x0008_0000;
/// End address of the flash memory.
pub const FLASH_END: u32 = FLASH_BASE + FLASH_SIZE;
/// Number of flash pages.
pub const FLASH_PAGE_COUNT: u32 = 64;
/// Size of a flash page.
pub const FLASH_PAGE_SIZE: u32 = 0x2000;

/// Flash controller errors.
#[derive(Debug, PartialEq)]
pub enum FlashError {
    /// The target address to write or erase is invalid.
    InvalidAddress,
    /// The page number is invalid
    InvalidPage,
    /// The flash controller was busy or locked when attempting to write or erase.
    AccessViolation,
    /// Writing over the old data with new data would cause 0 -> 1 bit transitions.
    /// The target address must be erased before writing new data.
    NeedsErase,
}

/// # Flash Controller (FLC) Peripheral
///
/// The flash controller manages read, write, and erase accesses to the
/// internal flash and provides the following features:
/// - Up to 512 KiB of flash memory
/// - 64 pages (8192 bytes per page)
/// - 2048 words by 128 bits per page
/// - 128-bit write granularity
/// - Page erase and mass erase
/// - Read and write protection
///
/// Example:
/// ```
/// let flc = Flc::new(p.flc, sys_clk);
///
/// // Erase page number 48
/// unsafe { flc.erase_page(0x1006_0000).unwrap(); }
/// // Read the value at address 0x1006_0004
/// let data: u32 = flc.read_32(0x1006_0004).unwrap();
/// // Should be 0xFFFFFFFF since flash defaults to all 1's
/// assert_eq!(data, 0xFFFF_FFFF);
///
/// // Write a value to address 0x1006_0004
/// flc.write_32(0x1006_0004, 0x7856_3412).unwrap();
/// // Read the data back from flash memory
/// let new_data: u32 = flc.read_32(0x1006_0004).unwrap();
/// assert_eq!(new_data, 0x7856_3412);
/// ```
pub struct Flc {
    flc: crate::pac::Flc,
    sys_clk: Clock<SystemClock>,
}

impl Flc {
    /// Construct a new flash controller peripheral.
    pub fn new(flc: crate::pac::Flc, sys_clk: Clock<SystemClock>) -> Self {
        let s = Self { flc, sys_clk };
        s.config();
        s
    }

    /// Configure the flash controller.
    #[inline]
    fn config(&self) {
        // Wait until the flash controller is not busy
        while self.is_busy() {}
        // Set FLC divisor
        let flc_div = self.sys_clk.frequency / 1_000_000;
        self.flc
            .clkdiv()
            .modify(|_, w| unsafe { w.clkdiv().bits(flc_div as u8) });
        // Clear stale interrupts
        if self.flc.intr().read().af().bit_is_set() {
            self.flc.intr().write(|w| w.af().clear_bit());
        }
    }

    /// Check if the flash controller is busy.
    #[inline]
    pub fn is_busy(&self) -> bool {
        let ctrl = self.flc.ctrl().read();
        ctrl.pend().is_busy()
            || ctrl.pge().bit_is_set()
            || ctrl.me().bit_is_set()
            || ctrl.wr().bit_is_set()
    }

    /// Check if an address is within the valid flash memory range.
    #[inline]
    pub fn check_address(&self, address: u32) -> Result<(), FlashError> {
        if address < FLASH_BASE || address >= FLASH_END {
            return Err(FlashError::InvalidAddress);
        }
        Ok(())
    }

    /// Check if an address is within the valid flash memory range.
    #[inline]
    pub fn check_page_number(&self, page_number: u32) -> Result<(), FlashError> {
        if page_number >= FLASH_PAGE_COUNT {
            return Err(FlashError::InvalidPage);
        }
        Ok(())
    }

    /// Get the base address of a page
    #[inline]
    pub fn get_address(&self, page_number: u32) -> Result<u32, FlashError> {
        self.check_page_number(page_number)?;

        let address = FLASH_BASE + FLASH_PAGE_SIZE * page_number;

        Ok(address)
    }
    
    /// Get the page number of a flash address.
    #[inline]
    pub fn get_page_number(&self, address: u32) -> Result<u32, FlashError> {
        self.check_address(address)?;
        let page_num = (address >> 13) & (FLASH_PAGE_COUNT - 1);
        // Check for invalid page number (redundant check)
        if page_num >= FLASH_PAGE_COUNT {
            return Err(FlashError::InvalidAddress);
        }
        Ok(page_num)
    }

    /// Set the target address for a write or erase operation.
    #[inline]
    fn set_address(&self, address: u32) -> Result<(), FlashError> {
        self.check_address(address)?;
        // Convert to physical address
        let phys_addr = address & (FLASH_SIZE - 1);
        // Safety: We have validated the address already
        self.flc
            .addr()
            .write(|w| unsafe { w.addr().bits(phys_addr) });
        Ok(())
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

    /// Commit a write operation.
    #[cfg_attr(feature = "flashprog-linkage", link_section = ".flashprog")]
    #[inline]
    fn commit_write(&self) {
        self.flc.ctrl().modify(|_, w| w.wr().start());
        while !self.flc.ctrl().read().wr().is_complete() {}
        while self.is_busy() {}
    }

    /// Commit a page erase operation.
    #[cfg_attr(feature = "flashprog-linkage", link_section = ".flashprog")]
    #[inline]
    fn commit_erase(&self) {
        self.flc.ctrl().modify(|_, w| w.pge().start());
        while !self.flc.ctrl().read().pge().is_complete() {}
        while self.is_busy() {}
    }

    /// Write a 128-bit word to flash memory. This is an internal function to
    /// be used by all other write functions.
    #[doc(hidden)]
    #[cfg_attr(feature = "flashprog-linkage", link_section = ".flashprog")]
    #[inline(never)]
    fn _write_128(&self, address: u32, data: &[u32; 4]) -> Result<(), FlashError> {
        // Target address must be 128-bit aligned
        if address & 0b1111 != 0 {
            return Err(FlashError::InvalidAddress);
        }
        self.check_address(address)?;
        // Ensure that the flash controller is configured
        self.config();
        // Verify that only 1 -> 0 transitions are being made by reading the existing data at the target address
        for i in 0..4 {
            // Safety: We have checked the address already
            let old_data = unsafe { core::ptr::read_volatile((address + i * 4) as *const u32) };
            if (old_data & data[i as usize]) != data[i as usize] {
                return Err(FlashError::NeedsErase);
            }
        }
        self.set_address(address)?;
        // Safety: Data can be written to all bits of the data registers
        unsafe {
            self.flc.data(0).write(|w| w.data().bits(data[0]));
            self.flc.data(1).write(|w| w.data().bits(data[1]));
            self.flc.data(2).write(|w| w.data().bits(data[2]));
            self.flc.data(3).write(|w| w.data().bits(data[3]));
        }
        self.unlock_flash();
        // Commit the write operation
        self.commit_write();
        self.lock_flash();
        // Check for access violation
        if self.flc.intr().read().af().bit_is_set() {
            self.flc.intr().write(|w| w.af().clear_bit());
            return Err(FlashError::AccessViolation);
        }
        Ok(())
    }

    /// Erases a page in flash memory.
    #[doc(hidden)]
    #[cfg_attr(feature = "flashprog-linkage", link_section = ".flashprog")]
    #[inline(never)]
    fn _erase_page(&self, address: u32) -> Result<(), FlashError> {
        while self.is_busy() {}
        self.set_address(address)?;
        self.unlock_flash();
        // Set erase page code
        self.flc.ctrl().modify(|_, w| w.erase_code().erase_page());
        // Commit the erase operation
        self.commit_erase();
        self.lock_flash();
        // Check for access violation
        if self.flc.intr().read().af().bit_is_set() {
            self.flc.intr().write(|w| w.af().clear_bit());
            return Err(FlashError::AccessViolation);
        }
        Ok(())
    }

    /// Writes four [`u32`] to flash memory. Uses little-endian byte order.
    /// The lowest [`u32`] in the array is written to the lowest address in flash.
    /// The target address must be 128-bit aligned.
    ///
    /// Example:
    /// ```
    /// let data: [u32; 4] = [0x0403_0201, 0x0807_0605, 0x0C0B_0A09, 0x100F_0E0D];
    /// flash.write_128(0x1006_0000, &data).unwrap();
    /// // The bytes in flash will look like:
    /// // 10060000: 0102 0304 0506 0708 090A 0B0C 0D0E 0F10
    /// ```
    pub fn write_128(&self, address: u32, data: &[u32; 4]) -> Result<(), FlashError> {
        self._write_128(address, &data)
    }

    /// Write a [`u32`] to flash memory. Uses little-endian byte order.
    /// The target address must be 32-bit aligned.
    ///
    /// Note: Writes to flash memory must be done in 128-bit (16-byte) blocks.
    /// This function will read the existing 128-bit word containing the target
    /// address, modify the 32-bit word within the 128-bit word, and write the
    /// modified 128-bit word back to flash memory.
    ///
    /// Example:
    /// ```
    /// let data: u32 = 0x7856_3412;
    /// flash.write_32(0x1006_0004, data).unwrap();
    /// // The bytes in flash will look like:
    /// // 10060000: FFFF FFFF 1234 5678 FFFF FFFF FFFF FFFF
    /// ```
    pub fn write_32(&self, address: u32, data: u32) -> Result<(), FlashError> {
        // Target address must be 32-bit aligned
        if address & 0b11 != 0 {
            return Err(FlashError::InvalidAddress);
        }
        self.check_address(address)?;
        let addr_128 = address & !0b1111;
        self.check_address(addr_128)?;
        let addr_128_ptr = addr_128 as *const u32;
        // Read existing data at the 128-bit word containing the target address
        let mut prev_data: [u32; 4] = [0xFFFF_FFFF; 4];
        // Safety: We have checked the address already
        unsafe {
            prev_data[0] = core::ptr::read_volatile(addr_128_ptr);
            prev_data[1] = core::ptr::read_volatile(addr_128_ptr.offset(1));
            prev_data[2] = core::ptr::read_volatile(addr_128_ptr.offset(2));
            prev_data[3] = core::ptr::read_volatile(addr_128_ptr.offset(3));
        }
        // Determine index of the 32-bit word within the 128-bit word
        let data_idx = (address & 0b1100) >> 2;
        // Modify the 32-bit word within the 128-bit word
        prev_data[data_idx as usize] = data;
        // Write the modified 128-bit word to flash memory
        self._write_128(addr_128, &prev_data)
    }

    /// Reads four [`u32`] from flash memory. Uses little-endian byte order.
    /// The lowest [`u32`] in the array is read from the lowest address in flash.
    /// The target address must be 128-bit aligned.
    pub fn read_128(&self, address: u32) -> Result<[u32; 4], FlashError> {
        // Target address must be 128-bit aligned
        if address & 0b1111 != 0 {
            return Err(FlashError::InvalidAddress);
        }
        self.check_address(address)?;
        let addr_128_ptr = address as *const u32;
        // Safety: We have checked the address already
        unsafe {
            Ok([
                core::ptr::read_volatile(addr_128_ptr),
                core::ptr::read_volatile(addr_128_ptr.offset(1)),
                core::ptr::read_volatile(addr_128_ptr.offset(2)),
                core::ptr::read_volatile(addr_128_ptr.offset(3)),
            ])
        }
    }

    /// Reads a [`u32`] from flash memory. Uses little-endian byte order.
    /// The target address must be 32-bit aligned.
    pub fn read_32(&self, address: u32) -> Result<u32, FlashError> {
        // Target address must be 32-bit aligned
        if address & 0b11 != 0 {
            return Err(FlashError::InvalidAddress);
        }
        self.check_address(address)?;
        let addr_32_ptr = address as *const u32;
        // Safety: We have checked the address already
        unsafe { Ok(core::ptr::read_volatile(addr_32_ptr)) }
    }

    /// Erases a page in flash memory.
    ///
    /// # Safety
    /// Care must be taken to not erase the page containing the executing code.
    pub unsafe fn erase_page(&self, address: u32) -> Result<(), FlashError> {
        self._erase_page(address)
    }

    /// Protects a page in flash memory from write or erase operations.
    /// Effective until the next external or power-on reset.
    pub fn disable_page_write(&self, address: u32) -> Result<(), FlashError> {
        while self.is_busy() {}
        let page_num = self.get_page_number(address)?;
        // Lock based on page number
        if page_num < 32 {
            let write_lock_bit = 1 << page_num;
            self.flc
                .welr0()
                .write(|w| unsafe { w.bits(write_lock_bit) });
            while self.flc.welr0().read().bits() & write_lock_bit == write_lock_bit {}
        } else {
            let write_lock_bit = 1 << (page_num - 32);
            self.flc
                .welr1()
                .write(|w| unsafe { w.bits(write_lock_bit) });
            while self.flc.welr1().read().bits() & write_lock_bit == write_lock_bit {}
        }
        Ok(())
    }

    /// Protects a page in flash memory from read operations.
    /// Effective until the next external or power-on reset.
    pub fn disable_page_read(&self, address: u32) -> Result<(), FlashError> {
        while self.is_busy() {}
        let page_num = self.get_page_number(address)?;
        // Lock based on page number
        if page_num < 32 {
            let read_lock_bit = 1 << page_num;
            self.flc.rlr0().write(|w| unsafe { w.bits(read_lock_bit) });
            while self.flc.rlr0().read().bits() & read_lock_bit == read_lock_bit {}
        } else {
            let read_lock_bit = 1 << (page_num - 32);
            self.flc.rlr1().write(|w| unsafe { w.bits(read_lock_bit) });
            while self.flc.rlr1().read().bits() & read_lock_bit == read_lock_bit {}
        }
        Ok(())
    }
}
