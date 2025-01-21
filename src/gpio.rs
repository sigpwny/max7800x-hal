//! General Purpose Input/Output (GPIO)
use core::marker::PhantomData;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};
use paste::paste;

/// Marker trait for GPIO pin modes.
pub trait PinMode: crate::Sealed {}

pub struct Input;
pub struct InputOutput;
pub struct Af1;
pub struct Af2;

impl crate::Sealed for Input {}
impl crate::Sealed for InputOutput {}
impl crate::Sealed for Af1 {}
impl crate::Sealed for Af2 {}

impl PinMode for Input {}
impl PinMode for InputOutput {}
impl PinMode for Af1 {}
impl PinMode for Af2 {}

/// Marker trait for GPIO pin power supply.
pub trait PowerSupply: crate::Sealed {}

pub struct Vddio;
pub struct Vddioh;

impl crate::Sealed for Vddio {}
impl crate::Sealed for Vddioh {}

impl PowerSupply for Vddio {}
impl PowerSupply for Vddioh {}

/// Marker trait for GPIO pin input pad modes.
pub trait PadMode: crate::Sealed {}

pub struct HighImpedance;
pub struct PullUpWeak;
pub struct PullUpStrong;
pub struct PullDownWeak;
pub struct PullDownStrong;

impl crate::Sealed for HighImpedance {}
impl crate::Sealed for PullUpWeak {}
impl crate::Sealed for PullUpStrong {}
impl crate::Sealed for PullDownWeak {}
impl crate::Sealed for PullDownStrong {}

impl PadMode for HighImpedance {}
impl PadMode for PullUpWeak {}
impl PadMode for PullUpStrong {}
impl PadMode for PullDownWeak {}
impl PadMode for PullDownStrong {}

/// Marker trait for GPIO pin output drive strengths.
pub trait DriveStrength: crate::Sealed {}

pub struct Strength0;
pub struct Strength1;
pub struct Strength2;
pub struct Strength3;

impl crate::Sealed for Strength0 {}
impl crate::Sealed for Strength1 {}
impl crate::Sealed for Strength2 {}
impl crate::Sealed for Strength3 {}

impl DriveStrength for Strength0 {}
impl DriveStrength for Strength1 {}
impl DriveStrength for Strength2 {}
impl DriveStrength for Strength3 {}

/// Zero-sized abstraction type for a GPIO pin.
///
/// Traits from [`embedded_hal::digital`] are also implemented for each pin.
///
/// - `P` is the GPIO port number (e.g. `0` for `Gpio0`, `1` for `Gpio1`, etc.)
/// - `N` is the GPIO pin number.
/// - `MODE` is one of the pin modes (e.g. `Input`, `InputOutput`, `Af1`, `Af2`).
pub struct Pin<
    const P: u8,
    const N: u8,
    MODE: PinMode = Input,
    SUPPLY: PowerSupply = Vddio,
    PAD: PadMode = HighImpedance,
    DRIVE: DriveStrength = Strength0,
> {
    _mode: PhantomData<MODE>,
    _supply: PhantomData<SUPPLY>,
    _pad: PhantomData<PAD>,
    _drive: PhantomData<DRIVE>,
}

/// Default methods that should work across all pin modes.
impl<const P: u8, const N: u8, MODE: PinMode> Pin<P, N, MODE> {
    const fn new() -> Self {
        Self {
            _mode: PhantomData,
            _supply: PhantomData,
            _pad: PhantomData,
            _drive: PhantomData,
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _output_enable(&mut self) {
        // Safety: Concurrent write access to the GPIO output enable atomic set register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.outen_set().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _output_disable(&mut self) {
        // Safety: Concurrent write access to the GPIO output enable atomic clear register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.outen_clr().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _into_af1(&mut self) {
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        // Set EN0 to 1
        gpio.en0_set().write(|w| unsafe { w.bits(1 << N) });
        // Set EN1 to 0
        gpio.en1_clr().write(|w| unsafe { w.bits(1 << N) });
        // Set EN0 to 0
        gpio.en0_clr().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _into_af2(&mut self) {
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        // Set EN0 to 1
        gpio.en0_set().write(|w| unsafe { w.bits(1 << N) });
        // Set EN1 to 1
        gpio.en1_set().write(|w| unsafe { w.bits(1 << N) });
        // Set EN1 to 0
        gpio.en1_clr().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_high(&self) -> bool {
        // Safety: Concurrent read access to the GPIO input register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.in_().read().gpio_in().bits() & (1 << N) != 0
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_low(&self) -> bool {
        // Safety: Concurrent read access to the GPIO input register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.in_().read().gpio_in().bits() & (1 << N) == 0
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _set_high(&mut self) {
        // Safety: Concurrent write access to the GPIO output atomic set register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.out_set().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _set_low(&mut self) {
        // Safety: Concurrent write access to the GPIO output atomic clear register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.out_clr().write(|w| unsafe { w.bits(1 << N) });
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_set_high(&self) -> bool {
        // Safety: Concurrent read access to the GPIO output register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.out().read().bits() & (1 << N) != 0
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_set_low(&self) -> bool {
        // Safety: Concurrent read access to the GPIO output register is safe
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.out().read().bits() & (1 << N) == 0
    }

    /// Returns [`true`] if the pin is high, [`false`] if the pin is low
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        self._is_high()
    }

    /// Returns [`true`] if the pin is low, [`false`] if the pin is high
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

/// Methods for input pins.
impl<const P: u8, const N: u8> Pin<P, N, Input> {
    /// Configures the pin as an input/output pin.
    #[inline(always)]
    pub fn into_input_output(self) -> Pin<P, N, InputOutput> {
        // Enable the output for the pin
        let mut pin = Pin::<P, N, InputOutput>::new();
        pin._output_enable();
        pin
    }

    /// Configures the pin as an alternate function 1 pin.
    #[inline(always)]
    pub fn into_af1(self) -> Pin<P, N, Af1> {
        let mut pin = Pin::<P, N, Af1>::new();
        pin._into_af1();
        pin
    }

    /// Configures the pin as an alternate function 2 pin.
    #[inline(always)]
    pub fn into_af2(self) -> Pin<P, N, Af2> {
        let mut pin = Pin::<P, N, Af2>::new();
        pin._into_af2();
        pin
    }
}

/// Methods for input/output pins.
impl<const P: u8, const N: u8> Pin<P, N, InputOutput> {
    /// Configures the pin as an input pin (disables output).
    #[inline(always)]
    pub fn into_input(self) -> Pin<P, N, Input> {
        // Disable the output for the pin
        let mut pin = Pin::<P, N, Input>::new();
        pin._output_disable();
        pin
    }

    /// Sets the pin high.
    #[inline(always)]
    pub fn set_high(&mut self) {
        self._set_high();
    }

    /// Sets the pin low.
    #[inline(always)]
    pub fn set_low(&mut self) {
        self._set_low();
    }

    /// Returns [`true`] if the pin is set to high, [`false`] if the pin is set to low.
    #[inline(always)]
    pub fn is_set_high(&self) -> bool {
        self._is_set_high()
    }

    /// Returns [`true`] if the pin is set to low, [`false`] if the pin is set to high.
    #[inline(always)]
    pub fn is_set_low(&self) -> bool {
        self._is_set_low()
    }

    /// Sets the pin power supply to VDDIO.
    #[inline(always)]
    pub fn set_power_vddio(&mut self) {
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.vssel().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << N)) });
    }

    /// Sets the pin power supply to VDDIOH.
    #[inline(always)]
    pub fn set_power_vddioh(&mut self) {
        let gpio = unsafe { &*gpiox_ptr::<P>() };
        gpio.vssel().modify(|r, w| unsafe { w.bits(r.bits() | (1 << N)) });
    }
}

/// embedded-hal ErrorType trait
impl<const P: u8, const N: u8, MODE: PinMode> ErrorType for Pin<P, N, MODE> {
    type Error = core::convert::Infallible;
}

/// embedded-hal InputPin trait
impl<const P: u8, const N: u8, MODE: PinMode> InputPin for Pin<P, N, MODE> {
    #[inline(always)]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self._is_high())
    }

    #[inline(always)]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self._is_low())
    }
}

/// embedded-hal OutputPin trait
impl<const P: u8, const N: u8> OutputPin for Pin<P, N, InputOutput> {
    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self._set_high();
        Ok(())
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self._set_low();
        Ok(())
    }
}

/// embedded-hal StatefulOutputPin trait
impl<const P: u8, const N: u8> StatefulOutputPin for Pin<P, N, InputOutput> {
    #[inline(always)]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self._is_set_high())
    }

    #[inline(always)]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self._is_set_low())
    }
}

/// Macro that generates a GPIO module with an interface for splitting GPIO pins.
///
/// - `$MODULE_PAC`: The peripheral access crate (PAC) module for the GPIO (e.g., `Gpio1`).
/// - `$MODULE_HAL`: The name of the module to be generated (e.g., `gpio1`).
/// - `$GCR_TYPE`: The type of the GCR register for the GPIO (e.g., `gcr`).
/// - `$PORT_NUM`: The port number (e.g., `1` for `Gpio1`).
/// - `[$($PIN_NUM:literal),*]`: A list of pin numbers to include (e.g., `[0, 1, 2, 3]`).
macro_rules! gpio {
    ($MODULE_PAC:ident, $MODULE_HAL:ident, $GCR_TYPE:ident, $PORT_NUM:expr, [$($PIN_NUM:literal),*]) => {
        paste!{
            pub mod $MODULE_HAL {
                /// Collection of GPIO pins from a single GPIO port.
                pub struct Parts {
                    $(
                        pub [<p $PORT_NUM _ $PIN_NUM>]: [<P $PORT_NUM _ $PIN_NUM>],
                    )+
                }

                /// # General Purpose Input/Output (GPIO) Peripheral
                ///
                /// This peripheral provides an interface for enabling and
                /// splitting up GPIO pins.
                ///
                /// ## Example
                /// ```
                /// // Initialize a Gcr
                /// let mut gcr = Gcr::new(peripherals.gcr, peripherals.lpgcr);
                /// // Initialize the GPIO0 peripheral
                /// let gpio0 = hal::gpio::Gpio0::new(p.gpio0, &mut gcr.reg);
                /// // Split into pins
                /// let pins0 = gpio0.split();
                /// // Set up pins for UART communication
                /// let rx_pin = pins0.p0_0.into_af1();
                /// let tx_pin = pins0.p0_1.into_af1();
                ///
                /// // Initialize the GPIO2 peripheral
                /// let gpio2 = hal::gpio::Gpio2::new(p.gpio2, &mut gcr.reg);
                /// // Split into pins
                /// let pins2 = gpio2.split();
                /// // Set up pins for LED control/output
                /// let led_red = pins2.p2_0.into_input_output();
                /// let led_green = pins2.p2_1.into_input_output();
                /// let led_blue = pins2.p2_2.into_input_output();
                ///
                /// // Acquired pins can then be passed to other peripherals in
                /// // the HAL or embedded-hal driver crates.
                /// ```
                pub struct GpioPeripheral {
                    _gpio: $crate::pac::$MODULE_PAC,
                }

                impl GpioPeripheral {
                    /// Constructs and initializes a GPIO peripheral.
                    pub fn new(gpio: $crate::pac::$MODULE_PAC, reg: &mut crate::gcr::GcrRegisters) -> Self {
                        // Enable the GPIO peripheral clock
                        use crate::gcr::ClockForPeripheral;
                        unsafe { gpio.enable_clock(&mut reg.$GCR_TYPE); };
                        Self {
                            _gpio: gpio,
                        }
                    }
                    /// Splits the GPIO peripheral into independent pins.
                    pub fn split(self) -> Parts {
                        Parts {
                            $(
                                [<p $PORT_NUM _ $PIN_NUM>]: [<P $PORT_NUM _ $PIN_NUM>]::new(),
                            )+
                        }
                    }
                }

                // #[doc="Common type for "]
                // #[doc=stringify!($GPIOX)]
                // #[doc=" related pins"]
                // pub type $PX_x<MODE> = super::PartiallyErasedPin<$port_id, MODE>;

                // Creates a zero-sized type for each pin
                $(
                    #[doc=stringify!([<P $PORT_NUM _ $PIN_NUM>])]
                    #[doc=" pin"]
                    pub type [<P $PORT_NUM _ $PIN_NUM>] = super::Pin<$PORT_NUM, $PIN_NUM>;
                )+
            }

            // Re-export the peripheral constructor for easy access
            pub use $MODULE_HAL::GpioPeripheral as $MODULE_PAC;
        }
    };
}

gpio!(Gpio0, gpio0, gcr, 0, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]);
gpio!(Gpio1, gpio1, gcr, 1, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
gpio!(Gpio2, gpio2, lpgcr, 2, [0, 1, 2, 3, 4, 5, 6, 7]);


/// Zero runtime cost function to get the address of a GPIO peripheral.
#[inline(always)]
const fn gpiox_ptr<const P: u8>() -> *const crate::pac::gpio0::RegisterBlock {
    match P {
        0 => crate::pac::Gpio0::ptr(),
        1 => crate::pac::Gpio1::ptr(),
        2 => crate::pac::Gpio2::ptr(),
        _ => panic!("Invalid GPIO port number"),
    }
}
