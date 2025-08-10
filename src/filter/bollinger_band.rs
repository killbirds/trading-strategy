use super::BollingerBandParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use crate::candle_store::CandleStore;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 볼린저 밴드 필터 적용
pub fn filter_bollinger_band<C: Candle + 'static>(
    coin: &str,
    params: &BollingerBandParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "볼린저 밴드 필터 적용 - 기간: {}, 편차 배수: {}, 타입: {}, 연속성: {}",
        params.period,
        params.dev_mult,
        params.filter_type,
        params.consecutive_n
    );

    if candles.len() < params.period {
        log::debug!(
            "코인 {} 캔들 데이터 부족: {} < {}",
            coin,
            candles.len(),
            params.period
        );
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candles_vec = candles.to_vec();
    let candle_store = CandleStore::new(candles_vec, candles.len() * 2, false);

    // BBandAnalyzer 생성
    let analyzer = BBandAnalyzer::new(params.period, params.dev_mult, &candle_store);

    // 기존 볼린저 밴드 계산 결과도 가져옴 (로깅용)
    let (lower, middle, upper) = analyzer.get_bband();

    log::debug!("코인 {coin} 볼린저 밴드 - 상단: {upper:.2}, 중간: {middle:.2}, 하단: {lower:.2}");

    let result = match params.filter_type {
        // 0: 가격이 상단밴드 위에 있는 경우 (과매수 상태)
        0 => analyzer.is_above_upper_band(params.consecutive_n, params.p),
        // 1: 가격이 하단밴드 아래에 있는 경우 (과매도 상태)
        1 => analyzer.is_below_lower_band(params.consecutive_n, params.p),
        // 2: 가격이 밴드 내에 있는 경우
        2 => {
            !analyzer.is_above_upper_band(params.consecutive_n, params.p)
                && !analyzer.is_below_lower_band(params.consecutive_n, params.p)
        }
        // 3: 가격이 밴드 밖에 있는 경우 (변동성 큼)
        3 => {
            analyzer.is_above_upper_band(params.consecutive_n, params.p)
                || analyzer.is_below_lower_band(params.consecutive_n, params.p)
        }
        // 4: 가격이 중간밴드보다 위에 있는 경우 (상승 추세)
        4 => analyzer.is_above_middle_band(params.consecutive_n, params.p),
        // 5: 가격이 중간밴드보다 아래에 있는 경우 (하락 추세)
        5 => analyzer.is_below_middle_band(params.consecutive_n, params.p),
        // 6: 밴드 폭이 충분히 넓은지 확인 (변동성 충분)
        6 => analyzer.is_band_width_sufficient(params.p),
        // 7: 하단 밴드 아래에서 위로 돌파한 경우 (상승 반전 신호)
        7 => analyzer.is_break_through_lower_band_from_below(params.consecutive_n, params.p),
        // 8: 스퀴즈 돌파 - 밴드가 좁아진 후 상위선을 돌파하는 경우
        8 => analyzer.is_squeeze_breakout_with_close_above_upper(5),
        // 9: 향상된 스퀴즈 돌파 - 밴드가 좁아지다가 좁은 상태를 유지한 후 상위선을 돌파
        9 => analyzer.is_enhanced_squeeze_breakout_with_close_above_upper(3, 2, 0.02),
        // 10: 스퀴즈 상태 확인 - 밴드 폭이 좁은 상태
        10 => analyzer.is_band_width_squeeze(params.consecutive_n, 0.02, params.p),
        // 11: 밴드 폭 좁아지는 중 (스퀴즈 진행 중)
        11 => analyzer.is_band_width_narrowing(params.consecutive_n),
        // 12: 스퀴즈 상태에서 확장 시작 (변동성 증가 시작)
        12 => analyzer.is_squeeze_expansion_start(0.02),
        // 13: 가격이 상단 밴드에서 아래로 돌파한 경우 (과매수 해제)
        13 => analyzer.is_break_through_upper_band_from_below(params.consecutive_n, params.p),
        // 14: 가격이 하단 밴드에서 위로 돌파한 경우 (과매도 해제)
        14 => analyzer.is_break_through_lower_band_from_below(params.consecutive_n, params.p),
        // 15: 밴드 폭이 확장 중 (변동성 증가)
        15 => !analyzer.is_band_width_narrowing(params.consecutive_n),
        // 16: 가격이 중간 밴드 근처에서 횡보 (중립 상태)
        16 => analyzer.is_middle_band_sideways(params.consecutive_n, params.p, 0.02),
        // 17: 상단 밴드가 횡보 중 (저항선 형성)
        17 => analyzer.is_upper_band_sideways(params.consecutive_n, params.p, 0.02),
        // 18: 하단 밴드가 횡보 중 (지지선 형성)
        18 => analyzer.is_lower_band_sideways(params.consecutive_n, params.p, 0.02),
        // 19: 밴드 폭이 횡보 중 (변동성 안정)
        19 => analyzer.is_band_width_sideways(params.consecutive_n, params.p, 0.02),
        // 20: 가격이 상단 밴드에 터치 후 하락 (저항선 테스트)
        20 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current = &analyzer.items[0];
                let previous = &analyzer.items[1];

                // 이전에 상단 밴드에 터치했고 현재 하락
                previous.candle.close_price() >= previous.bband.upper() * 0.99
                    && current.candle.close_price() < current.bband.upper()
            }
        }
        // 21: 가격이 하단 밴드에 터치 후 상승 (지지선 테스트)
        21 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current = &analyzer.items[0];
                let previous = &analyzer.items[1];

                // 이전에 하단 밴드에 터치했고 현재 상승
                previous.candle.close_price() <= previous.bband.lower() * 1.01
                    && current.candle.close_price() > current.bband.lower()
            }
        }
        // 22: 밴드 폭이 임계값을 돌파 (변동성 급증)
        22 => analyzer.is_band_width_threshold_breakthrough(
            params.consecutive_n,
            1,
            0.05, // 5% 임계값
            params.p,
        ),
        // 23: 가격이 밴드 중앙에서 상단으로 이동 중 (상승 모멘텀)
        23 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current = &analyzer.items[0];
                let previous = &analyzer.items[1];

                // 이전에는 중간 밴드 근처, 현재는 상단 밴드 근처
                let prev_middle_dist =
                    (previous.candle.close_price() - previous.bband.middle()).abs();
                let current_upper_dist =
                    (current.candle.close_price() - current.bband.upper()).abs();
                let band_width = previous.bband.upper() - previous.bband.lower();

                prev_middle_dist < band_width * 0.1 && current_upper_dist < band_width * 0.1
            }
        }
        // 24: 가격이 밴드 중앙에서 하단으로 이동 중 (하락 모멘텀)
        24 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current = &analyzer.items[0];
                let previous = &analyzer.items[1];

                // 이전에는 중간 밴드 근처, 현재는 하단 밴드 근처
                let prev_middle_dist =
                    (previous.candle.close_price() - previous.bband.middle()).abs();
                let current_lower_dist =
                    (current.candle.close_price() - current.bband.lower()).abs();
                let band_width = previous.bband.upper() - previous.bband.lower();

                prev_middle_dist < band_width * 0.1 && current_lower_dist < band_width * 0.1
            }
        }
        // 25: 밴드가 수렴 후 발산 시작 (변동성 증가 전조)
        25 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_width =
                    analyzer.items[0].bband.upper() - analyzer.items[0].bband.lower();
                let prev_width = analyzer.items[1].bband.upper() - analyzer.items[1].bband.lower();
                let prev_prev_width =
                    analyzer.items[2].bband.upper() - analyzer.items[2].bband.lower();

                // 이전에 수렴했다가 현재 발산하기 시작
                prev_width < prev_prev_width && current_width > prev_width
            }
        }
        // 26: 밴드가 발산 후 수렴 시작 (변동성 감소 전조)
        26 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_width =
                    analyzer.items[0].bband.upper() - analyzer.items[0].bband.lower();
                let prev_width = analyzer.items[1].bband.upper() - analyzer.items[1].bband.lower();
                let prev_prev_width =
                    analyzer.items[2].bband.upper() - analyzer.items[2].bband.lower();

                // 이전에 발산했다가 현재 수렴하기 시작
                prev_width > prev_prev_width && current_width < prev_width
            }
        }
        // 27: 가격이 밴드 내에서 상단으로 이동 중 (상승 압력)
        27 => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let middle = data.bband.middle();
                let upper = data.bband.upper();

                // 가격이 중간 밴드와 상단 밴드 사이에 있고, 중간보다 위에 있음
                price > middle && price < upper
            },
            params.consecutive_n,
            params.p,
        ),
        // 28: 가격이 밴드 내에서 하단으로 이동 중 (하락 압력)
        28 => analyzer.is_all(
            |data| {
                let price = data.candle.close_price();
                let middle = data.bband.middle();
                let lower = data.bband.lower();

                // 가격이 하단 밴드와 중간 밴드 사이에 있고, 중간보다 아래에 있음
                price < middle && price > lower
            },
            params.consecutive_n,
            params.p,
        ),
        // 29: 밴드 폭이 평균 대비 좁음 (저변동성)
        29 => {
            let current_width = analyzer.items[0].bband.upper() - analyzer.items[0].bband.lower();
            let avg_price = analyzer.items[0].candle.close_price();
            let width_ratio = current_width / avg_price;

            // 밴드 폭이 가격의 2% 이하일 때 저변동성으로 판단
            width_ratio <= 0.02
        }
        // 30: 밴드 폭이 평균 대비 넓음 (고변동성)
        30 => {
            let current_width = analyzer.items[0].bband.upper() - analyzer.items[0].bband.lower();
            let avg_price = analyzer.items[0].candle.close_price();
            let width_ratio = current_width / avg_price;

            // 밴드 폭이 가격의 5% 이상일 때 고변동성으로 판단
            width_ratio >= 0.05
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
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 1,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 13,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 14,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 15,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 16,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 20,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 21,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 29,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 30,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 99, // 유효하지 않은 필터 타입
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_bollinger_band("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 유효하지 않은 필터 타입은 항상 false 반환
    }
}
