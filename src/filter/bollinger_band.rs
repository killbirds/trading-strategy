use super::{BollingerBandFilterType, BollingerBandParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 볼린저 밴드 필터 적용
pub fn filter_bollinger_band<C: Candle + 'static>(
    coin: &str,
    params: &BollingerBandParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "볼린저 밴드 필터 적용 - 기간: {}, 편차 배수: {}, 타입: {:?}, 연속성: {}",
        params.period,
        params.dev_mult,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 검증
    utils::validate_period(params.period, "BollingerBand")?;

    // 경계 조건 체크
    if !utils::check_sufficient_candles(candles.len(), params.period, coin) {
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candle_store = utils::create_candle_store(candles);

    // BBandAnalyzer 생성
    let analyzer = BBandAnalyzer::new(params.period, params.dev_mult, &candle_store);

    // 기존 볼린저 밴드 계산 결과도 가져옴 (로깅용)
    let (lower, middle, upper) = analyzer.get_bband();

    log::debug!("코인 {coin} 볼린저 밴드 - 상단: {upper:.2}, 중간: {middle:.2}, 하단: {lower:.2}");

    let result = match params.filter_type {
        BollingerBandFilterType::AboveUpperBand => {
            analyzer.is_above_upper_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::BelowLowerBand => {
            analyzer.is_below_lower_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::InsideBand => {
            !analyzer.is_above_upper_band(params.consecutive_n, params.p)
                && !analyzer.is_below_lower_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::OutsideBand => {
            analyzer.is_above_upper_band(params.consecutive_n, params.p)
                || analyzer.is_below_lower_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::AboveMiddleBand => {
            analyzer.is_above_middle_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::BelowMiddleBand => {
            analyzer.is_below_middle_band(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::BandWidthSufficient => analyzer.is_band_width_sufficient(params.p),
        BollingerBandFilterType::BreakThroughLowerBand => {
            analyzer.is_break_through_lower_band_from_below(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::SqueezeBreakout => {
            analyzer.is_squeeze_breakout_with_close_above_upper(params.squeeze_breakout_period)
        }
        BollingerBandFilterType::EnhancedSqueezeBreakout => analyzer
            .is_enhanced_squeeze_breakout_with_close_above_upper(
                params.enhanced_narrowing_period,
                params.enhanced_squeeze_period,
                params.squeeze_threshold,
            ),
        BollingerBandFilterType::SqueezeState => {
            analyzer.is_band_width_squeeze(params.consecutive_n, params.squeeze_threshold, params.p)
        }
        BollingerBandFilterType::BandWidthNarrowing => {
            analyzer.is_band_width_narrowing(params.consecutive_n)
        }
        BollingerBandFilterType::SqueezeExpansionStart => {
            analyzer.is_squeeze_expansion_start(params.squeeze_threshold)
        }
        BollingerBandFilterType::BreakThroughUpperBand => {
            analyzer.is_break_through_upper_band_from_below(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::BreakThroughLowerBandFromBelow => {
            analyzer.is_break_through_lower_band_from_below(params.consecutive_n, params.p)
        }
        BollingerBandFilterType::BandWidthExpanding => {
            !analyzer.is_band_width_narrowing(params.consecutive_n)
        }
        BollingerBandFilterType::MiddleBandSideways => analyzer.is_middle_band_sideways(
            params.consecutive_n,
            params.p,
            params.squeeze_threshold,
        ),
        BollingerBandFilterType::UpperBandSideways => analyzer.is_upper_band_sideways(
            params.consecutive_n,
            params.p,
            params.squeeze_threshold,
        ),
        BollingerBandFilterType::LowerBandSideways => analyzer.is_lower_band_sideways(
            params.consecutive_n,
            params.p,
            params.squeeze_threshold,
        ),
        BollingerBandFilterType::BandWidthSideways => analyzer.is_band_width_sideways(
            params.consecutive_n,
            params.p,
            params.squeeze_threshold,
        ),
        BollingerBandFilterType::UpperBandTouch => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                previous.candle.close_price()
                    >= previous.bband.upper() * params.upper_touch_threshold
                    && current.candle.close_price() < current.bband.upper()
            }
        }
        BollingerBandFilterType::LowerBandTouch => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                previous.candle.close_price()
                    <= previous.bband.lower() * params.lower_touch_threshold
                    && current.candle.close_price() > current.bband.lower()
            }
        }
        BollingerBandFilterType::BandWidthThresholdBreakthrough => analyzer
            .is_band_width_threshold_breakthrough(
                params.consecutive_n,
                1,
                params.medium_threshold,
                params.p,
            ),
        BollingerBandFilterType::PriceMovingToUpperFromMiddle => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                let prev_middle_dist =
                    (previous.candle.close_price() - previous.bband.middle()).abs();
                let current_upper_dist =
                    (current.candle.close_price() - current.bband.upper()).abs();
                let band_width = previous.bband.upper() - previous.bband.lower();
                let threshold = params.large_threshold;
                prev_middle_dist < band_width * threshold
                    && current_upper_dist < band_width * threshold
            }
        }
        BollingerBandFilterType::PriceMovingToLowerFromMiddle => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                let prev_middle_dist =
                    (previous.candle.close_price() - previous.bband.middle()).abs();
                let current_lower_dist =
                    (current.candle.close_price() - current.bband.lower()).abs();
                let band_width = previous.bband.upper() - previous.bband.lower();
                let threshold = params.large_threshold;
                prev_middle_dist < band_width * threshold
                    && current_lower_dist < band_width * threshold
            }
        }
        BollingerBandFilterType::BandConvergenceThenDivergence => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_width =
                    analyzer.items[params.p].bband.upper() - analyzer.items[params.p].bband.lower();
                let prev_width = analyzer.items[params.p + 1].bband.upper()
                    - analyzer.items[params.p + 1].bband.lower();
                let prev_prev_width = analyzer.items[params.p + 2].bband.upper()
                    - analyzer.items[params.p + 2].bband.lower();
                prev_width < prev_prev_width && current_width > prev_width
            }
        }
        BollingerBandFilterType::BandDivergenceThenConvergence => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_width =
                    analyzer.items[params.p].bband.upper() - analyzer.items[params.p].bband.lower();
                let prev_width = analyzer.items[params.p + 1].bband.upper()
                    - analyzer.items[params.p + 1].bband.lower();
                let prev_prev_width = analyzer.items[params.p + 2].bband.upper()
                    - analyzer.items[params.p + 2].bband.lower();
                prev_width > prev_prev_width && current_width < prev_width
            }
        }
        BollingerBandFilterType::PriceMovingToUpperWithinBand => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let middle = data.bband.middle();
                let upper = data.bband.upper();
                price > middle && price < upper
            },
            params.consecutive_n,
            params.p,
        ),
        BollingerBandFilterType::PriceMovingToLowerWithinBand => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let middle = data.bband.middle();
                let lower = data.bband.lower();
                price < middle && price > lower
            },
            params.consecutive_n,
            params.p,
        ),
        BollingerBandFilterType::LowVolatility => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_width =
                    analyzer.items[params.p].bband.upper() - analyzer.items[params.p].bband.lower();
                let avg_price = analyzer.items[params.p].candle.close_price();
                if avg_price == 0.0 {
                    false
                } else {
                    let width_ratio = current_width / avg_price;
                    width_ratio <= params.squeeze_threshold
                }
            }
        }
        BollingerBandFilterType::HighVolatility => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_width =
                    analyzer.items[params.p].bband.upper() - analyzer.items[params.p].bband.lower();
                let avg_price = analyzer.items[params.p].candle.close_price();
                if avg_price == 0.0 {
                    false
                } else {
                    let width_ratio = current_width / avg_price;
                    width_ratio >= params.medium_threshold
                }
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

    impl std::fmt::Display for TestCandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "TestCandle {{ open: {}, high: {}, low: {}, close: {}, volume: {} }}",
                self.open, self.high, self.low, self.close, self.volume
            )
        }
    }

    fn create_test_candles() -> Vec<TestCandle> {
        vec![
            // 상승 추세
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
            // 하락 추세
            TestCandle::new(131.0, 132.0, 125.0, 126.0, 3200.0),
            TestCandle::new(126.0, 127.0, 120.0, 121.0, 3000.0),
            TestCandle::new(121.0, 122.0, 115.0, 116.0, 2800.0),
            TestCandle::new(116.0, 117.0, 110.0, 111.0, 2600.0),
            TestCandle::new(111.0, 112.0, 105.0, 106.0, 2400.0),
            TestCandle::new(106.0, 107.0, 100.0, 101.0, 2200.0),
            TestCandle::new(101.0, 102.0, 95.0, 96.0, 2000.0),
            TestCandle::new(96.0, 97.0, 90.0, 91.0, 1800.0),
            TestCandle::new(91.0, 92.0, 85.0, 86.0, 1600.0),
            TestCandle::new(86.0, 87.0, 80.0, 81.0, 1400.0),
            // 추가 데이터
            TestCandle::new(81.0, 82.0, 75.0, 76.0, 1200.0),
            TestCandle::new(76.0, 77.0, 70.0, 71.0, 1000.0),
            TestCandle::new(71.0, 72.0, 65.0, 66.0, 800.0),
            TestCandle::new(66.0, 67.0, 60.0, 61.0, 600.0),
            TestCandle::new(61.0, 62.0, 55.0, 56.0, 400.0),
        ]
    }

    #[test]
    fn test_filter_type_0_above_upper_band() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 가격이 상단 밴드 위에 있는지 확인
        let is_above = result.unwrap();
        println!("상단 밴드 위 테스트 결과: {is_above}");
    }

    #[test]
    fn test_filter_type_1_below_lower_band() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 1.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 가격이 하단 밴드 아래에 있는지 확인
        let is_below = result.unwrap();
        println!("하단 밴드 아래 테스트 결과: {is_below}");
    }

    #[test]
    fn test_filter_type_13_break_through_upper() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 13.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 상단 밴드 돌파 확인
        let is_breakthrough = result.unwrap();
        println!("상단 밴드 돌파 테스트 결과: {is_breakthrough}");
    }

    #[test]
    fn test_filter_type_14_break_through_lower() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 14.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 하단 밴드 돌파 확인
        let is_breakthrough = result.unwrap();
        println!("하단 밴드 돌파 테스트 결과: {is_breakthrough}");
    }

    #[test]
    fn test_filter_type_15_band_expansion() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 15.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 밴드 확장 확인
        let is_expanding = result.unwrap();
        println!("밴드 확장 테스트 결과: {is_expanding}");
    }

    #[test]
    fn test_filter_type_16_middle_band_sideways() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 16.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 중간 밴드 횡보 확인
        let is_sideways = result.unwrap();
        println!("중간 밴드 횡보 테스트 결과: {is_sideways}");
    }

    #[test]
    fn test_filter_type_20_upper_band_touch() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 20.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 상단 밴드 터치 후 하락 확인
        let is_touch = result.unwrap();
        println!("상단 밴드 터치 테스트 결과: {is_touch}");
    }

    #[test]
    fn test_filter_type_21_lower_band_touch() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 21.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 하단 밴드 터치 후 상승 확인
        let is_touch = result.unwrap();
        println!("하단 밴드 터치 테스트 결과: {is_touch}");
    }

    #[test]
    fn test_filter_type_29_low_volatility() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 29.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 저변동성 확인
        let is_low_vol = result.unwrap();
        println!("저변동성 테스트 결과: {is_low_vol}");
    }

    #[test]
    fn test_filter_type_30_high_volatility() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 30.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 고변동성 확인
        let is_high_vol = result.unwrap();
        println!("고변동성 테스트 결과: {is_high_vol}");
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = vec![TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0)]; // 캔들 데이터 부족
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 캔들 데이터 부족으로 false 반환
    }

    #[test]
    fn test_invalid_filter_type() {
        let candles = create_test_candles();
        let params = BollingerBandParams {
            period: 10,
            dev_mult: 2.0,
            filter_type: 99.into(), // 유효하지 않은 필터 타입
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 유효하지 않은 필터 타입은 항상 false 반환
    }
}
