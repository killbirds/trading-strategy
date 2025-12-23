use super::{MovingAverageFilterType, MovingAverageParams, utils};
use crate::analyzer::AnalyzerOps;
use crate::analyzer::ma_analyzer::MAAnalyzer;
use crate::indicator::ma::MAType;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 이동평균선 필터 적용
pub fn filter_moving_average<C: Candle + 'static>(
    coin: &str,
    params: &MovingAverageParams,
    candles: &[C],
) -> Result<bool> {
    if params.periods.is_empty() {
        log::debug!("이동평균선 필터 적용 - 기간 목록이 비어 있음");
        return Ok(false);
    }

    log::debug!(
        "이동평균선 필터 적용 - 기간 목록: {:?}, 타입: {:?}, 연속성: {}",
        params.periods,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 검증
    for period in &params.periods {
        utils::validate_period(*period, "MovingAverage")?;
    }

    // 필터링 로직
    let max_period = params.periods.iter().max().copied().unwrap_or(1);
    if !utils::check_sufficient_candles(candles.len(), max_period, coin) {
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candle_store = utils::create_candle_store(candles);

    // MAAnalyzer 생성 (SMA 타입 사용)
    let ma_type = MAType::SMA;
    let analyzer = MAAnalyzer::new(&ma_type, &params.periods, &candle_store);

    // 필터 타입에 따라 로직 처리
    // 첫 번째와 마지막 MA 인덱스 결정
    let first_index = 0;
    let last_index = if params.periods.len() > 1 {
        params.periods.len() - 1
    } else {
        0
    };

    let result = match params.filter_type {
        MovingAverageFilterType::PriceAboveFirstMA => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let first_ma = data.mas.get_by_key_index(first_index).get();
                price > first_ma
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::PriceAboveLastMA => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let last_ma = data.mas.get_by_key_index(last_index).get();
                price > last_ma
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::RegularArrangement => {
            analyzer.is_ma_regular_arrangement(params.consecutive_n, params.p)
        }
        MovingAverageFilterType::FirstMAAboveLastMA => analyzer.is_all(
            |data| {
                let first_ma = data.mas.get_by_key_index(first_index).get();
                let last_ma = data.mas.get_by_key_index(last_index).get();
                first_ma > last_ma
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::FirstMABelowLastMA => analyzer.is_all(
            |data| {
                let first_ma = data.mas.get_by_key_index(first_index).get();
                let last_ma = data.mas.get_by_key_index(last_index).get();
                first_ma < last_ma
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::GoldenCross => {
            analyzer.is_ma_regular_arrangement_golden_cross(1, params.consecutive_n, params.p)
        }
        MovingAverageFilterType::PriceBetweenMA => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let first_ma = data.mas.get_by_key_index(first_index).get();
                let last_ma = data.mas.get_by_key_index(last_index).get();
                (first_ma <= price && price <= last_ma) || (last_ma <= price && price <= first_ma)
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::MAConvergence => {
            if analyzer.items.len() < params.p + 2 || params.periods.len() < 2 {
                false
            } else {
                let current_gap = (analyzer.items[params.p]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let previous_gap = (analyzer.items[params.p + 1]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 1]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                current_gap < previous_gap
            }
        }
        MovingAverageFilterType::MADivergence => {
            if analyzer.items.len() < params.p + 2 || params.periods.len() < 2 {
                false
            } else {
                let current_gap = (analyzer.items[params.p]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let previous_gap = (analyzer.items[params.p + 1]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 1]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                current_gap > previous_gap
            }
        }
        MovingAverageFilterType::AllMAAbove => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                for i in 0..params.periods.len() {
                    let ma = data.mas.get_by_key_index(i).get();
                    if price <= ma {
                        return false;
                    }
                }
                true
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::AllMABelow => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                for i in 0..params.periods.len() {
                    let ma = data.mas.get_by_key_index(i).get();
                    if price >= ma {
                        return false;
                    }
                }
                true
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::ReverseArrangement => {
            analyzer.is_ma_reverse_arrangement(params.consecutive_n, params.p)
        }
        MovingAverageFilterType::DeadCross => {
            analyzer.is_ma_reverse_arrangement_dead_cross(1, params.consecutive_n, params.p)
        }
        MovingAverageFilterType::MASideways => {
            if params.periods.is_empty() {
                false
            } else {
                analyzer.is_ma_sideways(
                    0,
                    params.consecutive_n,
                    params.p,
                    params.sideways_threshold,
                )
            }
        }
        MovingAverageFilterType::StrongUptrend => {
            if params.periods.is_empty() {
                false
            } else {
                analyzer.is_ma_greater_than_rate_of_return(0, 0.0, params.consecutive_n, params.p)
            }
        }
        MovingAverageFilterType::StrongDowntrend => {
            if params.periods.is_empty() {
                false
            } else {
                analyzer.is_ma_less_than_rate_of_return(0, 0.0, params.consecutive_n, params.p)
            }
        }
        MovingAverageFilterType::PriceCrossingMA => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let mut above_count = 0;
                let mut below_count = 0;

                for i in 0..params.periods.len() {
                    let ma = data.mas.get_by_key_index(i).get();
                    if price > ma {
                        above_count += 1;
                    } else if price < ma {
                        below_count += 1;
                    }
                }

                // 가격이 일부 MA 위에 있고 일부 MA 아래에 있으면 교차 중
                above_count > 0 && below_count > 0
            },
            params.consecutive_n,
            params.p,
        ),
        MovingAverageFilterType::ConvergenceDivergence => {
            if analyzer.items.len() < params.p + 3 || params.periods.len() < 2 {
                false
            } else {
                let current_gap = (analyzer.items[params.p]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let prev_gap = (analyzer.items[params.p + 1]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 1]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let prev_prev_gap = (analyzer.items[params.p + 2]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 2]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();

                // 이전에 수렴했다가 현재 발산하기 시작
                prev_gap < prev_prev_gap && current_gap > prev_gap
            }
        }
        MovingAverageFilterType::DivergenceConvergence => {
            if analyzer.items.len() < params.p + 3 || params.periods.len() < 2 {
                false
            } else {
                let current_gap = (analyzer.items[params.p]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let prev_gap = (analyzer.items[params.p + 1]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 1]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let prev_prev_gap = (analyzer.items[params.p + 2]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p + 2]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();

                // 이전에 발산했다가 현재 수렴하기 시작
                prev_gap > prev_prev_gap && current_gap < prev_gap
            }
        }
        MovingAverageFilterType::ParallelMovement => {
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut all_parallel = true;
                for i in 0..params.consecutive_n {
                    let current = &analyzer.items[params.p + i];
                    let previous = &analyzer.items[params.p + i + 1];

                    let mut all_rising = true;
                    let mut all_falling = true;

                    for j in 0..params.periods.len() {
                        let current_ma = current.mas.get_by_key_index(j).get();
                        let previous_ma = previous.mas.get_by_key_index(j).get();

                        if current_ma <= previous_ma {
                            all_rising = false;
                        }
                        if current_ma >= previous_ma {
                            all_falling = false;
                        }
                    }

                    if !(all_rising || all_falling) {
                        all_parallel = false;
                        break;
                    }
                }
                all_parallel
            }
        }
        MovingAverageFilterType::NearCrossover => {
            if analyzer.items.len() <= params.p || params.periods.len() < 2 {
                false
            } else {
                let current_gap = (analyzer.items[params.p]
                    .mas
                    .get_by_key_index(first_index)
                    .get()
                    - analyzer.items[params.p]
                        .mas
                        .get_by_key_index(last_index)
                        .get())
                .abs();
                let avg_price = analyzer.items[params.p].candle.close_price();
                if avg_price == 0.0 {
                    false
                } else {
                    let gap_ratio = current_gap / avg_price;
                    gap_ratio <= params.crossover_threshold
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
            static COUNTER: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
            let timestamp = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Self {
                open,
                high,
                low,
                close,
                volume,
                datetime: Utc.timestamp_opt(timestamp, 0).unwrap(),
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
            periods: vec![5, 20],
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 첫번째 MA(5) 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_1_price_above_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 1.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 마지막 MA(20) 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_2_price_above_both_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 2.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 모든 MA 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_3_fast_ma_above_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 3.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 첫번째 MA(5)가 마지막 MA(20) 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_4_fast_ma_below_slow_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 4.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 첫번째 MA(5)가 마지막 MA(20) 아래에 있는지 확인
        assert!(result.unwrap()); // 하락 추세이므로 true
    }

    #[test]
    fn test_filter_type_5_golden_cross() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 5.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
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
            periods: vec![5, 20],
            filter_type: 6.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 가격이 두 MA 사이에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_invalid_filter_type() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 99.into(), // 유효하지 않은 필터 타입
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 유효하지 않은 필터 타입은 항상 false 반환
    }

    #[test]
    fn test_consecutive_n_condition() {
        let candles = create_test_candles();

        // 첫번째 MA가 마지막 MA 아래에 있는 경우 (하락 추세)
        // 이 테스트는 단순히 캔들 데이터와 이동평균 필터가 제대로 작동하는지 확인
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 4.into(), // 첫번째 MA가 마지막 MA 아래에 있는 경우
            consecutive_n: 1,      // 1개 조건만 확인
            p: 0,
            ..Default::default()
        };

        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        let is_passed = result.unwrap();
        println!("하락 추세 테스트 결과 (1연속): {is_passed}");
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = vec![TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0)]; // 캔들 데이터 부족
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 캔들 데이터 부족으로 false 반환
    }

    #[test]
    fn test_multiple_periods() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 10, 20],
            filter_type: 2.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 마지막 가격이 모든 MA 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_empty_periods() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![],
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 기간 목록이 비어 있으므로 false 반환
    }

    #[test]
    fn test_single_period() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![10],
            filter_type: 0.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 단일 MA만 있는 경우 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_11_reverse_arrangement() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 11.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 역배열 (단기 MA가 장기 MA 아래에 있음) 확인
        assert!(result.unwrap()); // 하락 추세이므로 true
    }

    #[test]
    fn test_filter_type_12_dead_cross() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 12.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 데드 크로스 발생 확인
        assert!(!result.unwrap()); // 단일 데이터로는 크로스 확인 불가
    }

    #[test]
    fn test_filter_type_13_ma_sideways() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![10],
            filter_type: 13.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 이동평균선이 횡보 중인지 확인
        assert!(result.unwrap()); // 실제로는 횡보로 판단됨
    }

    #[test]
    fn test_filter_type_14_strong_uptrend() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![10],
            filter_type: 14.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 강한 상승 추세 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_15_strong_downtrend() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![10],
            filter_type: 15.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 강한 하락 추세 확인
        assert!(result.unwrap()); // 하락 추세이므로 true
    }

    #[test]
    fn test_filter_type_16_price_crossing_ma() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 16.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 가격이 이동평균선들과 교차 중인지 확인
        assert!(!result.unwrap()); // 가격이 모든 MA 아래에 있으므로 false
    }

    #[test]
    fn test_filter_type_17_convergence_divergence() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 17.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 수렴 후 발산 시작 확인
        assert!(!result.unwrap()); // 충분한 데이터가 없어서 false
    }

    #[test]
    fn test_filter_type_18_divergence_convergence() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 18.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 발산 후 수렴 시작 확인
        assert!(!result.unwrap()); // 충분한 데이터가 없어서 false
    }

    #[test]
    fn test_filter_type_19_parallel_movement() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 19.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 이동평균선들이 평행 이동 중인지 확인
        assert!(result.unwrap()); // 실제로는 평행 이동으로 판단됨
    }

    #[test]
    fn test_filter_type_20_near_crossover() {
        let candles = create_test_candles();
        let params = MovingAverageParams {
            periods: vec![5, 20],
            filter_type: 20.into(),
            consecutive_n: 1,
            p: 0,
            ..Default::default()
        };
        let result = filter_moving_average("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 이동평균선들이 교차점 근처인지 확인
        assert!(!result.unwrap()); // 간격이 넓어서 false
    }
}
