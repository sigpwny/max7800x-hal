use crate::aes::CipherType::{Decrypt, Encrypt};
use crate::gcr::ResetForPeripheral;
use cipher::consts::{U16, U24, U32};
use cipher::{
    Block, BlockCipherDecBackend, BlockCipherDecClosure, BlockCipherDecrypt, BlockCipherEncBackend,
    BlockCipherEncClosure, BlockCipherEncrypt, BlockSizeUser, InOut, KeySizeUser,
    ParBlocksSizeUser,
};
use max78000_pac::aes::ctrl::KeySize;

/// Address of the AES key registers in memory.
pub const AES_KEYS: usize = 0x4000_7800;

/// Enum representing the type of cipher operation.
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
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
pub struct AesBackend<const KEY_SIZE: usize> {
    aes: crate::pac::Aes,
}

/// AES-128 Implementation
pub type Aes128Hardware = AesBackend<16>;
impl KeySizeUser for Aes128Hardware {
    type KeySize = U16;
}

/// AES-192 Implementation
pub type Aes192Hardware = AesBackend<24>;
impl KeySizeUser for Aes192Hardware {
    type KeySize = U24;
}

/// AES-256 Implementation
pub type Aes256Hardware = AesBackend<32>;
impl KeySizeUser for Aes256Hardware {
    type KeySize = U32;
}

impl<const N: usize> AesBackend<N>
where
    Self: KeySizeUser,
{
    /// Creates a new instance with a key.
    pub fn new_with_key(
        aes: crate::pac::Aes,
        reg: &mut crate::gcr::GcrRegisters,
        key: cipher::Key<Self>, // GenericArray<u8, Self::KeySize>
    ) -> Self {
        let mut instance = Self::new_backend(aes, reg);

        // Convert the GenericArray to a slice and take the first 16 bytes.
        let key_bytes: [u8; 16] = key.as_slice()[..16]
            .try_into()
            .expect("Slice with 16 bytes");
        instance.set_key(&Key::Bits128(&key_bytes));

        instance
    }
}

impl<const KEY_SIZE: usize> AesBackend<KEY_SIZE> {
    /// Creates a new AES instance and initializes the hardware module.
    fn new_backend(aes: crate::pac::Aes, reg: &mut crate::gcr::GcrRegisters) -> Self {
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
    fn set_key(&mut self, key: &Key) {
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
        self.aes
            .ctrl()
            .write(|reg| reg.key_size().variant(key.into()));

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
    fn set_cipher_type(&self, cipher_type: CipherType) {
        if self.aes.ctrl().read().type_() == cipher_type as u8 {
            return;
        }

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
    fn write_block_to_fifo(&self, block: &[u8; 16]) {
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

    /// Reads a single 16-byte block from the output FIFO.
    fn read_block_from_fifo(&self) -> [u8; 16] {
        while self._is_busy() {}

        let mut data = [0u8; 16];
        for chunk in data.chunks_exact_mut(4).rev() {
            let word = self.aes.fifo().read().bits();
            let bytes = word.to_be_bytes(); // Convert the 32-bit word to 4 bytes
            chunk.copy_from_slice(&bytes);
        }
        data
    }
}

impl<const KEY_SIZE: usize> ParBlocksSizeUser for AesBackend<KEY_SIZE> {
    type ParBlocksSize = U16;
}

impl<const KEY_SIZE: usize> BlockSizeUser for AesBackend<KEY_SIZE> {
    type BlockSize = U16; // AES block size is 16 bytes (128 bits)
}

impl<const KEY_SIZE: usize> BlockCipherEncBackend for AesBackend<KEY_SIZE> {
    fn encrypt_block(&self, mut block: InOut<'_, '_, Block<Self>>) {
        // If AES is an in-place transform, read from `get()`
        let mut data = [0u8; 16];

        self.set_cipher_type(Encrypt);

        let input_block = block.get_in();
        data.copy_from_slice(input_block);
        self.write_block_to_fifo(&data);

        // Then modify the same buffer with `get_mut()`
        let output_block = block.get_out();
        output_block.copy_from_slice(&self.read_block_from_fifo());
    }
}

impl<const KEY_SIZE: usize> BlockCipherDecBackend for AesBackend<KEY_SIZE> {
    fn decrypt_block(&self, mut block: InOut<'_, '_, Block<Self>>) {
        // If AES is an in-place transform, read from `get()`
        let mut data = [0u8; 16];

        self.set_cipher_type(Decrypt);

        let input_block = block.get_in();
        data.copy_from_slice(input_block);
        self.write_block_to_fifo(&data);

        // Then modify the same buffer with `get_mut()`
        let output_block = block.get_out();
        output_block.copy_from_slice(&data);
    }
}

impl<const KEY_SIZE: usize> BlockCipherEncrypt for AesBackend<KEY_SIZE> {
    fn encrypt_with_backend(&self, f: impl BlockCipherEncClosure<BlockSize = Self::BlockSize>) {
        f.call(self);
    }
}

impl<const KEY_SIZE: usize> BlockCipherDecrypt for AesBackend<KEY_SIZE> {
    fn decrypt_with_backend(&self, f: impl BlockCipherDecClosure<BlockSize = Self::BlockSize>) {
        f.call(self)
    }
}
