use max78000_pac::aes::ctrl::KeySize;
use crate::gcr::ResetForPeripheral;

/// Address of the AES key registers in memory.
pub const AES_KEYS: usize = 0x4000_7800;

/// Enum representing the type of cipher operation.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum CipherType {
    Encrypt = 0b_00, // Encryption mode
    Decrypt = 0b_10, // Decryption mode
}

/// Enum for representing the AES key sizes.
pub enum Key<'a> {
    Bits128(&'a [u8; 16]), // 128-bit key
    Bits192(&'a [u8; 24]), // 192-bit key
    Bits256(&'a [u8; 32]), // 256-bit key
}

impl<'a> Key<'a> {
    /// Returns the size of the key in bytes.
    fn size(&self) -> usize {
        match self {
            Key::Bits128(_) => 16,
            Key::Bits192(_) => 24,
            Key::Bits256(_) => 32,
        }
    }

    /// Returns a pointer to the key data.
    fn as_ptr(&self) -> *const u8 {
        match self {
            Key::Bits128(key) => key.as_ptr(),
            Key::Bits192(key) => key.as_ptr(),
            Key::Bits256(key) => key.as_ptr(),
        }
    }
}

impl Into<KeySize> for &Key<'_> {
    /// Converts a `Key` into the corresponding `KeySize` enum variant.
    fn into(self) -> KeySize {
        match self {
            Key::Bits128(_) => KeySize::Aes128,
            Key::Bits192(_) => KeySize::Aes192,
            Key::Bits256(_) => KeySize::Aes256,
        }
    }
}

/// AES struct for handling encryption and decryption using the AES hardware module.
pub struct AES {
    aes: crate::pac::Aes,
}

impl AES {
    /// Creates a new AES instance and initializes the hardware module.
    pub fn new(aes: crate::pac::Aes, reg: &mut crate::gcr::GcrRegisters) -> AES {
        use crate::gcr::ClockForPeripheral;

        // Reset and enable the AES hardware module.
        unsafe {
            aes.reset(&mut reg.gcr);
            aes.enable_clock(&mut reg.gcr);
        }

        // Disable AES interrupt notifications.
        aes.inten().write(|reg| reg.done().clear_bit());

        Self { aes }
    }

    /// Checks if the AES hardware module is currently busy.
    fn _is_busy(&self) -> bool {
        self.aes.status().read().busy().bit_is_set()
    }

    /// Configures the AES hardware with the provided key.
    pub fn set_key(&mut self, key: &Key) {
        // Wait until the AES module is not busy.
        while self._is_busy() {}

        // Disable the AES module to configure the key.
        self.aes.ctrl().write(|reg| reg.en().clear_bit());

        // Flush input and output FIFOs if not empty.
        if self.aes.status().read().input_em().bit_is_clear() {
            self.aes.ctrl().write(|reg| reg.input_flush().set_bit());
        }
        if self.aes.status().read().output_em().bit_is_clear() {
            self.aes.ctrl().write(|reg| reg.output_flush().set_bit());
        }

        // Configure the key size.
        self.aes.ctrl().write(|reg| reg.key_size().variant(key.into()));

        // Write the key into the AES key registers.
        unsafe {
            for i in 0..256 {
                core::ptr::write_volatile((AES_KEYS + (i * 4)) as *mut u32, 0u32);
            }
            core::ptr::copy_nonoverlapping(key.as_ptr(), AES_KEYS as *mut u8, key.size());
        }

        // Enable the AES module.
        self.aes.ctrl().write(|reg| reg.en().set_bit());
    }

    /// Configures the AES hardware for encryption or decryption.
    pub fn set_cipher_type(&mut self, cipher_type: CipherType) {
        // Wait until the AES module is not busy.
        while self._is_busy() {}

        // Disable the AES module to change the mode.
        self.aes.ctrl().write(|reg| reg.en().clear_bit());

        // Configure the cipher type (encrypt or decrypt).
        self.aes.ctrl().modify(|read, write| unsafe {
            let mut data = read.bits();
            data |= (cipher_type as u32) << 8 | 1;
            write.bits(data)
        });
    }

    /// Helper function to write a single 16-byte block to the input FIFO.
    ///
    /// # Warning:
    /// This function assumes the block size is exactly 16 bytes.
    fn write_block_to_fifo(&mut self, block: &[u8; 16]) {
        let words = [
            u32::from_be_bytes(block[12..16].try_into().unwrap()),
            u32::from_be_bytes(block[8..12].try_into().unwrap()),
            u32::from_be_bytes(block[4..8].try_into().unwrap()),
            u32::from_be_bytes(block[0..4].try_into().unwrap()),
        ];

        for &word in &words {
            self.aes.fifo().write(|reg| unsafe { reg.bits(word) });
        }
    }

    /// Writes data to the AES hardware input FIFO in 16-byte chunks.
    ///
    /// # Warning:
    /// Any remainder bytes (less than 16) at the end of the input will be ignored.
    pub fn write_data_to_fifo(&mut self, data: &[u8]) {
        // Wait until the FIFO is ready.
        while self._is_busy() {}

        // Process full 16-byte chunks.
        for chunk in data.chunks_exact(16) {
            self.write_block_to_fifo(chunk.try_into().unwrap());
        }

        // Warning: Any data not divisible by 16 bytes is ignored here.
    }

    /// Reads a single 16-byte block from the output FIFO.
    fn read_block_from_fifo(&mut self) -> [u8; 16] {
        while self._is_busy() {}

        let mut data = [0u8; 16];
        for chunk in data.chunks_exact_mut(4).rev() {
            let word = self.aes.fifo().read().bits();
            let bytes = word.to_be_bytes(); // Convert the 32-bit word to 4 bytes
            chunk.copy_from_slice(&bytes);
        }
        data
    }

    /// Reads multiple 16-byte blocks from the AES output FIFO into the provided buffer.
    ///
    /// # Warning:
    /// This function assumes the output buffer is a multiple of 16 bytes.
    pub fn read_data_from_fifo(&mut self, output: &mut [u8]) {
        while self._is_busy() {}

        // Process full 16-byte chunks.
        for chunk in output.chunks_exact_mut(16) {
            let block = self.read_block_from_fifo();
            chunk.copy_from_slice(&block);
        }

        // Warning: If the output buffer size is not a multiple of 16 bytes, the extra space will remain unchanged.
    }
}
