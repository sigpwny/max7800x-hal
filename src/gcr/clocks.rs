//! # Clock and Oscillator Configuration
//!
//! This module provides a full [typestate](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)
//! API for enabling oscillators, configuring the system clock, and calculating
//! clock frequencies. By using typestates, calculation of clock frequencies
//! are done entirely at compile time, with no runtime or memory overhead.

use core::marker::PhantomData;

pub enum OscillatorSourceEnum {
    /// Internal Primary Oscillator (100 MHz)
    Ipo,
    /// Internal Secondary Oscillator (60 MHz)
    Iso,
    // Inro,
    /// Internal Baud Rate Oscillator (7.3728 MHz)
    Ibro,
    /// External RTC Oscillator (32.768 kHz)
    ///
    /// Requires initialization of the RTC peripheral. Currently unsupported.
    Ertco,
}

/// Marker trait for an oscillator source.
pub trait OscillatorSource: crate::Sealed {
    const SOURCE: OscillatorSourceEnum;
    const BASE_FREQUENCY: u32;
}

pub struct InternalPrimaryOscillator;
pub struct InternalSecondaryOscillator;
// pub struct InternalNanoRingOscillator;
pub struct InternalBaudRateOscillator;
pub struct ExternalRtcOscillator;
// pub struct ExternalClockOscillator;

impl crate::Sealed for InternalPrimaryOscillator {}
impl crate::Sealed for InternalSecondaryOscillator {}
impl crate::Sealed for InternalBaudRateOscillator {}
impl crate::Sealed for ExternalRtcOscillator {}

impl OscillatorSource for InternalPrimaryOscillator {
    const SOURCE: OscillatorSourceEnum = OscillatorSourceEnum::Ipo;
    const BASE_FREQUENCY: u32 = 100_000_000; // 100 MHz
}
impl OscillatorSource for InternalSecondaryOscillator {
    const SOURCE: OscillatorSourceEnum = OscillatorSourceEnum::Iso;
    const BASE_FREQUENCY: u32 = 60_000_000; // 60 MHz
}
impl OscillatorSource for InternalBaudRateOscillator {
    const SOURCE: OscillatorSourceEnum = OscillatorSourceEnum::Ibro;
    const BASE_FREQUENCY: u32 = 7_372_800; // 7.3728 MHz
}
impl OscillatorSource for ExternalRtcOscillator {
    const SOURCE: OscillatorSourceEnum = OscillatorSourceEnum::Ertco;
    const BASE_FREQUENCY: u32 = 32_768; // 32.768 kHz
}

/// Marker trait for the state of an oscillator.
pub trait OscillatorState: crate::Sealed {}

pub struct Disabled;
pub struct Enabled;

impl crate::Sealed for Disabled {}
impl crate::Sealed for Enabled {}

impl OscillatorState for Disabled {}
impl OscillatorState for Enabled {}

/// Marker trait for a clock option (enabled oscillator or clock).
pub trait ClockOption: crate::Sealed {}

pub struct SystemClock;
pub struct PeripheralClock;

impl crate::Sealed for SystemClock {}
impl crate::Sealed for PeripheralClock {}
impl ClockOption for SystemClock {}
impl ClockOption for PeripheralClock {}

impl ClockOption for InternalPrimaryOscillator {}
impl ClockOption for InternalSecondaryOscillator {}
impl ClockOption for InternalBaudRateOscillator {}
impl ClockOption for ExternalRtcOscillator {}

/// Marker trait for the system clock divider
pub trait SystemClockDivider: crate::Sealed {
    const DIVISOR: u32;
}

pub struct DivUnknown;
pub struct Div1;
pub struct Div2;
pub struct Div4;
pub struct Div8;
pub struct Div16;
pub struct Div32;
pub struct Div64;
pub struct Div128;

impl crate::Sealed for DivUnknown {}
impl crate::Sealed for Div1 {}
impl crate::Sealed for Div2 {}
impl crate::Sealed for Div4 {}
impl crate::Sealed for Div8 {}
impl crate::Sealed for Div16 {}
impl crate::Sealed for Div32 {}
impl crate::Sealed for Div64 {}
impl crate::Sealed for Div128 {}

impl SystemClockDivider for DivUnknown {
    const DIVISOR: u32 = 1;
}
impl SystemClockDivider for Div1 {
    const DIVISOR: u32 = 1;
}
impl SystemClockDivider for Div2 {
    const DIVISOR: u32 = 2;
}
impl SystemClockDivider for Div4 {
    const DIVISOR: u32 = 4;
}
impl SystemClockDivider for Div8 {
    const DIVISOR: u32 = 8;
}
impl SystemClockDivider for Div16 {
    const DIVISOR: u32 = 16;
}
impl SystemClockDivider for Div32 {
    const DIVISOR: u32 = 32;
}
impl SystemClockDivider for Div64 {
    const DIVISOR: u32 = 64;
}
impl SystemClockDivider for Div128 {
    const DIVISOR: u32 = 128;
}

/// Oscillators represent the state of a physical oscillator. To use an
/// oscillator, it must be enabled. Then, it can be converted into a clock.
pub struct Oscillator<O: OscillatorSource, S: OscillatorState> {
    _source: PhantomData<O>,
    _state: PhantomData<S>,
}

/// Clocks are used to drive peripherals after the system clock is configured.
pub struct Clock<SRC: ClockOption> {
    _src: PhantomData<SRC>,
    pub frequency: u32,
}

/// An OscillatorGuard protects the initialization of an [`Oscillator`],
/// ensuring that each oscillator source is only initialized once.
pub struct OscillatorGuard<O: OscillatorSource> {
    _source: PhantomData<O>,
}

impl<O> OscillatorGuard<O>
where
    O: OscillatorSource,
{
    pub(super) fn new() -> Self {
        Self {
            _source: PhantomData,
        }
    }
}

/// A collection of OscillatorGuards for each [`Oscillator`] source.
pub struct OscillatorGuards {
    pub ipo: OscillatorGuard<InternalPrimaryOscillator>,
    pub iso: OscillatorGuard<InternalSecondaryOscillator>,
    pub ibro: OscillatorGuard<InternalBaudRateOscillator>,
    pub ertco: OscillatorGuard<ExternalRtcOscillator>,
}

impl OscillatorGuards {
    pub(super) fn new() -> Self {
        Self {
            ipo: OscillatorGuard::new(),
            iso: OscillatorGuard::new(),
            ibro: OscillatorGuard::new(),
            ertco: OscillatorGuard::new(),
        }
    }
}

/// Initialization of an [`Oscillator`] requires consumption of a
/// corresponding typed OscillatorGuard.
impl<O> Oscillator<O, Disabled>
where
    O: OscillatorSource,
{
    pub fn new(_guard: OscillatorGuard<O>) -> Self {
        Self {
            _source: PhantomData,
            _state: PhantomData,
        }
    }
}

pub type Ipo = Oscillator<InternalPrimaryOscillator, Disabled>;
impl Ipo {
    pub fn enable(
        &self,
        reg: &mut super::GcrRegisters,
    ) -> Oscillator<InternalPrimaryOscillator, Enabled> {
        reg.gcr.clkctrl().modify(|_, w| w.ipo_en().set_bit());
        while reg.gcr.clkctrl().read().ipo_rdy().bit_is_clear() {}
        Oscillator {
            _source: PhantomData,
            _state: PhantomData,
        }
    }
}
impl Oscillator<InternalPrimaryOscillator, Enabled> {
    pub const fn into_clock(self) -> Clock<InternalPrimaryOscillator> {
        Clock::<InternalPrimaryOscillator> {
            _src: PhantomData,
            frequency: InternalPrimaryOscillator::BASE_FREQUENCY,
        }
    }
}

pub type Iso = Oscillator<InternalSecondaryOscillator, Disabled>;
impl Iso {
    pub fn enable(
        self,
        reg: &mut super::GcrRegisters,
    ) -> Oscillator<InternalSecondaryOscillator, Enabled> {
        reg.gcr.clkctrl().modify(|_, w| w.iso_en().set_bit());
        while reg.gcr.clkctrl().read().iso_rdy().bit_is_clear() {}
        Oscillator {
            _source: PhantomData,
            _state: PhantomData,
        }
    }
}
impl Oscillator<InternalSecondaryOscillator, Enabled> {
    pub const fn into_clock(self) -> Clock<InternalSecondaryOscillator> {
        Clock::<InternalSecondaryOscillator> {
            _src: PhantomData,
            frequency: InternalSecondaryOscillator::BASE_FREQUENCY,
        }
    }
}

// pub type Inro = Oscillator<InternalNanoRingOscillator, Disabled>;
// impl Inro {
//     pub fn enable(self, reg: &mut super::GcrRegisters) -> Oscillator<InternalNanoRingOscillator, Enabled> {
//         // INRO is always enabled
//         while reg.gcr.clkctrl().read().inro_rdy().bit_is_clear() {}
//         Oscillator {
//             _source: PhantomData,
//             _state: PhantomData,
//         }
//     }
// }

pub type Ibro = Oscillator<InternalBaudRateOscillator, Disabled>;
impl Ibro {
    pub fn enable(
        self,
        reg: &mut super::GcrRegisters,
    ) -> Oscillator<InternalBaudRateOscillator, Enabled> {
        // IBRO is always enabled
        while reg.gcr.clkctrl().read().ibro_rdy().bit_is_clear() {}
        Oscillator {
            _source: PhantomData,
            _state: PhantomData,
        }
    }
}
impl Oscillator<InternalBaudRateOscillator, Enabled> {
    pub const fn into_clock(self) -> Clock<InternalBaudRateOscillator> {
        Clock::<InternalBaudRateOscillator> {
            _src: PhantomData,
            frequency: InternalBaudRateOscillator::BASE_FREQUENCY,
        }
    }
}

pub type Ertco = Oscillator<ExternalRtcOscillator, Disabled>;
impl Oscillator<ExternalRtcOscillator, Disabled> {
    pub fn enable(
        self,
        reg: &mut super::GcrRegisters,
    ) -> Oscillator<ExternalRtcOscillator, Enabled> {
        reg.gcr.clkctrl().modify(|_, w| w.ertco_en().set_bit());
        while reg.gcr.clkctrl().read().ertco_rdy().bit_is_clear() {}
        todo!("ERTCO requires initialization of the RTC peripheral");
        // Oscillator {
        //     _source: PhantomData,
        //     _state: PhantomData,
        // }
    }
}

/// System clock setup configuration (source and divider).
pub struct SystemClockConfig<S: OscillatorSource, D: SystemClockDivider> {
    _source: PhantomData<S>,
    _divider: PhantomData<D>,
}

/// Initialized system clock configuration and resulting [`Clock`]s and frequencies.
pub struct SystemClockResults {
    pub sys_clk: Clock<SystemClock>,
    pub pclk: Clock<PeripheralClock>,
}

impl<S, D> SystemClockConfig<S, D>
where
    S: OscillatorSource,
    D: SystemClockDivider,
{
    pub fn new() -> Self {
        SystemClockConfig {
            _source: PhantomData,
            _divider: PhantomData,
        }
    }

    /// Set the source oscillator of the system clock (SYS_CLK).
    /// The oscillator must be enabled beforehand (enforced by the type system).
    pub fn set_source<NewS: OscillatorSource>(
        self,
        reg: &mut super::GcrRegisters,
        _oscillator: &Oscillator<NewS, Enabled>,
    ) -> SystemClockConfig<NewS, D> {
        match NewS::SOURCE {
            OscillatorSourceEnum::Ipo => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_sel().ipo());
            }
            OscillatorSourceEnum::Iso => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_sel().iso());
            }
            OscillatorSourceEnum::Ibro => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_sel().ibro());
            }
            OscillatorSourceEnum::Ertco => {
                // reg.gcr.clkctrl().modify(|_, w| w.sysclk_sel().ertco());
                todo!("ERTCO requires initialization of the RTC peripheral");
            }
        }
        while reg.gcr.clkctrl().read().sysclk_rdy().bit_is_clear() {}
        SystemClockConfig {
            _source: PhantomData,
            _divider: PhantomData,
        }
    }

    /// Set the divider of the system clock (SYS_CLK).
    /// The divider must be a valid value (enforced by the type system).
    pub fn set_divider<NewD: SystemClockDivider>(
        self,
        reg: &mut super::GcrRegisters,
    ) -> SystemClockConfig<S, NewD> {
        match NewD::DIVISOR {
            1 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div1());
            }
            2 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div2());
            }
            4 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div4());
            }
            8 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div8());
            }
            16 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div16());
            }
            32 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div32());
            }
            64 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div64());
            }
            128 => {
                reg.gcr.clkctrl().modify(|_, w| w.sysclk_div().div128());
            }
            _ => {
                unreachable!("Invalid system clock divider");
            }
        }
        while reg.gcr.clkctrl().read().sysclk_rdy().bit_is_clear() {}
        SystemClockConfig {
            _source: PhantomData,
            _divider: PhantomData,
        }
    }

    /// Freeze the system clock configuration and return configured clocks.
    pub const fn freeze(self) -> SystemClockResults {
        SystemClockResults {
            sys_clk: Clock::<SystemClock> {
                _src: PhantomData,
                frequency: S::BASE_FREQUENCY / D::DIVISOR,
            },
            pclk: Clock::<PeripheralClock> {
                _src: PhantomData,
                frequency: (S::BASE_FREQUENCY / D::DIVISOR) / 2,
            },
        }
    }
}
