use super::MovingAverageParams;
use crate::analyzer::AnalyzerOps;
use crate::analyzer::ma_analyzer::MAAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 이동평균선 필터 적용
pub fn filter_moving_average<C: Candle + 'static>(
    coin: &str,
    params: &MovingAverageParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "이동평균선 필터 적용 - 빠른 기간: {}, 느린 기간: {}, 타입: {}, 연속성: {}",
        params.fast_period,
        params.slow_period,
        params.filter_type,
        params.consecutive_n
    );

    // 필터링 로직
    let required_length = params.slow_period + params.consecutive_n; // 더 긴 기간 + 연속성 확인 기간
    if candles.len() < required_length {
        log::debug!(
            "코인 {} 캔들 데이터 부족: {} < {}",
            coin,
            candles.len(),
            required_length
        );
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candles_vec = candles.to_vec();
    let candle_store = CandleStore::new(candles_vec, candles.len() * 2, false);

    // MAAnalyzer 생성 (SMA 타입 사용)
    let ma_type = MAType::SMA;
    let ma_periods = vec![params.fast_period, params.slow_period];
    let analyzer = MAAnalyzer::new(&ma_type, &ma_periods, &candle_store);

    let result = match params.filter_type {
        // 0: 가격이 빠른 MA 위에 있는 경우 (단기 상승)
        0 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let price = data.candle.close_price();
                        let fast_ma = data.mas.get_by_key_index(0).get();
                        price > fast_ma
                    },
                    params.consecutive_n,
                )
            }
        }
        // 1: 가격이 느린 MA 위에 있는 경우 (장기 상승)
        1 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let price = data.candle.close_price();
                        let slow_ma = data.mas.get_by_key_index(1).get();
                        price > slow_ma
                    },
                    params.consecutive_n,
                )
            }
        }
        // 2: 가격이 빠르고 느린 MA 모두 위에 있는 경우 (강한 상승)
        2 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let price = data.candle.close_price();
                        let fast_ma = data.mas.get_by_key_index(0).get();
                        let slow_ma = data.mas.get_by_key_index(1).get();
                        price > fast_ma && price > slow_ma
                    },
                    params.consecutive_n,
                )
            }
        }
        // 3: 빠른 MA가 느린 MA 위에 있는 경우 (골든 크로스 이후 상태)
        3 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let fast_ma = data.mas.get_by_key_index(0).get();
                        let slow_ma = data.mas.get_by_key_index(1).get();
                        fast_ma > slow_ma
                    },
                    params.consecutive_n,
                )
            }
        }
        // 4: 빠른 MA가 느린 MA 아래에 있는 경우 (데드 크로스 이후 상태)
        4 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                let last_n_items = &analyzer.items[analyzer.items.len() - params.consecutive_n..];
                last_n_items.iter().all(|data| {
                    let fast_ma = data.mas.get_by_key_index(0).get();
                    let slow_ma = data.mas.get_by_key_index(1).get();
                    fast_ma < slow_ma
                })
            }
        }
        // 5: 골든 크로스 발생 확인
        5 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_fast_ma = analyzer.items[0].mas.get_by_key_index(0).get();
                let current_slow_ma = analyzer.items[0].mas.get_by_key_index(1).get();
                let previous_fast_ma = analyzer.items[1].mas.get_by_key_index(0).get();
                let previous_slow_ma = analyzer.items[1].mas.get_by_key_index(1).get();
                current_fast_ma > current_slow_ma && previous_fast_ma <= previous_slow_ma
            }
        }
        // 6: 가격이 두 MA 사이에 있는 경우
        6 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let price = data.candle.close_price();
                        let fast_ma = data.mas.get_by_key_index(0).get();
                        let slow_ma = data.mas.get_by_key_index(1).get();
                        (fast_ma <= price && price <= slow_ma)
                            || (slow_ma <= price && price <= fast_ma)
                    },
                    params.consecutive_n,
                )
            }
        }
        _ => false,
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
            // 하락 추세 (더 가파른 하락)
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
            // 추가 하락 데이터
            TestCandle::new(81.0, 82.0, 75.0, 76.0, 1200.0),
            TestCandle::new(76.0, 77.0, 70.0, 71.0, 1000.0),
            TestCandle::new(71.0, 72.0, 65.0, 66.0, 800.0),
            TestCandle::new(66.0, 67.0, 60.0, 61.0, 600.0),
            TestCandle::new(61.0, 62.0, 55.0, 56.0, 400.0),
        ]
    }

    #[test]
    fn test_filter_type_0_price_above_fast_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 0,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 5일 이동평균선 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_1_price_above_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 1,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 20일 이동평균선 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_2_price_above_both_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 2,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 두 이동평균선 모두 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_3_fast_ma_above_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 3,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 5일 이동평균선이 20일 이동평균선 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_4_fast_ma_below_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 4,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 5일 이동평균선이 20일 이동평균선 아래에 있는지 확인
        assert!(result.unwrap()); // 하락 추세이므로 true
    }

    #[test]
    fn test_filter_type_5_golden_cross() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 5,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 골든 크로스가 발생했는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_6_price_between_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 6,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 가격이 두 이동평균선 사이에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_invalid_filter_type() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 7, // 유효하지 않은 필터 타입
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 유효하지 않은 필터 타입은 항상 false 반환
    }

    #[test]
    fn test_consecutive_n_condition() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 4,   // 빠른 MA가 느린 MA 아래에 있는 경우
            consecutive_n: 3, // 3연속 조건
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(result.unwrap()); // 하락 추세이므로 true
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = vec![TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0)]; // 캔들 데이터 부족
        let params = MovingAverageParams {
            fast_period: 5,
            slow_period: 20,
            filter_type: 0,
            consecutive_n: 1,
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 캔들 데이터 부족으로 false 반환
    }
}
