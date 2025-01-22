//! # Universal Asynchronous Receiver/Transmitter (UART)
use core::marker::PhantomData;
use core::ops::Deref;

use crate::gcr::{
    ClockForPeripheral,
    clocks::{
        Clock,
        InternalBaudRateOscillator,
        PeripheralClock,
    }
};
use crate::gpio::{Pin, Af1};
use embedded_hal_nb::{serial, nb};
use paste::paste;

enum UartClockSource {
    Pclk,
    Ibro,
}

/// Number of data bits in a UART frame.
pub enum DataBits {
    /// 5 data bits.
    Five,
    /// 6 data bits.
    Six,
    /// 7 data bits.
    Seven,
    /// 8 data bits.
    Eight,
}

/// Number of stop bits in a UART frame.
pub enum StopBits {
    /// 1 stop bit.
    One,
    /// 1.5 stop bits when using 5 data bits.
    /// 2 stop bits when using 6-8 data bits.
    More,
}

/// Parity bit configuration for a UART frame.
pub enum ParityBit {
    /// Parity bit is not used.
    None,
    /// The total count of 1 bits in the data frame, including the parity bit,
    /// is even.
    ///
    /// Examples:
    /// - `01101` would have a parity bit of `1` since there is an odd number
    /// of 1s and an extra 1 is needed to make it even.
    /// - `01100` would have a parity bit of `0` since there is already an even
    /// number of 1s.
    Even,
    /// The total count of 1 bits in the data frame, including the parity bit,
    /// is odd.
    ///
    /// Examples:
    /// - `01101` would have a parity bit of `0` since there is already an odd
    /// number of 1s.
    /// - `01100` would have a parity bit of `1` since there is an even number
    /// of 1s and an extra 1 is needed to make it odd.
    Odd,
    /// The parity bit is always `0`.
    SpaceZero,
    /// The parity bit is always `1`.
    MarkOne,
}

#[doc(hidden)]
pub mod marker {
    /// Marker traits for the build state of the UART peripheral.
    pub trait UartState: crate::Sealed {}
    #[doc(hidden)]
    pub struct NotBuilt;
    #[doc(hidden)]
    pub struct Built;

    impl crate::Sealed for NotBuilt {}
    impl crate::Sealed for Built {}
    impl UartState for NotBuilt {}
    impl UartState for Built {}

    /// Marker traits for the clock state of the UART peripheral.
    pub trait UartClockState: crate::Sealed {}
    #[doc(hidden)]
    pub struct NotClockSet;
    #[doc(hidden)]
    pub struct ClockSet;
    impl crate::Sealed for NotClockSet {}
    impl crate::Sealed for ClockSet {}
    impl UartClockState for NotClockSet {}
    impl UartClockState for ClockSet {}
}

/// # Universal Asynchronous Receiver/Transmitter (UART) Peripheral
///
/// This struct makes use of [type states](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)
/// to ensure any UART peripheral cannot be configured with an invalid set
/// of pins or clocks.
///
/// Traits from [`embedded_hal_nb::serial`] are also implemented for the UART
/// peripherals.
///
/// ## Example
/// ```
/// let pins = hal::gpio::Gpio0::new(p.gpio0, &mut gcr.reg).split();
/// let uart = hal::uart::UartPeripheral::uart0(
///     p.uart0,                // UART peripheral from the PAC
///     &mut gcr.reg,           // GCR instance
///     pins.p0_0.into_af1(),   // RX pin
///     pins.p0_1.into_af1()    // TX pin
/// )
///     .clock_pclk(&clks.pclk) // or clocks_ibro(&ibro)
///     .baud(115200)
///     .data_bits(hal::uart::DataBits::Eight)
///     .stop_bits(hal::uart::StopBits::One)
///     .parity(hal::uart::Parity::None)
///     .build();
///
/// uart.write_bytes(b"Hello, world!\r\n");

/// ```
pub struct UartPeripheral<STATE: marker::UartState, CLOCK, UART, RX, TX, CTS, RTS> {
    _state: PhantomData<STATE>,
    _clock: PhantomData<CLOCK>,
    uart: UART,
    _rx_pin: RX,
    _tx_pin: TX,
    _cts_pin: CTS,
    _rts_pin: RTS,
    clk_src: Option<UartClockSource>,
    clk_src_freq: Option<u32>,
    baud: u32,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: ParityBit,
}

pub struct BuiltUartPeripheral<UART, RX, TX, CTS, RTS> {
    uart: UART,
    _rx_pin: RX,
    _tx_pin: TX,
    _cts_pin: CTS,
    _rts_pin: RTS
}

// TODO
// pub struct UartReceiver<UART, RX, CTS> {
//     _uart: UART,
//     _rx_pin: RX,
//     _cts_pin: CTS,
// }

// TODO
// pub struct UartTransmitter<UART, TX, RTS> {
//     _uart: UART,
//     _tx_pin: TX,
//     _rts_pin: RTS,
// }

/// Pins that can be used for receiving data on a UART peripheral.
pub trait RxPin<UART>: crate::Sealed {}
/// Pins that can be used for transmitting data on a UART peripheral.
pub trait TxPin<UART>: crate::Sealed {}

// TODO: Implement CTS and RTS pins for hardware flow control
// pub trait CtsPin<UART>: crate::Sealed {}
// pub trait RtsPin<UART>: crate::Sealed {}

// All UART peripherals are derived from the same register block
type UartRegisterBlock = crate::pac::uart0::RegisterBlock;

macro_rules! uart {
    (
        $uart:ident,
        rx: $rx_pin:ty,
        tx: $tx_pin:ty,
        cts: $cts_pin:ty,
        rts: $rts_pin:ty,
    ) => {
        paste! {
            use crate::pac::$uart;

            impl crate::Sealed for $rx_pin {}
            impl RxPin<$uart> for $rx_pin {}

            impl crate::Sealed for $tx_pin {}
            impl TxPin<$uart> for $tx_pin {}

            impl UartPeripheral<
                marker::NotBuilt,
                marker::NotClockSet,
                $uart,
                $rx_pin,
                $tx_pin,
                // $cts_pin,
                // $rts_pin
                (),
                (),
            >
            {
                #[doc = "Construct a new "]
                #[doc = stringify!([<$uart:upper>])]
                #[doc = " peripheral."]
                pub fn [<$uart:lower>](
                    uart: $uart,
                    reg: &mut crate::gcr::GcrRegisters,
                    rx_pin: $rx_pin,
                    tx_pin: $tx_pin
                ) -> UartPeripheral<marker::NotBuilt, marker::NotClockSet, $uart, $rx_pin, $tx_pin, (), ()> {
                    // Enable the UART peripheral clock
                    unsafe { uart.enable_clock(&mut reg.gcr); }
                    UartPeripheral {
                        _state: PhantomData,
                        _clock: PhantomData,
                        uart,
                        _rx_pin: rx_pin,
                        _tx_pin: tx_pin,
                        _cts_pin: (),
                        _rts_pin: (),
                        clk_src: None,
                        clk_src_freq: None,
                        baud: 115200,
                        data_bits: DataBits::Eight,
                        stop_bits: StopBits::One,
                        parity: ParityBit::None,
                    }
                }
            }
        }
    };
}

uart! {Uart0,
    rx: Pin<0, 0, Af1>,
    tx: Pin<0, 1, Af1>,
    cts: (),
    rts: (),
}

uart! {Uart1,
    rx: Pin<0, 12, Af1>,
    tx: Pin<0, 13, Af1>,
    cts: (),
    rts: (),
}

uart! {Uart2,
    rx: Pin<1, 0, Af1>,
    tx: Pin<1, 1, Af1>,
    cts: (),
    rts: (),
}

/// # Clock Methods
/// You must set the clock source for the UART peripheral after using a
/// constructor and before building the peripheral.
impl<UART, RX, TX, CTS, RTS> UartPeripheral<marker::NotBuilt, marker::NotClockSet, UART, RX, TX, CTS, RTS> {
    /// Set the clock source for the UART peripheral to the PCLK.
    pub fn clock_pclk(self, clock: &Clock<PeripheralClock>) ->
    UartPeripheral<marker::NotBuilt, marker::ClockSet, UART, RX, TX, CTS, RTS>
    {
        UartPeripheral {
            _state: PhantomData,
            _clock: PhantomData,
            uart: self.uart,
            _rx_pin: self._rx_pin,
            _tx_pin: self._tx_pin,
            _cts_pin: self._cts_pin,
            _rts_pin: self._rts_pin,
            clk_src: Some(UartClockSource::Pclk),
            clk_src_freq: Some(clock.frequency),
            baud: self.baud,
            data_bits: self.data_bits,
            stop_bits: self.stop_bits,
            parity: self.parity,
        }
    }

    /// Set the clock source for the UART peripheral to the IBRO.
    pub fn clock_ibro(self, clock: &Clock<InternalBaudRateOscillator>) ->
    UartPeripheral<marker::NotBuilt, marker::ClockSet, UART, RX, TX, CTS, RTS>
    {
        UartPeripheral {
            _state: PhantomData,
            _clock: PhantomData,
            uart: self.uart,
            _rx_pin: self._rx_pin,
            _tx_pin: self._tx_pin,
            _cts_pin: self._cts_pin,
            _rts_pin: self._rts_pin,
            clk_src: Some(UartClockSource::Ibro),
            clk_src_freq: Some(clock.frequency),
            baud: self.baud,
            data_bits: self.data_bits,
            stop_bits: self.stop_bits,
            parity: self.parity,
        }
    }
}

/// # Builder Methods
/// These methods are used to configure the UART peripheral before it is built
/// to be used. Configure the peripheral by chaining these methods together,
/// with the [`UartPeripheral::build()`] method called at the end.
impl<CLOCK, UART, RX, TX, CTS, RTS> UartPeripheral<marker::NotBuilt, CLOCK, UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    /// Set the baud rate (bits per second) for the UART peripheral.
    /// 
    /// Default: `115200`
    pub fn baud(mut self, baud: u32) -> Self {
        self.baud = baud;
        self
    }

    /// Set the number of data bits for the UART peripheral.
    /// 
    /// Default: [`DataBits::Eight`]
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the number of stop bits for the UART peripheral.
    /// 
    /// Default: [`StopBits::One`]
    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Set the parity for the UART peripheral.
    /// 
    /// Default: [`ParityBit::None`]
    pub fn parity(mut self, parity: ParityBit) -> Self {
        self.parity = parity;
        self
    }

    // TODO: Implement hardware flow control
    // pub fn enable_hfc(
    //     self,
    //     cts_pin: $cts_pin,
    //     rts_pin: $rts_pin
    // ) -> UartPeripheral<NotBuilt, CLOCK, $uart, RX, TX, $cts_pin, $rts_pin> {
    //     // Enable CTS and RTS pins
    //     // cts_pin.enable();
    //     // rts_pin.enable();
    //     UartPeripheral {
    //         _state: PhantomData,
    //         _clock: PhantomData,
    //         uart: self.uart,
    //         _rx_pin: self._rx_pin,
    //         _tx_pin: self._tx_pin,
    //         _cts_pin: cts_pin,
    //         _rts_pin: rts_pin,
    //         clk_src: self.clk_src,
    //         clk_src_freq: self.clk_src_freq,
    //         baud: self.baud,
    //         data_bits: self.data_bits,
    //         stop_bits: self.stop_bits,
    //         parity: self.parity,
    //     }
    // }
}

impl<UART, RX, TX, CTS, RTS> UartPeripheral<marker::NotBuilt, marker::ClockSet, UART, RX, TX, CTS, RTS>
where UART: Deref<Target = UartRegisterBlock>
{
    /// Apply all settings and configure the UART peripheral.
    /// This must be called before the UART peripheral can be used.
    pub fn build(self) -> BuiltUartPeripheral<UART, RX, TX, CTS, RTS> {
        // Configure the UART peripheral
        let clk_src_freq = self.clk_src_freq.unwrap();
        self.uart.ctrl().write(|w| {
            w.ucagm().set_bit();
            match self.clk_src {
                Some(UartClockSource::Pclk) => w.bclksrc().peripheral_clock(),
                Some(UartClockSource::Ibro) => w.bclksrc().clk2(),
                None => unreachable!("UART clock source not set"),
            };
            w.bclken().set_bit();
            match self.data_bits {
                DataBits::Five => w.char_size()._5bits(),
                DataBits::Six => w.char_size()._6bits(),
                DataBits::Seven => w.char_size()._7bits(),
                DataBits::Eight => w.char_size()._8bits(),
            };
            match self.stop_bits {
                StopBits::One => w.stopbits().clear_bit(),
                StopBits::More => w.stopbits().set_bit(),
            };
            match self.parity {
                ParityBit::None => w.par_en().clear_bit(),
                ParityBit::Even => w.par_en().set_bit().par_eo().clear_bit(),
                ParityBit::Odd => w.par_en().set_bit().par_eo().set_bit(),
                ParityBit::SpaceZero => w.par_en().set_bit().par_md().clear_bit(),
                ParityBit::MarkOne => w.par_en().set_bit().par_md().set_bit(),
            };
            return w;
        });
        // Set the baud rate
        let clkdiv = clk_src_freq / self.baud;
        self.uart.clkdiv().write(|w| unsafe { w.clkdiv().bits(clkdiv) });
        // Wait until baud clock is ready
        while self.uart.ctrl().read().bclkrdy().bit_is_clear() {}
        BuiltUartPeripheral {
            uart: self.uart,
            _rx_pin: self._rx_pin,
            _tx_pin: self._tx_pin,
            _cts_pin: self._cts_pin,
            _rts_pin: self._rts_pin
        }
    }
}

/// # UART Methods
/// These methods are used to interact with the UART peripheral after it has
/// been built.
impl<UART, RX, TX, CTS, RTS> BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    #[doc(hidden)]
    #[inline(always)]
    fn _is_tx_full(&self) -> bool {
        self.uart.status().read().tx_full().bit_is_set()
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_tx_empty(&self) -> bool {
        self.uart.status().read().tx_em().bit_is_set()
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _is_rx_empty(&self) -> bool {
        self.uart.status().read().rx_em().bit_is_set()
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _read_byte(&self) -> nb::Result<u8, serial::ErrorKind> {
        if self._is_rx_empty() {
            return Err(nb::Error::WouldBlock);
        }
        Ok(self.uart.fifo().read().data().bits())
    }

    #[doc(hidden)]
    #[inline(always)]
    fn _write_byte(&self, byte: u8) -> nb::Result<(), serial::ErrorKind> {
        if self._is_tx_full() {
            return Err(nb::Error::WouldBlock);
        }
        self.uart.fifo().write(|w| unsafe { w.data().bits(byte) });
        Ok(())
    }

    /// Flush the transmit buffer, ensuring that all bytes have been sent.
    /// This is a blocking operation.
    #[inline(always)]
    fn flush_tx(&self) {
        while !self._is_tx_empty() {}
    }

    /// Reads a single byte. This is a blocking operation.
    pub fn read_byte(&self) -> u8 {
        nb::block!(self._read_byte()).unwrap()
    }

    /// Writes a single byte. This is a blocking operation.
    pub fn write_byte(&self, byte: u8) {
        nb::block!(self._write_byte(byte)).unwrap()
    }

    /// Reads bytes to a buffer. The entire length of the buffer will be
    /// filled with bytes from the UART peripheral. This is a blocking
    /// operation.
    pub fn read_bytes(&self, buffer: &mut [u8]) {
        for byte in buffer {
            *byte = self.read_byte();
        }
    }

    /// Write bytes from a buffer (blocking). The entire buffer will be written
    /// to the UART peripheral. This is a blocking operation.
    pub fn write_bytes(&self, buffer: &[u8]) {
        for byte in buffer {
            self.write_byte(*byte);
        }
    }
}

// Embedded HAL non-blocking serial traits
impl<UART, RX, TX, CTS, RTS> serial::ErrorType for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    type Error = serial::ErrorKind;
}

impl<UART, RX, TX, CTS, RTS> serial::Read<u8> for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self._read_byte()
    }
}

impl<UART, RX, TX, CTS, RTS> serial::Write<u8> for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self._write_byte(byte)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.flush_tx();
        Ok(())
    }
}

// Embedded IO traits
impl<UART, RX, TX, CTS, RTS> embedded_io::ErrorType for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    type Error = core::convert::Infallible;
}

impl<UART, RX, TX, CTS, RTS> embedded_io::Read for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut count = 0;
        if buf.len() == 0 {
            return Ok(0);
        }
        // If no bytes are currently available to read, this function blocks
        // until at least one byte is available.
        if self._is_rx_empty() {
            let byte = self.read_byte();
            buf[count] = byte;
            count += 1;
        // If bytes are available, a non-zero amount of bytes is read.
        } else {
            while count < buf.len() && !self._is_rx_empty() {
                let byte = self.read_byte();
                buf[count] = byte;
                count += 1;
            }
        }
        Ok(count)
    }
}

impl<UART, RX, TX, CTS, RTS> embedded_io::ReadReady for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self._is_rx_empty())
    }
}

impl<UART, RX, TX, CTS, RTS> embedded_io::Write for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for byte in buf {
            self.write_byte(*byte);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_tx();
        Ok(())
    }
}

impl<UART, RX, TX, CTS, RTS> embedded_io::WriteReady for BuiltUartPeripheral<UART, RX, TX, CTS, RTS>
where
    UART: Deref<Target = UartRegisterBlock>
{
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self._is_tx_full())
    }
}