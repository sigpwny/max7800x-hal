//! # Global Control Registers (GCR)
//! Handles global control registers, such as resets, system clocks,
//! peripheral clocks, etc.
//!
//! Initialization of the [`Gcr`] peripheral is required to constrain the
//! GCR from the PAC and safely use them within the HAL.

pub mod clocks;

/// Wrapper struct to constrain the GCR.
pub struct GcrRegisters {
    pub(crate) gcr: crate::pac::Gcr,
    pub(crate) lpgcr: crate::pac::Lpgcr,
}

/// Global Control Registers (GCR) Peripheral
pub struct Gcr {
    pub reg: GcrRegisters,
    pub osc_guards: clocks::OscillatorGuards,
    pub sys_clk: clocks::SystemClockConfig<clocks::InternalSecondaryOscillator, clocks::DivUnknown>,
}

impl Gcr {
    pub fn new(gcr: crate::pac::Gcr, lpgcr: crate::pac::Lpgcr) -> Self {
        Gcr {
            reg: GcrRegisters { gcr, lpgcr },
            osc_guards: clocks::OscillatorGuards::new(),
            sys_clk: clocks::SystemClockConfig::new(),
        }
    }
}

#[doc(hidden)]
pub trait GcrRegisterType {}
impl GcrRegisterType for crate::pac::Gcr {}
impl GcrRegisterType for crate::pac::Lpgcr {}

/// Extension trait for enabling and disabling peripheral clocks.
pub trait ClockForPeripheral {
    type ValidatedGcrRegisterType: GcrRegisterType;
    unsafe fn enable_clock(&self, gcr: &mut Self::ValidatedGcrRegisterType);
    unsafe fn disable_clock(&self, gcr: &mut Self::ValidatedGcrRegisterType);
}

// Extension trait for peripheral resets.
pub trait ResetForPeripheral {
    type ValidatedGcrRegisterType: GcrRegisterType;
    unsafe fn reset(&self, resets: &mut Self::ValidatedGcrRegisterType);
}

macro_rules! generate_clock {
    ($MODULE:ident, $GCR_TYPE:ident, $PCLKDISN:ident, $PCLK_FIELD:ident) => {
        impl ClockForPeripheral for $crate::pac::$MODULE {
            type ValidatedGcrRegisterType = $crate::pac::$GCR_TYPE;

            /// Enables the peripheral clock.
            ///
            /// ## Safety
            /// It is recommended that this function is only called
            /// through the constructor of a HAL peripheral, rather than
            /// directly by the user. If called directly, the user should
            /// ensure that the peripheral clock is not already enabled.
            unsafe fn enable_clock(&self, gcr: &mut Self::ValidatedGcrRegisterType) {
                gcr.$PCLKDISN().modify(|_, w| w.$PCLK_FIELD().clear_bit());
                while gcr.$PCLKDISN().read().$PCLK_FIELD().bit_is_set() {}
            }

            /// Disables the peripheral clock.
            ///
            /// ## Safety
            /// It is recommended that this function is only called
            /// through the constructor of a HAL peripheral, rather than
            /// directly by the user. If called directly, the user should
            /// ensure that the peripheral clock is not already disabled.
            unsafe fn disable_clock(&self, gcr: &mut Self::ValidatedGcrRegisterType) {
                gcr.$PCLKDISN().modify(|_, w| w.$PCLK_FIELD().set_bit());
                while gcr.$PCLKDISN().read().$PCLK_FIELD().bit_is_clear() {}
            }
        }
    };
}

macro_rules! generate_reset {
    ($MODULE:ident, $GCR_TYPE:ident, $RST_REG:ident, $RST_REG_FIELD:ident) => {
        impl ResetForPeripheral for $crate::pac::$MODULE {
            type ValidatedGcrRegisterType = $crate::pac::$GCR_TYPE;

            /// Resets the peripheral.
            ///
            /// ## Safety
            /// User should ensure that the peripheral is not in use when
            /// initiating a reset of a peripheral.
            unsafe fn reset(&self, gcr: &mut Self::ValidatedGcrRegisterType) {
                gcr.$RST_REG().modify(|_, w| w.$RST_REG_FIELD().set_bit());
                while gcr.$RST_REG().read().$RST_REG_FIELD().bit_is_set() {}
            }
        }
    };
}

generate_clock!(Adc, Gcr, pclkdis0, adc);
generate_clock!(Aes, Gcr, pclkdis1, aes);
// CNN?
// CPU1 (RISC-V core)?
generate_clock!(Crc, Gcr, pclkdis1, crc);
generate_clock!(Dma, Gcr, pclkdis0, dma);
generate_clock!(Gpio0, Gcr, pclkdis0, gpio0);
generate_clock!(Gpio1, Gcr, pclkdis0, gpio1);
generate_clock!(Gpio2, Lpgcr, pclkdis, gpio2);
generate_clock!(I2c0, Gcr, pclkdis0, i2c0);
generate_clock!(I2c1, Gcr, pclkdis0, i2c1);
generate_clock!(I2c2, Gcr, pclkdis1, i2c2);
generate_clock!(I2s, Gcr, pclkdis1, i2s);
generate_clock!(Lpcmp, Lpgcr, pclkdis, lpcomp);
generate_clock!(Owm, Gcr, pclkdis1, owm);
generate_clock!(Pt0, Gcr, pclkdis0, pt);
generate_clock!(Sema, Gcr, pclkdis1, smphr);
generate_clock!(Spi0, Gcr, pclkdis1, spi0);
generate_clock!(Spi1, Gcr, pclkdis0, spi1);
generate_clock!(Tmr0, Gcr, pclkdis0, tmr0);
generate_clock!(Tmr1, Gcr, pclkdis0, tmr1);
generate_clock!(Tmr2, Gcr, pclkdis0, tmr2);
generate_clock!(Tmr3, Gcr, pclkdis0, tmr3);
generate_clock!(Tmr4, Lpgcr, pclkdis, tmr4);
generate_clock!(Tmr5, Lpgcr, pclkdis, tmr5);
generate_clock!(Trng, Gcr, pclkdis1, trng);
generate_clock!(Uart0, Gcr, pclkdis0, uart0);
generate_clock!(Uart1, Gcr, pclkdis0, uart1);
generate_clock!(Uart2, Gcr, pclkdis1, uart2);
generate_clock!(Uart3, Lpgcr, pclkdis, uart3);
generate_clock!(Wdt0, Gcr, pclkdis1, wdt0);
generate_clock!(Wdt1, Lpgcr, pclkdis, wdt1);

// TODO: add system, peripheral, and soft resets
generate_reset!(Adc, Gcr, rst0, adc);
generate_reset!(Aes, Gcr, rst1, aes);
// CNN?
// CPU1 (RISC-V core)?
generate_reset!(Crc, Gcr, rst1, crc);
generate_reset!(Dma, Gcr, rst0, dma);
generate_reset!(Dvs, Gcr, rst1, dvs); // Note: Dynamic Voltage Scaling Controller does not have its own peripheral clock
generate_reset!(Gpio0, Gcr, rst0, gpio0); // Note: Peripheral resets may not affect the GPIO peripherals
generate_reset!(Gpio1, Gcr, rst0, gpio1); // Note: Peripheral resets may not affect the GPIO peripherals
generate_reset!(Gpio2, Lpgcr, rst, gpio2); // Note: Peripheral resets may not affect the GPIO peripherals
generate_reset!(I2c0, Gcr, rst0, i2c0);
generate_reset!(I2c1, Gcr, rst1, i2c1);
generate_reset!(I2c2, Gcr, rst1, i2c2);
generate_reset!(I2s, Gcr, rst1, i2s);
generate_reset!(Lpcmp, Lpgcr, rst, lpcomp);
generate_reset!(Owm, Gcr, rst1, owm);
generate_reset!(Pt0, Gcr, rst1, pt);
generate_reset!(Rtc, Gcr, rst0, rtc);
generate_reset!(Sema, Gcr, rst1, smphr);
generate_reset!(Simo, Gcr, rst1, simo);
generate_reset!(Spi0, Gcr, rst1, spi0);
generate_reset!(Spi1, Gcr, rst0, spi1);
generate_reset!(Tmr0, Gcr, rst0, tmr0);
generate_reset!(Tmr1, Gcr, rst0, tmr1);
generate_reset!(Tmr2, Gcr, rst0, tmr2);
generate_reset!(Tmr3, Gcr, rst0, tmr3);
generate_reset!(Tmr4, Lpgcr, rst, tmr4);
generate_reset!(Tmr5, Lpgcr, rst, tmr5);
generate_reset!(Trng, Gcr, rst0, trng);
generate_reset!(Uart0, Gcr, rst0, uart0);
generate_reset!(Uart1, Gcr, rst0, uart1);
generate_reset!(Uart2, Gcr, rst0, uart2);
generate_reset!(Uart3, Lpgcr, rst, uart3);
generate_reset!(Wdt0, Gcr, rst0, wdt0);
generate_reset!(Wdt1, Lpgcr, rst, wdt1);
