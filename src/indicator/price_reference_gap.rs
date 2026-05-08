use trading_chart::Candle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceReferenceGap {
    pub current_price: f64,
    pub reference_price: f64,
    pub gap_ratio: f64,
}

pub fn required_candle_count(
    reference_period: usize,
    excludes_current_candle: bool,
    p: usize,
    consecutive_n: usize,
) -> usize {
    reference_period + usize::from(excludes_current_candle) + p + consecutive_n.saturating_sub(1)
}

pub fn matches_reference_gap<C: Candle>(
    ascending_items: &[C],
    current_price: f64,
    p: usize,
    consecutive_n: usize,
    mut reference_value: impl FnMut(&[C]) -> Option<f64>,
    matches_gap_threshold: impl Fn(f64) -> bool,
) -> bool {
    for offset in p..p + consecutive_n {
        let Some(window_end) = ascending_items.len().checked_sub(offset) else {
            return false;
        };

        if window_end == 0 {
            return false;
        }

        let window = &ascending_items[..window_end];
        let Some(reference_price) = reference_value(window) else {
            return false;
        };

        let Some(gap) = calculate_gap(current_price, reference_price) else {
            return false;
        };

        if !matches_gap_threshold(gap.gap_ratio) {
            return false;
        }
    }

    true
}

pub fn high_low_reference_window<C>(window: &[C], include_current_candle: bool) -> Option<&[C]> {
    if include_current_candle {
        return Some(window);
    }

    if window.len() < 2 {
        return None;
    }

    Some(&window[..window.len() - 1])
}

pub fn calculate_gap(current_price: f64, reference_price: f64) -> Option<PriceReferenceGap> {
    if reference_price == 0.0 {
        return None;
    }

    Some(PriceReferenceGap {
        current_price,
        reference_price,
        gap_ratio: (current_price - reference_price) / reference_price,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn calculates_gap_ratio() {
        let gap = calculate_gap(120.0, 100.0).unwrap();

        assert_eq!(gap.current_price, 120.0);
        assert_eq!(gap.reference_price, 100.0);
        assert!((gap.gap_ratio - 0.2).abs() < 1e-10);
    }

    #[test]
    fn matches_consecutive_reference_gaps() {
        let candles = vec![
            TestCandle {
                timestamp: 1,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 2,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 3,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1.0,
            },
        ];

        assert!(matches_reference_gap(
            &candles,
            120.0,
            0,
            2,
            |_| Some(100.0),
            |gap_ratio| gap_ratio >= 0.1,
        ));
    }
}
