//! # True Random Number Generator (TRNG)
//!
//! The TRNG is a hardware module that generates random numbers using
//! physical entropy sources.

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
    pub fn next_u32(&self) -> u32 {
        while !self._is_ready() {}
        self.trng.data().read().bits() as u32
    }

    /// Generate a random 8-bit number.
    #[inline(always)]
    pub fn next_u8(&self) -> u8 {
        self.next_u32() as u8
    }

    /// Fill a buffer with random bytes.
    #[inline(always)]
    pub fn fill_bytes(&self, buffer: &mut [u8]) {
        for word in buffer.chunks_mut(size_of::<u32>()) {
            let random = self.next_u32();
            word.copy_from_slice(&random.to_le_bytes()[..word.len()]);
        }
    }
}