//! # Hardware Abstraction Layer for MAX7800x Microcontrollers
#![no_std]

/// Re-export of the Peripheral Access Crate (PAC) for the MAX78000.
pub use max78000_pac as pac;
pub use pac::Interrupt;
/// Entry point for the runtime application.
pub use cortex_m_rt::entry;

mod private {
    pub trait Sealed {}
}
use private::Sealed;

pub mod gcr;
pub mod gpio;
pub mod trng;
pub mod uart;