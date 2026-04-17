use super::{
    PriceReferenceGapFilterType, PriceReferenceGapParams, PriceReferenceSource, Result, utils,
};
use crate::candle_store::CandleStore;
use crate::indicator::ma::MABuilderFactory;
use crate::indicator::max::MAXBuilder;
use crate::indicator::min::MINBuilder;
use crate::indicator::vwap::{VWAPBuilder, VWAPParams as IndicatorVWAPParams};
use trading_chart::Candle;

pub(crate) fn filter_price_reference_gap<C: Candle + 'static>(
    symbol: &str,
    params: &PriceReferenceGapParams,
    candle_store: &CandleStore<C>,
) -> Result<bool> {
    let required_length = required_candle_count(params);
    if !utils::check_sufficient_candles(candle_store.len(), required_length, symbol) {
        return Ok(false);
    }

    let ascending_items = candle_store.get_ascending_items();

    let result = match &params.reference_source {
        PriceReferenceSource::MovingAverage { ma_type, period } => {
            matches_reference_gap(&ascending_items, params, |window| {
                let mut builder = MABuilderFactory::build::<C>(ma_type, *period);
                let mut value = 0.0;

                for candle in window {
                    value = builder.next(candle).get();
                }

                Some(value)
            })
        }
        PriceReferenceSource::VWAP { period } => {
            let mut builder = VWAPBuilder::<C>::new(IndicatorVWAPParams { period: *period });
            matches_reference_gap(&ascending_items, params, |window| {
                Some(builder.build(window).value)
            })
        }
        PriceReferenceSource::HighestHigh {
            lookback_period,
            include_current_candle,
        } => {
            let mut builder = MAXBuilder::<C>::new(*lookback_period);
            matches_reference_gap(&ascending_items, params, |window| {
                high_low_reference_window(window, *include_current_candle)
                    .map(|reference_window| builder.build(reference_window).max)
            })
        }
        PriceReferenceSource::LowestLow {
            lookback_period,
            include_current_candle,
        } => {
            let mut builder = MINBuilder::<C>::new(*lookback_period);
            matches_reference_gap(&ascending_items, params, |window| {
                high_low_reference_window(window, *include_current_candle)
                    .map(|reference_window| builder.build(reference_window).min)
            })
        }
    };

    Ok(result)
}

fn required_candle_count(params: &PriceReferenceGapParams) -> usize {
    let reference_period = match &params.reference_source {
        PriceReferenceSource::MovingAverage { period, .. } => *period,
        PriceReferenceSource::VWAP { period } => *period,
        PriceReferenceSource::HighestHigh {
            lookback_period,
            include_current_candle,
        } => *lookback_period + usize::from(!include_current_candle),
        PriceReferenceSource::LowestLow {
            lookback_period,
            include_current_candle,
        } => *lookback_period + usize::from(!include_current_candle),
    };

    reference_period + params.p + params.consecutive_n.saturating_sub(1)
}

fn matches_reference_gap<C: Candle>(
    ascending_items: &[C],
    params: &PriceReferenceGapParams,
    mut reference_value: impl FnMut(&[C]) -> Option<f64>,
) -> bool {
    for offset in params.p..params.p + params.consecutive_n {
        let Some(window_end) = ascending_items.len().checked_sub(offset) else {
            return false;
        };

        if window_end == 0 {
            return false;
        }

        let window = &ascending_items[..window_end];
        let Some(current_candle) = window.last() else {
            return false;
        };

        let current_price = current_candle.close_price();
        let Some(reference_price) = reference_value(window) else {
            return false;
        };

        let Some(gap_ratio) = compute_gap_ratio(current_price, reference_price) else {
            return false;
        };

        if !matches_gap_threshold(gap_ratio, params.filter_type, params.gap_threshold) {
            return false;
        }
    }

    true
}

fn high_low_reference_window<C>(window: &[C], include_current_candle: bool) -> Option<&[C]> {
    if include_current_candle {
        return Some(window);
    }

    if window.len() < 2 {
        return None;
    }

    Some(&window[..window.len() - 1])
}

fn compute_gap_ratio(current_price: f64, reference_price: f64) -> Option<f64> {
    if reference_price == 0.0 {
        return None;
    }

    Some((current_price - reference_price) / reference_price)
}

fn matches_gap_threshold(
    gap_ratio: f64,
    filter_type: PriceReferenceGapFilterType,
    threshold: f64,
) -> bool {
    match filter_type {
        PriceReferenceGapFilterType::GapAboveThreshold => gap_ratio.abs() >= threshold,
        PriceReferenceGapFilterType::GapBelowThreshold => gap_ratio.abs() <= threshold,
        PriceReferenceGapFilterType::GapAboveReferenceThreshold => gap_ratio >= threshold,
        PriceReferenceGapFilterType::GapBelowReferenceThreshold => gap_ratio <= -threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator::ma::MAType;
    use crate::tests::TestCandle;

    fn test_candle(timestamp: i64, close: f64, high: f64, low: f64, volume: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume,
        }
    }

    fn build_candles(values: &[(f64, f64, f64)]) -> Vec<TestCandle> {
        values
            .iter()
            .enumerate()
            .map(|(index, (close, high, low))| {
                test_candle(index as i64 + 1, *close, *high, *low, 1_000.0)
            })
            .collect()
    }

    fn run_filter(params: PriceReferenceGapParams, candles: &[TestCandle]) -> bool {
        let candle_store = utils::create_candle_store(candles);
        filter_price_reference_gap("TEST/USDT", &params, &candle_store).unwrap()
    }

    #[test]
    fn test_filter_price_reference_gap_sma_gap_above_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (130.0, 131.0, 129.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_ema_gap_below_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (130.0, 131.0, 129.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::EMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
            gap_threshold: 0.15,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_vwap_gap_above_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (150.0, 152.0, 148.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::VWAP { period: 4 },
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: 0.20,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_highest_high_gap_below_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (104.0, 106.0, 103.0),
            (108.0, 110.0, 107.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::HighestHigh {
                lookback_period: 3,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
            gap_threshold: 0.02,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_lowest_low_gap_above_threshold() {
        let candles =
            build_candles(&[(94.0, 95.0, 90.0), (96.0, 97.0, 92.0), (120.0, 121.0, 95.0)]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::LowestLow {
                lookback_period: 3,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: 0.20,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_respects_p_for_highest_high() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (110.0, 111.0, 109.0),
            (200.0, 210.0, 190.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::HighestHigh {
                lookback_period: 3,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
            gap_threshold: 0.02,
            consecutive_n: 1,
            p: 1,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_consecutive_n_recomputes_reference_per_bar() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (104.0, 105.0, 103.0),
            (107.0, 108.0, 106.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::HighestHigh {
                lookback_period: 2,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
            gap_threshold: 0.02,
            consecutive_n: 2,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_returns_false_for_insufficient_history() {
        let candles = build_candles(&[(100.0, 101.0, 99.0), (102.0, 103.0, 101.0)]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: 0.02,
            consecutive_n: 1,
            p: 0,
        };

        assert!(!run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_returns_false_when_reference_is_zero() {
        let candles = build_candles(&[(0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::LowestLow {
                lookback_period: 3,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
            gap_threshold: 0.02,
            consecutive_n: 1,
            p: 0,
        };

        assert!(!run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_highest_high_previous_bars_only() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (104.0, 105.0, 103.0),
            (108.0, 109.0, 107.0),
            (120.0, 121.0, 119.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::HighestHigh {
                lookback_period: 3,
                include_current_candle: false,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_highest_high_including_current_rejects_same_case() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (104.0, 105.0, 103.0),
            (108.0, 109.0, 107.0),
            (120.0, 121.0, 119.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::HighestHigh {
                lookback_period: 3,
                include_current_candle: true,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(!run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_lowest_low_previous_bars_only() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (96.0, 97.0, 95.0),
            (92.0, 93.0, 91.0),
            (80.0, 81.0, 79.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::LowestLow {
                lookback_period: 3,
                include_current_candle: false,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_above_reference_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (130.0, 131.0, 129.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_below_reference_threshold() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (70.0, 71.0, 69.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(run_filter(params, &candles));
    }

    #[test]
    fn test_filter_price_reference_gap_above_reference_threshold_rejects_below_case() {
        let candles = build_candles(&[
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (100.0, 101.0, 99.0),
            (70.0, 71.0, 69.0),
        ]);
        let params = PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        };

        assert!(!run_filter(params, &candles));
    }
}
