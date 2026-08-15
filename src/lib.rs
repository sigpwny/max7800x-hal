//! # Hardware Abstraction Layer for MAX7800x Microcontrollers
#![no_std]

#[cfg(not(any(feature = "max78000", feature = "max78002")))]
compile_error!("exactly one of the `max78000` or `max78002` features must be enabled");
#[cfg(all(feature = "max78000", feature = "max78002"))]
compile_error!("the `max78000` and `max78002` features are mutually exclusive");

/// Entry point for the runtime application.
pub use cortex_m_rt::entry;
/// Re-export of the Peripheral Access Crate (PAC) for the target microcontroller.
#[cfg(feature = "max78000")]
pub use max78000_pac as pac;
#[cfg(feature = "max78002")]
pub use max78002_pac as pac;
pub use pac::Interrupt;

mod private {
    pub trait Sealed {}
}
use private::Sealed;

pub mod flc;
pub mod gcr;
pub mod gpio;
pub mod icc;
pub mod trng;
pub mod uart;
