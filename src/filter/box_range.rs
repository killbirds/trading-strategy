use super::{BoxRangeFilterType, BoxRangeParams, Result, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::box_range_analyzer::{BoxRangeAnalyzer, BoxRangeAnalyzerData};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_box_range<C: Candle + 'static>(
    symbol: &str,
    params: &BoxRangeParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    utils::validate_period(params.period, "BoxRange")?;
    utils::validate_positive_number(params.max_width_ratio, "BoxRange max_width_ratio")?;
    utils::validate_consecutive_n(params.consecutive_n, "BoxRange consecutive_n")?;
    utils::validate_non_negative_number(
        params.width_ratio_threshold,
        "BoxRange width_ratio_threshold",
    )?;
    utils::validate_period(params.prior_box_count, "BoxRange prior_box_count")?;

    let required_length = required_candle_count(params);
    if !utils::check_sufficient_candles(candle_store.len(), required_length, symbol) {
        return Ok(false);
    }

    let analyzer = BoxRangeAnalyzer::new(params.period, params.max_width_ratio, candle_store);

    let result = match params.filter_type {
        BoxRangeFilterType::IsBoxRange => analyzer.is_box_range(params.consecutive_n, params.p),
        BoxRangeFilterType::InsideBox => matches_current_box(&analyzer, params, |data| {
            data.box_range.contains_price(current_price)
        }),
        BoxRangeFilterType::OutsideBox => matches_current_box(&analyzer, params, |data| {
            !data.box_range.contains_price(current_price)
        }),
        BoxRangeFilterType::AboveUpperBox => matches_current_box(&analyzer, params, |data| {
            current_price > data.box_range.upper()
        }),
        BoxRangeFilterType::BelowLowerBox => matches_current_box(&analyzer, params, |data| {
            current_price < data.box_range.lower()
        }),
        BoxRangeFilterType::BoxRangeStart => {
            analyzer.is_box_range_start(params.consecutive_n, params.prior_box_count, params.p)
        }
        BoxRangeFilterType::BreakoutAbove => {
            matches_previous_box(&analyzer, params, |current, previous| {
                let _ = current;
                current_price > previous.box_range.upper()
            })
        }
        BoxRangeFilterType::BreakoutBelow => {
            matches_previous_box(&analyzer, params, |current, previous| {
                let _ = current;
                current_price < previous.box_range.lower()
            })
        }
        BoxRangeFilterType::HighBreakThroughUpperBox => {
            matches_previous_box(&analyzer, params, |current, previous| {
                current.candle.high_price() > previous.box_range.upper()
            })
        }
        BoxRangeFilterType::LowBreakThroughLowerBox => {
            matches_previous_box(&analyzer, params, |current, previous| {
                current.candle.low_price() < previous.box_range.lower()
            })
        }
        BoxRangeFilterType::WidthRatioBelowThreshold => {
            matches_current_box(&analyzer, params, |data| {
                data.box_range.width_ratio() <= params.width_ratio_threshold
            })
        }
        BoxRangeFilterType::WidthRatioAboveThreshold => {
            matches_current_box(&analyzer, params, |data| {
                data.box_range.width_ratio() >= params.width_ratio_threshold
            })
        }
    };

    Ok(result)
}

fn required_candle_count(params: &BoxRangeParams) -> usize {
    let base = params
        .period
        .saturating_add(params.p)
        .saturating_add(params.consecutive_n.saturating_sub(1));

    match params.filter_type {
        BoxRangeFilterType::BreakoutAbove
        | BoxRangeFilterType::BreakoutBelow
        | BoxRangeFilterType::HighBreakThroughUpperBox
        | BoxRangeFilterType::LowBreakThroughLowerBox => base.saturating_add(1),
        BoxRangeFilterType::BoxRangeStart => base.saturating_add(params.prior_box_count),
        _ => base,
    }
}

fn matches_current_box<C: Candle>(
    analyzer: &BoxRangeAnalyzer<C>,
    params: &BoxRangeParams,
    predicate: impl Fn(&BoxRangeAnalyzerData<C>) -> bool,
) -> bool {
    analyzer.is_all(
        |data| data.box_range.is_box_range() && predicate(data),
        params.consecutive_n,
        params.p,
    )
}

fn matches_previous_box<C: Candle>(
    analyzer: &BoxRangeAnalyzer<C>,
    params: &BoxRangeParams,
    predicate: impl Fn(&BoxRangeAnalyzerData<C>, &BoxRangeAnalyzerData<C>) -> bool,
) -> bool {
    if analyzer.items.len() < params.p + params.consecutive_n + 1 {
        return false;
    }

    for index in params.p..params.p + params.consecutive_n {
        let current = match analyzer.items.get(index) {
            Some(data) => data,
            None => return false,
        };
        let previous = match analyzer.items.get(index + 1) {
            Some(data) => data,
            None => return false,
        };

        if !previous.box_range.is_box_range() || !predicate(current, previous) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    fn box_candles() -> Vec<TestCandle> {
        vec![
            candle(1, 101.0, 99.0, 100.0),
            candle(2, 102.0, 98.5, 101.0),
            candle(3, 101.5, 99.5, 100.5),
            candle(4, 102.0, 98.0, 99.5),
            candle(5, 101.0, 99.0, 100.0),
        ]
    }

    fn run_filter(params: BoxRangeParams, candles: &[TestCandle], current_price: f64) -> bool {
        let candle_store = utils::create_candle_store(candles);
        filter_box_range("TEST/USDT", &params, &candle_store, current_price).unwrap()
    }

    fn params(filter_type: BoxRangeFilterType) -> BoxRangeParams {
        BoxRangeParams {
            period: 5,
            max_width_ratio: 0.05,
            filter_type,
            consecutive_n: 1,
            p: 0,
            prior_box_count: 1,
            width_ratio_threshold: 0.05,
        }
    }

    #[test]
    fn test_filter_box_range_detects_box() {
        assert!(run_filter(
            params(BoxRangeFilterType::IsBoxRange),
            &box_candles(),
            100.0,
        ));
    }

    #[test]
    fn test_filter_box_range_inside_and_outside_box() {
        let candles = box_candles();

        assert!(run_filter(
            params(BoxRangeFilterType::InsideBox),
            &candles,
            100.0,
        ));
        assert!(run_filter(
            params(BoxRangeFilterType::OutsideBox),
            &candles,
            103.0,
        ));
    }

    #[test]
    fn test_filter_box_range_above_and_below_box() {
        let candles = box_candles();

        assert!(run_filter(
            params(BoxRangeFilterType::AboveUpperBox),
            &candles,
            103.0,
        ));
        assert!(run_filter(
            params(BoxRangeFilterType::BelowLowerBox),
            &candles,
            97.0,
        ));
    }

    #[test]
    fn test_filter_box_range_rejects_wide_range() {
        let mut candles = box_candles();
        candles.push(candle(6, 130.0, 80.0, 110.0));

        assert!(!run_filter(
            params(BoxRangeFilterType::IsBoxRange),
            &candles,
            110.0,
        ));
    }

    #[test]
    fn test_filter_box_range_breakout_above_uses_previous_box() {
        let mut candles = box_candles();
        candles.push(candle(6, 103.0, 102.0, 103.0));

        assert!(run_filter(
            params(BoxRangeFilterType::BreakoutAbove),
            &candles,
            103.0,
        ));
        assert!(run_filter(
            params(BoxRangeFilterType::HighBreakThroughUpperBox),
            &candles,
            100.0,
        ));
    }

    #[test]
    fn test_filter_box_range_breakout_above_uses_live_price() {
        let mut candles = box_candles();
        candles.push(candle(6, 101.0, 99.0, 100.0));

        assert!(run_filter(
            params(BoxRangeFilterType::BreakoutAbove),
            &candles,
            103.0,
        ));
        assert!(!run_filter(
            params(BoxRangeFilterType::HighBreakThroughUpperBox),
            &candles,
            103.0,
        ));
    }

    #[test]
    fn test_filter_box_range_breakout_below_uses_previous_box() {
        let mut candles = box_candles();
        candles.push(candle(6, 98.0, 97.0, 97.0));

        assert!(run_filter(
            params(BoxRangeFilterType::BreakoutBelow),
            &candles,
            97.0,
        ));
        assert!(run_filter(
            params(BoxRangeFilterType::LowBreakThroughLowerBox),
            &candles,
            100.0,
        ));
    }

    #[test]
    fn test_filter_box_range_breakout_below_uses_live_price() {
        let mut candles = box_candles();
        candles.push(candle(6, 101.0, 99.0, 100.0));

        assert!(run_filter(
            params(BoxRangeFilterType::BreakoutBelow),
            &candles,
            97.0,
        ));
        assert!(!run_filter(
            params(BoxRangeFilterType::LowBreakThroughLowerBox),
            &candles,
            97.0,
        ));
    }

    #[test]
    fn test_filter_box_range_breakout_requires_previous_box() {
        let candles = vec![
            candle(1, 130.0, 80.0, 100.0),
            candle(2, 101.0, 99.0, 100.0),
            candle(3, 101.0, 99.0, 100.0),
            candle(4, 101.0, 99.0, 100.0),
            candle(5, 101.0, 99.0, 100.0),
            candle(6, 103.0, 102.0, 103.0),
        ];

        assert!(!run_filter(
            params(BoxRangeFilterType::BreakoutAbove),
            &candles,
            103.0,
        ));
    }

    #[test]
    fn test_filter_box_range_start_detects_new_box_after_prior_non_box() {
        let candles = vec![
            candle(1, 130.0, 80.0, 100.0),
            candle(2, 101.0, 99.0, 100.0),
            candle(3, 101.0, 99.0, 100.0),
            candle(4, 101.0, 99.0, 100.0),
            candle(5, 101.0, 99.0, 100.0),
            candle(6, 101.0, 99.0, 100.0),
        ];

        assert!(run_filter(
            params(BoxRangeFilterType::BoxRangeStart),
            &candles,
            100.0,
        ));
    }

    #[test]
    fn test_filter_box_range_width_ratio_threshold() {
        let candles = box_candles();
        let mut below_params = params(BoxRangeFilterType::WidthRatioBelowThreshold);
        below_params.width_ratio_threshold = 0.05;
        let mut above_params = params(BoxRangeFilterType::WidthRatioAboveThreshold);
        above_params.width_ratio_threshold = 0.03;

        assert!(run_filter(below_params, &candles, 100.0));
        assert!(run_filter(above_params, &candles, 100.0));
    }
}
