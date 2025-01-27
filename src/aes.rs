use max78000_pac::aes::ctrl::KeySize;
use crate::gcr::ResetForPeripheral;

pub const AES_KEYS: usize = 0x4000_7800;

#[repr(u8)]
pub enum CipherType {
    Encrypt = 0b_00,
    Decrypt = 0b_01,
}

pub enum Key<'a> {
    Bits128(&'a [u8; 16]),
    Bits192(&'a [u8; 24]),
    Bits256(&'a [u8; 32]),
}

impl<'a> Key<'a> {
    fn size(&self) -> usize {
        match self {
            Key::Bits128(_) => { 16 }
            Key::Bits192(_) => { 24 }
            Key::Bits256(_) => { 32 }
        }
    }

    fn as_ptr(&self) -> *const u8 {
        match self {
            Key::Bits128(key) => { key.as_ptr() }
            Key::Bits192(key) => { key.as_ptr() }
            Key::Bits256(key) => { key.as_ptr() }
        }
    }
}

impl Into<KeySize> for &Key<'_> {
    fn into(self) -> KeySize {
        match self {
            Key::Bits128(_) => {KeySize::Aes128}
            Key::Bits192(_) => {KeySize::Aes192}
            Key::Bits256(_) => {KeySize::Aes256}
        }
    }
}

pub struct AES {
    aes: crate::pac::Aes
}


impl AES {
    pub fn new(aes: crate::pac::Aes, reg: &mut crate::gcr::GcrRegisters) -> AES {
        use crate::gcr::ClockForPeripheral;
        unsafe {
            aes.reset(&mut reg.gcr);
            aes.enable_clock(&mut reg.gcr);
        }

        aes.inten().write(|reg| reg.done().clear_bit());

        Self { aes }
    }

    fn _is_busy(&self) -> bool {
        self.aes.status().read().busy().bit_is_set()
    }

    pub fn set_key(&mut self, key: &Key) {

        /// Wait for AES module to not be busy
        while self._is_busy() {}

        /// Disable aes module for key change
        self.aes.ctrl().write(|reg| reg.en().clear_bit());

        /// Check if fifos are empty and if they are not flush the current fifos
        if self.aes.status().read().input_em().bit_is_clear() {
            self.aes.ctrl().write(|reg| reg.input_flush().set_bit());
        }
        if self.aes.status().read().output_em().bit_is_clear() {
            self.aes.ctrl().write(|reg| reg.output_flush().set_bit());
        }

        /// Configure Key size
        self.aes.ctrl().write(|reg| reg.key_size().variant(key.into()));


        unsafe {
            for i in 0..256 {
                core::ptr::write_volatile((AES_KEYS + (i * 4)) as *mut u32, 0u32);
            }
            core::ptr::copy_nonoverlapping(key.as_ptr(), AES_KEYS as *mut u8, key.size())
        }

        self.aes.ctrl().write(|reg| reg.en().set_bit());

        // self.add_data_to_fifo(&[0u8; 16]);

    }

    pub fn set_cipher_type(&mut self, cipher_type: CipherType) {
        self.aes.ctrl().write(|reg| {reg.en().clear_bit()});
        self.aes.ctrl().write(|reg| unsafe {reg.type_().bits(cipher_type as u8)});
        self.aes.ctrl().write(|reg| {reg.en().set_bit()});

    }

    // Helper function to process a 16-byte block
    fn add_block_to_fifo(&mut self, block: &[u8; 16]) {
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

    // Main function to add data to the FIFO
    pub fn add_data_to_fifo(&mut self, data: &[u8]) {
        // Wait until FIFO is ready
        while self._is_busy() {}


        // Process full 16-byte chunks
        for chunk in data.chunks_exact(16) {
            self.add_block_to_fifo(chunk.try_into().unwrap());
        }

        // Handle the remainder (less than 16 bytes)
        let remainder = data.chunks_exact(16).remainder();
        if !remainder.is_empty() {
            let mut padded_block = [0u8; 16];
            padded_block[..remainder.len()].copy_from_slice(remainder);

            // Apply PKCS#7 padding
            let padding_size = 16 - remainder.len();
            padded_block[remainder.len()..].fill(padding_size as u8);

            self.add_block_to_fifo(&padded_block);
        }
    }

    fn fetch_block_from_fifo(&mut self) -> [u8; 16] {
        while self._is_busy() {}

        let mut data = [0u8; 16];
        for chunk in data.chunks_exact_mut(4).rev() {
            let word = self.aes.fifo().read().bits();
            let bytes = word.to_be_bytes(); // Convert the 32-bit word to 4 bytes
            chunk.copy_from_slice(&bytes);
        }
        data
    }

    // Main function to fetch data of variable size
    pub fn fetch_data_from_fifo(&mut self, output: &mut [u8]) {
        while self._is_busy() {}


        // Fetch full blocks
        for chunk in output.chunks_exact_mut(16) {
            let block = self.fetch_block_from_fifo();
            chunk.copy_from_slice(&block);
        }

        // Handle remainder
        let remainder = output.chunks_exact_mut(16).into_remainder();
        if !remainder.is_empty() {
            let block = self.fetch_block_from_fifo();
            remainder.copy_from_slice(&block[..remainder.len()]);
        }

    }
}
















