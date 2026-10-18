//! Quorum size for a voting population, ported from
//! `web-backend/app/utils/votes.py::required_votes` (Cochran sample size with a
//! margin of error that shrinks as the population grows, capped at 75% of N).
//!
//! Floats are safe here: every validator runs the same WASM, and Rust's WASM float
//! maths is IEEE-754 deterministic for these operations.

/// Minimum number of counted votes needed before a proposal can pass.
#[must_use]
pub fn required_votes(population: u32) -> u32 {
    if population == 0 {
        return 0;
    }
    let n = f64::from(population);
    let margin = error_margin(n);
    let base = 0.9604 / (margin * margin);
    let cochran = ceil_u32(base / (1.0 + (base - 1.0) / n));
    ceil_u32(0.75 * n).min(cochran)
}

fn error_margin(n: f64) -> f64 {
    if n < 100.0 {
        0.10 - (0.03 * (n - 1.0) / 99.0)
    } else if n < 500.0 {
        0.07 - (0.02 * (n - 100.0) / 400.0)
    } else {
        (0.05 - 0.03 * (n / 500.0).log10() / 2000_f64.log10()).max(0.02)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ceil_u32(value: f64) -> u32 {
    value.ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::required_votes;

    /// Reference values computed with the Python implementation in web-backend.
    #[test]
    fn matches_legacy_python_formula() {
        let cases = [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (10, 8),
            (50, 37),
            (99, 66),
            (100, 67),
            (250, 125),
            (499, 218),
            (500, 218),
            (1_000, 301),
            (10_000, 619),
            (1_000_000, 2_396),
        ];
        for (population, expected) in cases {
            assert_eq!(
                required_votes(population),
                expected,
                "population {population}"
            );
        }
    }
}
