//! # True Random Number Generator (TRNG)
//!
//! The TRNG is a hardware module that generates random numbers using
//! physical entropy sources.
use rand::Error;
#[cfg(feature = "rand")]
use rand::RngCore;
#[cfg(feature = "rand")]
use rand_core::impls::{fill_bytes_via_next, next_u64_via_u32};

/// # True Random Number Generator (TRNG) Peripheral
///
/// Example:
/// ```
/// // Create a new TRNG peripheral instance
/// let trng = Trng::new(p.trng, &mut gcr.reg);
/// // Generate a random 32-bit number
/// let random_number = trng.next_u32();
/// // Fill an array with random bytes
/// let mut buffer = [0u8; 64];
/// trng.fill_bytes(&mut buffer);
/// ```
pub struct Trng {
    trng: crate::pac::Trng,
}

impl Trng {
    /// Create a new TRNG peripheral instance.
    pub fn new(trng: crate::pac::Trng, reg: &mut crate::gcr::GcrRegisters) -> Self {
        use crate::gcr::ClockForPeripheral;
        unsafe { trng.enable_clock(&mut reg.gcr); }
        Self { trng }
    }

    /// Check if the TRNG peripheral is ready to generate random numbers.
    #[doc(hidden)]
    #[inline(always)]
    fn _is_ready(&self) -> bool {
        self.trng.status().read().rdy().is_ready()
    }

    /// Generate a random 32-bit number.
    #[inline(always)]
    pub fn gen_u32(&self) -> u32 {
        while !self._is_ready() {}
        self.trng.data().read().bits() as u32
    }
}

#[cfg(feature = "rand")]
impl RngCore for Trng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.gen_u32()
    }

        
    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        next_u64_via_u32(self)
    }


    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}