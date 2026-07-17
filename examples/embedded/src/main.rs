//! Bare-metal Cortex-M demo for `wickra-embed`.
//!
//! Runs `Sma<20>` and `Ema(20)` over a synthetic price path with **no OS and no
//! heap**, times each `update` with the DWT cycle counter, and reports the warm
//! bar count and mean cycles-per-update over semihosting. Under QEMU:
//!
//! ```text
//! cargo run --release   # from examples/embedded/
//! ```
//!
//! On real hardware the same firmware would drive a GPIO/LED from the crossover
//! instead of printing; the semihosting output here stands in for that so the
//! example is observable in CI/QEMU.

#![no_std]
#![no_main]

use cortex_m::peripheral::{Peripherals, DWT};
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use embed_core::{Ema, Indicator, Sma};
use panic_halt as _;

/// A finite, drifting-and-oscillating price path. `libm::sin` is the same routine
/// `embed-core` uses for its no_std math, so this needs no `std`.
fn price(i: usize) -> f64 {
    100.0 + 8.0 * libm::sin(i as f64 * 0.1) + 0.05 * i as f64
}

#[entry]
fn main() -> ! {
    let mut cp = Peripherals::take().unwrap();
    // Enable the data-watchpoint cycle counter for per-update latency timing.
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    let mut sma = Sma::<20>::new();
    let mut ema = Ema::new(20);

    let mut warm: u32 = 0;
    let mut total_cycles: u32 = 0;
    let mut updates: u32 = 0;

    for i in 0..64 {
        let p = price(i);
        let before = DWT::cycle_count();
        let s = sma.update(p);
        let e = ema.update(p);
        let after = DWT::cycle_count();

        total_cycles = total_cycles.wrapping_add(after.wrapping_sub(before));
        updates += 1;

        if s.is_some() && e.is_some() {
            // On hardware this is where a GPIO/LED crossover signal would toggle.
            warm += 1;
        }
    }

    hprintln!("warm bars: {}", warm);
    hprintln!("mean cycles/update (sma+ema): {}", total_cycles / updates);

    // Cleanly terminate the QEMU machine so `cargo run` returns success.
    debug::exit(debug::EXIT_SUCCESS);

    // Unreached under QEMU (the exit above halts the machine); on real hardware
    // there is no semihosting exit, so sleep the core instead of spinning hot.
    loop {
        cortex_m::asm::wfi();
    }
}
