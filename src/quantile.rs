//! Confidence intervals for quantiles
//!
//! # Examples
//!
//! ```
//! # fn main() -> stats_ci::error::CIResult<()> {
//! use stats_ci::{quantile,Confidence,Interval};
//! let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
//! let confidence = Confidence::new_two_sided(0.95);
//! let quantile = 0.5; // median
//! let interval = quantile::ci(confidence, &data, quantile).unwrap();
//! assert_eq!(interval, Interval::new(4, 12));
//!
//! let confidence = Confidence::new_two_sided(0.8);
//! let interval2 = quantile::ci(confidence, &data, quantile).unwrap();
//! assert_eq!(interval2, Interval::new(6, 10));
//!
//! let confidence = Confidence::new_two_sided(0.5);
//! let quantile = 0.2; // 20th percentile
//! let interval3 = quantile::ci(confidence, &data, quantile).unwrap();
//! assert_eq!(interval3, Interval::new(2, 5));
//! # Ok(())
//! # }
//! ```
//!
use super::*;
use crate::stats::z_value;

/// compute the confidence interval for a given quantile, assuming that the data is already sorted.
/// this is the function to call if the data is known to be sorted,
/// or if the order of elements is meant to be their position in the slice (e.g., order of arrival).
///
/// # Arguments
///
/// * `confidence` - the confidence level (must be in (0, 1))
/// * `sorted` - the sorted sample
/// * `quantile` - the quantile to compute the confidence interval for (must be in (0, 1))
///
/// # Output
/// 
/// * `Interval` - the confidence interval for the quantile
/// * `None` - if the number of samples is too small to compute a confidence interval, or if the interval falls outside the range of the data.
/// 
/// # Errors
///
/// * `TooFewSamples` - if the number of samples is too small to compute a confidence interval
/// * `InvalidConfidenceLevel` - if the confidence level is not in (0, 1)
/// * `InvalidQuantile` - if the quantile is not in (0, 1)
///
/// # Examples
///
/// ```
/// # use stats_ci::*;
/// # fn main() -> error::CIResult<()> {
/// let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
/// let confidence = Confidence::new_two_sided(0.95);
/// let quantile = 0.5; // median
/// let interval = quantile::ci_sorted_unchecked(confidence, &data, quantile).unwrap();
/// assert_eq!(interval, Interval::new(4, 12));
///
/// let confidence = Confidence::new_two_sided(0.8);
/// let interval2 = quantile::ci_sorted_unchecked(confidence, &data, quantile).unwrap();
/// assert_eq!(interval2, Interval::new(6, 10));
///
/// let confidence = Confidence::new_two_sided(0.5);
/// let quantile = 0.2; // 20th percentile
/// let interval3 = quantile::ci_sorted_unchecked(confidence, &data, quantile).unwrap();
/// assert_eq!(interval3, Interval::new(2, 5));
/// # Ok(())
/// # }
/// ```
///
pub fn ci_sorted_unchecked<T: Clone>(
    confidence: Confidence,
    sorted: &[T],
    quantile: f64,
) -> Option<Interval<T>> {
    assert!(quantile > 0. && quantile < 1.);

    let len = sorted.len();
    if len < 3 {
        // too few smaples to compute
        return None;
    }

    let z = z_value(confidence);
    let q = quantile; /* 0.5 for median */
    let n = len as f64;
    let mid_span = z * f64::sqrt(n * q * (1. - q));
    let lo = f64::ceil(n * q - mid_span) as usize - 1; // FIXME: check bounds; panics if the result is negative
    let hi = f64::ceil(n * q + mid_span) as usize - 1;
    if let (Some(lo), Some(hi)) = (sorted.get(lo), sorted.get(hi)) {
        Some(Interval::new_unordered_unchecked(lo.clone(), hi.clone()))
    } else {
        None
    }
}

/// compute the confidence interval for a given quantile
///
/// # Arguments
///
/// * `confidence` - the confidence level (must be in (0, 1))
/// * `data` - the sample
/// * `quantile` - the quantile to compute the confidence interval for (must be in (0, 1))
///
/// # Errors
///
/// * `TooFewSamples` - if the number of samples is too small to compute a confidence interval
/// * `InvalidConfidenceLevel` - if the confidence level is not in (0, 1)
/// * `InvalidQuantile` - if the quantile is not in (0, 1)
///
/// # Panics
/// 
/// * if the data contains elements that are not comparable (with their partial ordering).
/// 
/// # Examples
///
/// ```
/// # use stats_ci::*;
/// let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
/// let confidence = Confidence::new_two_sided(0.95);
/// let quantile = 0.5; // median
/// let interval = quantile::ci(confidence, &data, quantile).unwrap();
/// assert_eq!(interval, Interval::new(4, 12));
///
/// let data2 = [2, 14, 13, 6, 8, 4, 15, 9, 3, 11, 10, 7, 1, 12, 5];
/// let interval2 = quantile::ci(confidence, &data, quantile).unwrap();
/// assert_eq!(interval, interval2);
///
/// let confidence = Confidence::new_two_sided(0.8);
/// let interval3 = quantile::ci(confidence, &data, quantile).unwrap();
/// assert_eq!(interval3, Interval::new(6, 10));
///
/// let confidence = Confidence::new_two_sided(0.5);
/// let quantile = 0.2; // 20th percentile
/// let interval4 = quantile::ci(confidence, &data, quantile).unwrap();
/// assert_eq!(interval4, Interval::new(2, 5));
/// ```
pub fn ci<T: PartialOrd + Clone>(
    confidence: Confidence,
    data: &[T],
    quantile: f64,
) -> Option<Interval<T>> {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ci_sorted_unchecked(confidence, &sorted, quantile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_median_ci() {
        let data = [
            8., 11., 12., 13., 15., 17., 19., 20., 21., 21., 22., 23., 25., 26., 28.,
        ];
        let confidence = Confidence::new_two_sided(0.95);
        let median_ci = ci_sorted_unchecked(confidence, &data, 0.5);
        assert_eq!(median_ci, Some(Interval::new(13., 23.)));
    }

    #[test]
    fn test_quantile_ci() {
        let data = [
            8., 11., 12., 13., 15., 17., 19., 20., 21., 21., 22., 23., 25., 26., 28.,
        ];
        let confidence = Confidence::new_two_sided(0.95);
        let quantile_ci = ci_sorted_unchecked(confidence, &data, 0.25);
        assert_eq!(quantile_ci, Some(Interval::new(8., 20.)));
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Numbers {
        One,
        Two,
        Three,
        Four,
        Five,
        Six,
        Seven,
        Eight,
        Nine,
        Ten,
        Eleven,
        Twelve,
        Thirteen,
        Fourteen,
        Fifteen,
    }

    #[test]
    fn test_median_undordered() {
        use Numbers::*;
        let data = [
            One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Eleven, Twelve, Thirteen,
            Fourteen, Fifteen,
        ];
        let confidence = Confidence::new_two_sided(0.95);
        let median_ci = ci_sorted_unchecked(confidence, &data, 0.5).unwrap();
        assert_eq!(median_ci, Interval::new_unordered_unchecked(Four, Twelve));
        assert_eq!(median_ci.left(), Some(&Four));
        assert_eq!(median_ci.right(), Some(&Twelve));
    }

    #[test]
    fn test_median_ci_unsorted() {
        use rand::seq::SliceRandom;
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let confidence = Confidence::new_two_sided(0.95);
        let quantile = 0.5; // median
        for _i in 0..100 {
            let mut shuffled = data.to_vec();
            shuffled.shuffle(&mut thread_rng());
            let interval = ci(confidence, &shuffled, quantile).unwrap();
            assert_eq!(interval, Interval::new(4, 12));
        }
    }
}
