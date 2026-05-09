use super::{FilterError, Result, utils};
use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps};
use crate::indicator::MAX_INDICATOR_CAPACITY;
use trading_chart::Candle;

pub(crate) fn validate_indicator_capacity(
    period: usize,
    multiplier: usize,
    addend: usize,
    param_name: &str,
) -> Result<()> {
    let Some(capacity) = period
        .checked_mul(multiplier)
        .and_then(|value| value.checked_add(addend))
    else {
        return Err(FilterError::InvalidPeriod {
            param_name: param_name.to_string(),
        });
    };

    if capacity > MAX_INDICATOR_CAPACITY {
        return Err(FilterError::InvalidPeriod {
            param_name: param_name.to_string(),
        });
    }

    Ok(())
}

pub(crate) fn checked_required_add(left: usize, right: usize, param_name: &str) -> Result<usize> {
    left.checked_add(right)
        .ok_or_else(|| FilterError::InvalidPeriod {
            param_name: param_name.to_string(),
        })
}

pub(crate) fn required_with_offsets(
    base: usize,
    consecutive_n: usize,
    p: usize,
    needs_previous: bool,
    name: &str,
) -> Result<usize> {
    let with_offset = checked_required_add(base, p, name)?;
    let with_consecutive =
        checked_required_add(with_offset, consecutive_n.saturating_sub(1), name)?;
    if needs_previous {
        checked_required_add(with_consecutive, 1, name)
    } else {
        Ok(with_consecutive)
    }
}

pub(crate) fn validate_common(consecutive_n: usize, name: &str) -> Result<()> {
    utils::validate_consecutive_n(consecutive_n, name)
}

pub(crate) fn matches_all<A, D, C>(
    analyzer: &A,
    consecutive_n: usize,
    p: usize,
    predicate: impl Fn(&D) -> bool,
) -> bool
where
    C: Candle,
    D: AnalyzerDataOps<C>,
    A: AnalyzerOps<D, C>,
{
    analyzer.is_all(predicate, consecutive_n, p)
}

pub(crate) fn matches_previous<A, D, C>(
    analyzer: &A,
    consecutive_n: usize,
    p: usize,
    predicate: impl Fn(&D, &D) -> bool,
) -> bool
where
    C: Candle,
    D: AnalyzerDataOps<C>,
    A: AnalyzerOps<D, C>,
{
    if analyzer.items().len() < p + consecutive_n + 1 {
        return false;
    }

    for index in p..p + consecutive_n {
        let current = match analyzer.items().get(index) {
            Some(data) => data,
            None => return false,
        };
        let previous = match analyzer.items().get(index + 1) {
            Some(data) => data,
            None => return false,
        };
        if !predicate(current, previous) {
            return false;
        }
    }

    true
}

pub(crate) fn matches_rising<A, D, C>(
    analyzer: &A,
    consecutive_n: usize,
    p: usize,
    value: impl Fn(&D) -> f64,
) -> bool
where
    C: Candle,
    D: AnalyzerDataOps<C>,
    A: AnalyzerOps<D, C>,
{
    matches_previous(analyzer, consecutive_n, p, |current, previous| {
        value(current) > value(previous)
    })
}

pub(crate) fn matches_falling<A, D, C>(
    analyzer: &A,
    consecutive_n: usize,
    p: usize,
    value: impl Fn(&D) -> f64,
) -> bool
where
    C: Candle,
    D: AnalyzerDataOps<C>,
    A: AnalyzerOps<D, C>,
{
    matches_previous(analyzer, consecutive_n, p, |current, previous| {
        value(current) < value(previous)
    })
}

pub(crate) fn channel_position(price: f64, lower: f64, upper: f64) -> f64 {
    let width = upper - lower;
    if width.abs() < f64::EPSILON {
        return 0.5;
    }
    (price - lower) / width
}
