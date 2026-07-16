//! The [`Indicator`] contract — the same streaming state machine `wickra-core`
//! uses, narrowed to a scalar `f64` output and no allocation.

/// A streaming technical indicator.
///
/// The contract mirrors `wickra-core`'s trait so a no-alloc indicator here is a
/// drop-in behavioural twin of its std counterpart:
///
/// - [`update`](Indicator::update) is called once per input point and is O(1) in
///   the series length. It returns `None` while the indicator is still warming
///   up and `Some(value)` once a defined value exists.
/// - [`reset`](Indicator::reset) clears all state, leaving the indicator exactly
///   as freshly constructed.
///
/// The associated [`Input`](Indicator::Input) is `Copy` (a price `f64` or a
/// [`Candle`](crate::Candle)) so the update path never clones or allocates.
pub trait Indicator {
    /// Type of one input data point (`f64` for a price, [`Candle`](crate::Candle)
    /// for an OHLC bar).
    type Input: Copy;

    /// Feed one new data point in and return the freshly computed value, or
    /// `None` while still warming up.
    fn update(&mut self, input: Self::Input) -> Option<f64>;

    /// Reset all internal state, leaving the indicator equivalent to a freshly
    /// constructed instance with the same parameters.
    fn reset(&mut self);

    /// Number of inputs required before the first non-`None` output.
    fn warmup_period(&self) -> usize;

    /// Whether the indicator has produced at least one value since construction
    /// or the last [`reset`](Indicator::reset).
    fn is_ready(&self) -> bool;

    /// Stable, human-readable indicator name.
    fn name(&self) -> &'static str;
}
