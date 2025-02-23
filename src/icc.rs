//! # Instruction Cache Controller (ICC)

/// # Instruction Cache Controller (ICC)
///
/// Example:
/// ```
/// let icc = Icc::new(p.icc0);
/// icc.enable();
/// icc.disable();
/// ```
pub struct Icc {
    icc: crate::pac::Icc0,
}

impl Icc {
    /// Create a new ICC peripheral instance.
    pub fn new(icc: crate::pac::Icc0) -> Self {
        Self { icc }
    }

    #[inline(always)]
    fn _is_ready(&self) -> bool {
        self.icc.ctrl().read().rdy().is_ready()
    }

    #[inline(always)]
    fn _invalidate(&self) {
        self.icc
            .invalidate()
            .write(|w| unsafe { w.invalid().bits(1) });
    }

    /// Disable the instruction cache controller.
    #[inline(always)]
    pub fn disable(&mut self) {
        self.icc.ctrl().modify(|_, w| w.en().dis());
    }

    /// Enable the instruction cache controller.
    #[inline(always)]
    pub fn enable(&mut self) {
        // Invalidate cache
        self.disable();
        self._invalidate();
        while !self._is_ready() {}
        self.icc.ctrl().modify(|_, w| w.en().en());
        while !self._is_ready() {}
    }
}
