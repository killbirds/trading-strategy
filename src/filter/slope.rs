use super::{SlopeFilterType, SlopeParams, utils};
use crate::analyzer::slope_analyzer::SlopeAnalyzer;
use crate::candle_store::CandleStore;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 기울기 필터 적용
pub(crate) fn filter_slope<C: Candle + 'static + Clone>(
    coin: &str,
    params: &SlopeParams,
    candle_store: &CandleStore<C>,
) -> Result<bool> {
    log::debug!(
        "기울기 필터 적용 - 지표 타입: {:?}, 분석 기간: {}, 필터 타입: {:?}, 연속성: {}",
        params.indicator_type,
        params.period,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 검증
    utils::validate_period(params.period, "Slope")?;

    // 경계 조건 체크
    let required_length = params.period + params.consecutive_n;
    if !utils::check_sufficient_candles(candle_store.len(), required_length, coin) {
        return Ok(false);
    }

    // SlopeAnalyzer 생성
    let analyzer = SlopeAnalyzer::from_config(candle_store, &params.indicator_type);

    if analyzer.items.is_empty() {
        return Ok(false);
    }

    let result = match params.filter_type {
        SlopeFilterType::Upward => {
            let analysis = if params.use_linear_regression.unwrap_or(false) {
                analyzer.calculate_slope(params.period, params.p)
            } else {
                analyzer.calculate_simple_slope(params.period, params.p)
            };
            if let Some(analysis) = analysis {
                analysis.slope > 0.0
                    && analysis.strength >= params.strength_threshold.unwrap_or(0.02)
            } else {
                false
            }
        }
        SlopeFilterType::Downward => {
            let analysis = if params.use_linear_regression.unwrap_or(false) {
                analyzer.calculate_slope(params.period, params.p)
            } else {
                analyzer.calculate_simple_slope(params.period, params.p)
            };
            if let Some(analysis) = analysis {
                analysis.slope < 0.0
                    && analysis.strength >= params.strength_threshold.unwrap_or(0.02)
            } else {
                false
            }
        }
        SlopeFilterType::Sideways => analyzer.is_slope_sideways(
            params.period,
            params.p,
            params.use_linear_regression.unwrap_or(false),
        ),
        SlopeFilterType::StrengthAboveThreshold => analyzer.is_slope_strength_above(
            params.period,
            params.p,
            params.strength_threshold.unwrap_or(0.01),
            params.use_linear_regression.unwrap_or(false),
        ),
        SlopeFilterType::Accelerating => {
            let short_period = params.short_period.unwrap_or(params.period / 2);
            let long_period = params.period;
            analyzer.is_slope_accelerating(
                short_period,
                long_period,
                params.p,
                params.use_linear_regression.unwrap_or(false),
            )
        }
        SlopeFilterType::Decelerating => {
            let short_period = params.short_period.unwrap_or(params.period / 2);
            let long_period = params.period;
            analyzer.is_slope_decelerating(
                short_period,
                long_period,
                params.p,
                params.use_linear_regression.unwrap_or(false),
            )
        }
        SlopeFilterType::StrongUpward => {
            let analysis = if params.use_linear_regression.unwrap_or(false) {
                analyzer.calculate_slope(params.period, params.p)
            } else {
                analyzer.calculate_simple_slope(params.period, params.p)
            };
            if let Some(analysis) = analysis {
                analysis.is_upward()
                    && analysis.strength >= params.strength_threshold.unwrap_or(0.02)
            } else {
                false
            }
        }
        SlopeFilterType::StrongDownward => {
            let analysis = if params.use_linear_regression.unwrap_or(false) {
                analyzer.calculate_slope(params.period, params.p)
            } else {
                analyzer.calculate_simple_slope(params.period, params.p)
            };
            if let Some(analysis) = analysis {
                analysis.is_downward()
                    && analysis.strength >= params.strength_threshold.unwrap_or(0.02)
            } else {
                false
            }
        }
        SlopeFilterType::HighRSquared => {
            let analysis = analyzer.calculate_slope(params.period, params.p);
            if let Some(analysis) = analysis {
                analysis.r_squared >= params.r_squared_threshold.unwrap_or(0.7)
            } else {
                false
            }
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use trading_chart::CandleInterval;

    #[derive(Debug, Clone, PartialEq, Default)]
    struct TestCandle {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        datetime: DateTime<Utc>,
    }

    impl std::fmt::Display for TestCandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "TestCandle {{ open: {}, high: {}, low: {}, close: {}, volume: {} }}",
                self.open, self.high, self.low, self.close, self.volume
            )
        }
    }

    impl TestCandle {
        fn new(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
            Self {
                open,
                high,
                low,
                close,
                volume,
                datetime: Utc.timestamp_opt(0, 0).unwrap(),
            }
        }
    }

    impl Candle for TestCandle {
        fn market(&self) -> &str {
            "TEST/USDT"
        }

        fn datetime(&self) -> DateTime<Utc> {
            self.datetime
        }

        fn interval(&self) -> &CandleInterval {
            &CandleInterval::Minute1
        }

        fn open_price(&self) -> f64 {
            self.open
        }

        fn high_price(&self) -> f64 {
            self.high
        }

        fn low_price(&self) -> f64 {
            self.low
        }

        fn close_price(&self) -> f64 {
            self.close
        }

        fn volume(&self) -> f64 {
            self.volume
        }

        fn quote_volume(&self) -> f64 {
            self.close * self.volume
        }

        fn trade_count(&self) -> Option<u64> {
            None
        }
    }

    fn create_test_candles() -> Vec<TestCandle> {
        let mut candles = vec![
            TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0),
            TestCandle::new(103.0, 108.0, 102.0, 107.0, 1200.0),
            TestCandle::new(107.0, 112.0, 106.0, 110.0, 1500.0),
            TestCandle::new(110.0, 115.0, 109.0, 113.0, 1800.0),
            TestCandle::new(113.0, 118.0, 112.0, 116.0, 2000.0),
            TestCandle::new(116.0, 121.0, 115.0, 119.0, 2200.0),
            TestCandle::new(119.0, 124.0, 118.0, 122.0, 2400.0),
            TestCandle::new(122.0, 127.0, 121.0, 125.0, 2600.0),
            TestCandle::new(125.0, 130.0, 124.0, 128.0, 2800.0),
            TestCandle::new(128.0, 133.0, 127.0, 131.0, 3000.0),
            TestCandle::new(131.0, 136.0, 130.0, 134.0, 3200.0),
            TestCandle::new(134.0, 139.0, 133.0, 137.0, 3400.0),
            TestCandle::new(137.0, 142.0, 136.0, 140.0, 3600.0),
            TestCandle::new(140.0, 145.0, 139.0, 143.0, 3800.0),
            TestCandle::new(143.0, 148.0, 142.0, 146.0, 4000.0),
            TestCandle::new(146.0, 151.0, 145.0, 149.0, 4200.0),
            TestCandle::new(149.0, 154.0, 148.0, 152.0, 4400.0),
            TestCandle::new(152.0, 157.0, 151.0, 155.0, 4600.0),
            TestCandle::new(155.0, 160.0, 154.0, 158.0, 4800.0),
            TestCandle::new(158.0, 163.0, 157.0, 161.0, 5000.0),
        ];
        // 각 캔들에 고유한 datetime 설정
        for (i, candle) in candles.iter_mut().enumerate() {
            candle.datetime = Utc.timestamp_opt(i as i64, 0).unwrap();
        }
        candles
    }

    #[test]
    fn test_slope_upward() {
        let candles = create_test_candles();
        let params = SlopeParams {
            indicator_type: crate::analyzer::IndicatorTypeConfig::ClosePrice,
            period: 10,
            filter_type: SlopeFilterType::Upward,
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let candle_store = utils::create_candle_store(&candles);
        let result = filter_slope("TEST/USDT", &params, &candle_store);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_slope_downward() {
        let mut candles = create_test_candles();
        let start_idx = candles.len();
        for i in 0..10 {
            let mut candle = TestCandle::new(
                160.0 - i as f64 * 3.0,
                162.0,
                158.0 - i as f64 * 3.0,
                159.0 - i as f64 * 3.0,
                5000.0 - i as f64 * 100.0,
            );
            candle.datetime = Utc.timestamp_opt((start_idx + i) as i64, 0).unwrap();
            candles.push(candle);
        }

        let params = SlopeParams {
            indicator_type: crate::analyzer::IndicatorTypeConfig::ClosePrice,
            period: 10,
            filter_type: SlopeFilterType::Downward,
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let candle_store = utils::create_candle_store(&candles);
        let result = filter_slope("TEST/USDT", &params, &candle_store);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = vec![TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0)];
        let params = SlopeParams {
            indicator_type: crate::analyzer::IndicatorTypeConfig::ClosePrice,
            period: 10,
            filter_type: SlopeFilterType::Upward,
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let candle_store = utils::create_candle_store(&candles);
        let result = filter_slope("TEST/USDT", &params, &candle_store);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
