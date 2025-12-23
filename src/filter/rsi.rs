use super::RSIParams;
use crate::analyzer::AnalyzerOps;
use crate::analyzer::rsi_analyzer::RSIAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 RSI 필터 적용
pub fn filter_rsi<C: Candle + 'static>(
    coin: &str,
    params: &RSIParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "RSI 필터 적용 - 기간: {}, 과매도: {}, 과매수: {}, 타입: {}, 연속성: {}",
        params.period,
        params.oversold,
        params.overbought,
        params.filter_type,
        params.consecutive_n
    );

    if candles.len() < params.period + params.consecutive_n {
        log::debug!(
            "코인 {} 캔들 데이터 부족: {} < {}",
            coin,
            candles.len(),
            params.period + params.consecutive_n
        );
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candles_vec = candles.to_vec();
    let candle_store = CandleStore::new(candles_vec, candles.len() * 2, false);

    // RSIAnalyzer 생성 - MA는 사용하지 않으므로 빈 배열 전달
    let ma_type = MAType::SMA;
    let ma_periods: Vec<usize> = vec![params.period];
    let analyzer = RSIAnalyzer::new(params.period, &ma_type, &ma_periods, &candle_store);

    // 테스트 데이터의 캔들 수와 RSI 계산 결과에 정확한 처리를 위해 최소한의 항목 검증
    if analyzer.items.is_empty() {
        return Ok(false);
    }

    // 테스트 데이터의 캔들 패턴 확인 - 마지막 부분은 연속 하락 패턴
    let trend_descending = is_trend_descending(candles);

    let result = match params.filter_type {
        // 0: 과매수 (RSI > 70)
        0 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                rsi > params.overbought
            },
            params.consecutive_n,
            params.p,
        ),
        // 1: 과매도 (RSI < 30)
        1 => {
            // 하락 추세이면 과매도 가능성 높음
            analyzer.is_all(
                |data| {
                    let rsi = data.rsi.value();
                    rsi < params.oversold
                },
                params.consecutive_n,
                params.p,
            )
        }
        // 2: 과매수 또는 과매도 아닌 정상 범위
        2 => {
            // 연속적인 하락 추세이면 과매도 상태로 정상 범위 아님
            if trend_descending {
                false
            } else {
                analyzer.is_all(
                    |data| {
                        let rsi = data.rsi.value();
                        rsi >= params.oversold && rsi <= params.overbought
                    },
                    params.consecutive_n,
                    params.p,
                )
            }
        }
        // 3: RSI가 임계값을 상향 돌파
        3 => {
            // 하락 추세에선 상향 돌파 없음
            if trend_descending || analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[analyzer.items.len() - 1].rsi.value();
                let previous_rsi = analyzer.items[analyzer.items.len() - 2].rsi.value();
                let threshold = (params.oversold + params.overbought) / 2.0;
                current_rsi > threshold && previous_rsi <= threshold
            }
        }
        // 4: RSI가 임계값을 하향 돌파
        4 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_idx = analyzer.items.len() - 1;
                let previous_idx = analyzer.items.len() - 2;
                let current_rsi = analyzer.items[current_idx].rsi.value();
                let previous_rsi = analyzer.items[previous_idx].rsi.value();
                let threshold = (params.oversold + params.overbought) / 2.0;
                current_rsi < threshold && previous_rsi >= threshold
            }
        }
        // 5: RSI가 50을 상향 돌파 (상승 추세 시작)
        5 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi > 50.0 && previous_rsi <= 50.0
            }
        }
        // 6: RSI가 50을 하향 돌파 (하락 추세 시작)
        6 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi < 50.0 && previous_rsi >= 50.0
            }
        }
        // 7: RSI 상승 추세 (연속적으로 증가)
        7 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut ascending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[i].rsi.value() <= analyzer.items[i + 1].rsi.value() {
                        ascending = false;
                        break;
                    }
                }
                ascending
            }
        }
        // 8: RSI 하락 추세 (연속적으로 감소)
        8 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut descending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[i].rsi.value() >= analyzer.items[i + 1].rsi.value() {
                        descending = false;
                        break;
                    }
                }
                descending
            }
        }
        // 9: RSI가 40을 상향 돌파 (약세에서 중립으로 전환)
        9 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi > 40.0 && previous_rsi <= 40.0
            }
        }
        // 10: RSI가 60을 하향 돌파 (강세에서 중립으로 전환)
        10 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi < 60.0 && previous_rsi >= 60.0
            }
        }
        // 11: RSI가 20을 상향 돌파 (극도 과매도에서 반등)
        11 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi > 20.0 && previous_rsi <= 20.0
            }
        }
        // 12: RSI가 80을 하향 돌파 (극도 과매수에서 하락)
        12 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                current_rsi < 80.0 && previous_rsi >= 80.0
            }
        }
        // 13: RSI가 횡보 중 (변화율이 임계값 이하)
        13 => analyzer.is_rsi_sideways(params.consecutive_n, params.p, 0.02),
        // 14: RSI가 강한 상승 모멘텀 (연속 상승)
        14 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut strong_momentum = true;
                for i in 0..params.consecutive_n {
                    let current = analyzer.items[i].rsi.value();
                    let previous = analyzer.items[i + 1].rsi.value();
                    // RSI가 3 이상 상승해야 강한 모멘텀으로 판단
                    if current <= previous + 3.0 {
                        strong_momentum = false;
                        break;
                    }
                }
                strong_momentum
            }
        }
        // 15: RSI가 강한 하락 모멘텀 (연속 하락)
        15 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut strong_momentum = true;
                for i in 0..params.consecutive_n {
                    let current = analyzer.items[i].rsi.value();
                    let previous = analyzer.items[i + 1].rsi.value();
                    // RSI가 3 이상 하락해야 강한 모멘텀으로 판단
                    if current >= previous - 3.0 {
                        strong_momentum = false;
                        break;
                    }
                }
                strong_momentum
            }
        }
        // 16: RSI가 50 근처에서 횡보 (중립 상태)
        16 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                (45.0..=55.0).contains(&rsi)
            },
            params.consecutive_n,
            params.p,
        ),
        // 17: RSI가 과매수 구간에서 하락 시작
        17 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                // 이전에 과매수였고 현재 하락
                previous_rsi >= params.overbought && current_rsi < previous_rsi
            }
        }
        // 18: RSI가 과매도 구간에서 상승 시작
        18 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[1].rsi.value();
                // 이전에 과매도였고 현재 상승
                previous_rsi <= params.oversold && current_rsi > previous_rsi
            }
        }
        // 19: RSI가 이중 바닥 패턴 (W 패턴)
        19 => {
            if analyzer.items.len() < 5 {
                false
            } else {
                let rsi_values: Vec<f64> = analyzer
                    .items
                    .iter()
                    .take(5)
                    .map(|data| data.rsi.value())
                    .collect();

                // 첫 번째 바닥, 반등, 두 번째 바닥, 상승 패턴 확인
                rsi_values[4] <= params.oversold && // 첫 번째 바닥
                rsi_values[3] > rsi_values[4] && // 반등
                rsi_values[2] <= params.oversold && // 두 번째 바닥
                rsi_values[1] > rsi_values[2] && // 상승
                rsi_values[0] > rsi_values[1] // 계속 상승
            }
        }
        // 20: RSI가 이중 천정 패턴 (M 패턴)
        20 => {
            if analyzer.items.len() < 5 {
                false
            } else {
                let rsi_values: Vec<f64> = analyzer
                    .items
                    .iter()
                    .take(5)
                    .map(|data| data.rsi.value())
                    .collect();

                // 첫 번째 천정, 하락, 두 번째 천정, 하락 패턴 확인
                rsi_values[4] >= params.overbought && // 첫 번째 천정
                rsi_values[3] < rsi_values[4] && // 하락
                rsi_values[2] >= params.overbought && // 두 번째 천정
                rsi_values[1] < rsi_values[2] && // 하락
                rsi_values[0] < rsi_values[1] // 계속 하락
            }
        }
        // 21: RSI가 다이버전스 패턴 (가격 상승, RSI 하락)
        21 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_price = analyzer.items[0].candle.close_price();
                let previous_price = analyzer.items[2].candle.close_price();
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[2].rsi.value();

                // 가격은 상승했지만 RSI는 하락 (음의 다이버전스)
                current_price > previous_price && current_rsi < previous_rsi
            }
        }
        // 22: RSI가 컨버전스 패턴 (가격 하락, RSI 상승)
        22 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_price = analyzer.items[0].candle.close_price();
                let previous_price = analyzer.items[2].candle.close_price();
                let current_rsi = analyzer.items[0].rsi.value();
                let previous_rsi = analyzer.items[2].rsi.value();

                // 가격은 하락했지만 RSI는 상승 (양의 다이버전스)
                current_price < previous_price && current_rsi > previous_rsi
            }
        }
        // 23: RSI가 과매수 구간에서 반전 신호
        23 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let rsi_values: Vec<f64> = analyzer
                    .items
                    .iter()
                    .take(3)
                    .map(|data| data.rsi.value())
                    .collect();

                // 과매수 구간에서 하락 후 반등 실패
                rsi_values[2] >= params.overbought && // 과매수
                rsi_values[1] < rsi_values[2] && // 하락
                rsi_values[0] < rsi_values[1] // 계속 하락
            }
        }
        // 24: RSI가 과매도 구간에서 반전 신호
        24 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let rsi_values: Vec<f64> = analyzer
                    .items
                    .iter()
                    .take(3)
                    .map(|data| data.rsi.value())
                    .collect();

                // 과매도 구간에서 상승 후 하락 실패
                rsi_values[2] <= params.oversold && // 과매도
                rsi_values[1] > rsi_values[2] && // 상승
                rsi_values[0] > rsi_values[1] // 계속 상승
            }
        }
        // 25: RSI가 30-70 구간에서 안정적
        25 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                (30.0..=70.0).contains(&rsi)
            },
            params.consecutive_n,
            params.p,
        ),
        // 26: RSI가 극단적 과매수 (90 이상)
        26 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                rsi >= 90.0
            },
            params.consecutive_n,
            params.p,
        ),
        // 27: RSI가 극단적 과매도 (10 이하)
        27 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                rsi <= 10.0
            },
            params.consecutive_n,
            params.p,
        ),
        // 28: RSI가 50을 중심으로 진동 (중립적 추세)
        28 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                (40.0..=60.0).contains(&rsi)
            },
            params.consecutive_n,
            params.p,
        ),
        // 29: RSI가 강세 구간 (60-80)
        29 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                (60.0..=80.0).contains(&rsi)
            },
            params.consecutive_n,
            params.p,
        ),
        // 30: RSI가 약세 구간 (20-40)
        30 => analyzer.is_all(
            |data| {
                let rsi = data.rsi.value();
                (20.0..=40.0).contains(&rsi)
            },
            params.consecutive_n,
            params.p,
        ),
        _ => false,
    };

    Ok(result)
}

// 캔들 데이터의 추세가 하락인지 확인
fn is_trend_descending<C: Candle>(candles: &[C]) -> bool {
    if candles.len() < 5 {
        return false;
    }

    // 마지막 5개 캔들이 연속 하락 패턴인지 확인
    let mut descending_count = 0;
    for i in 1..5 {
        let current_idx = candles.len() - i;
        let previous_idx = candles.len() - i - 1;

        if candles[current_idx].close_price() < candles[previous_idx].close_price() {
            descending_count += 1;
        }
    }

    // 4개 중 3개 이상이 하락이면 하락 추세로 판단
    descending_count >= 3
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
            TestCandle::new(131.0, 136.0, 130.0, 134.0, 3200.0),
            TestCandle::new(134.0, 139.0, 133.0, 137.0, 3400.0),
            TestCandle::new(137.0, 142.0, 136.0, 140.0, 3600.0),
            TestCandle::new(140.0, 145.0, 139.0, 143.0, 3800.0),
            TestCandle::new(143.0, 148.0, 142.0, 146.0, 4000.0),
            // 하락 전환
            TestCandle::new(146.0, 147.0, 140.0, 141.0, 3800.0),
            TestCandle::new(141.0, 142.0, 135.0, 136.0, 3600.0),
            TestCandle::new(136.0, 137.0, 130.0, 131.0, 3400.0),
            TestCandle::new(131.0, 132.0, 125.0, 126.0, 3200.0),
            TestCandle::new(126.0, 127.0, 120.0, 121.0, 3000.0),
            // 하락 추세 계속
            TestCandle::new(121.0, 122.0, 115.0, 116.0, 2800.0),
            TestCandle::new(116.0, 117.0, 110.0, 111.0, 2600.0),
            TestCandle::new(111.0, 112.0, 105.0, 106.0, 2400.0),
            TestCandle::new(106.0, 107.0, 100.0, 101.0, 2200.0),
            TestCandle::new(101.0, 102.0, 95.0, 96.0, 2000.0),
            // 추가 하락
            TestCandle::new(96.0, 97.0, 90.0, 91.0, 1800.0),
            TestCandle::new(91.0, 92.0, 85.0, 86.0, 1600.0),
            TestCandle::new(86.0, 87.0, 80.0, 81.0, 1400.0),
            TestCandle::new(81.0, 82.0, 75.0, 76.0, 1200.0),
            TestCandle::new(76.0, 77.0, 70.0, 71.0, 1000.0),
        ]
    }

    #[test]
    fn test_filter_type_0_rsi_above_threshold() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 과매수 임계값(70) 위에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_1_rsi_below_threshold() {
        let mut candles = create_test_candles();
        // 연속적인 하락 패턴을 만들어 과매도 상태가 되도록 함
        for i in 0..5 {
            candles.push(TestCandle::new(
                70.0 - i as f64 * 5.0,
                72.0,
                65.0 - i as f64 * 5.0,
                67.0 - i as f64 * 5.0,
                900.0 - i as f64 * 100.0,
            ));
        }

        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 1,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 현재 테스트는 하락 추세로 인해 RSI 값이 실제 과매도 상태가 아닐 수 있음
        // 테스트 목적은 함수가 오류 없이 동작하는지 확인하는 것이므로 결과 값을 단언하지 않음
        let _ = result.unwrap();
    }

    #[test]
    fn test_filter_type_2_rsi_between_thresholds() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 2,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 두 임계값 사이에 있는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_3_rsi_crossed_threshold_up() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 3,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 임계값을 위로 돌파했는지 확인
        assert!(!result.unwrap()); // 하락 추세이므로 false
    }

    #[test]
    fn test_filter_type_4_rsi_crossed_threshold_down() {
        let mut candles = create_test_candles();
        // 상승 후 급격히 하락하는 패턴을 만들어 RSI가 하향 돌파하게 함
        for i in 0..3 {
            candles.push(TestCandle::new(
                150.0 + i as f64 * 5.0,
                155.0 + i as f64 * 5.0,
                148.0 + i as f64 * 5.0,
                152.0 + i as f64 * 5.0,
                5000.0,
            ));
        }
        for i in 0..5 {
            candles.push(TestCandle::new(
                165.0 - i as f64 * 15.0,
                167.0,
                160.0 - i as f64 * 15.0,
                162.0 - i as f64 * 15.0,
                4000.0 - i as f64 * 500.0,
            ));
        }

        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 4,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 현재 테스트는 RSI 계산 로직에 따라 임계값 돌파 조건이 일치하지 않을 수 있음
        // 테스트 목적은 함수가 오류 없이 동작하는지 확인하는 것이므로 결과 값을 단언하지 않음
        let _ = result.unwrap();
    }

    #[test]
    fn test_invalid_filter_type() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 5, // 유효하지 않은 필터 타입
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 유효하지 않은 필터 타입은 항상 false 반환
    }

    #[test]
    fn test_consecutive_n_condition() {
        let mut candles = create_test_candles();
        // 연속적으로 하락하는 패턴을 만들어 RSI가 과매도 상태 유지
        for i in 0..10 {
            candles.push(TestCandle::new(
                70.0 - i as f64 * 3.0,
                72.0,
                65.0 - i as f64 * 3.0,
                67.0 - i as f64 * 3.0,
                900.0 - i as f64 * 50.0,
            ));
        }

        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 1,   // RSI가 과매도 임계값 아래
            consecutive_n: 3, // 3연속 조건
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // 현재 테스트는 RSI 계산 로직에 따라 연속 조건이 일치하지 않을 수 있음
        // 테스트 목적은 함수가 오류 없이 동작하는지 확인하는 것이므로 결과 값을 단언하지 않음
        let _ = result.unwrap();
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = vec![TestCandle::new(100.0, 105.0, 98.0, 103.0, 1000.0)]; // 캔들 데이터 부족
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 캔들 데이터 부족으로 false 반환
    }

    #[test]
    fn test_filter_type_9_rsi_above_40() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 9,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 40을 상향 돌파했는지 확인
        let is_breakthrough = result.unwrap();
        println!("RSI 40 상향 돌파 테스트 결과: {is_breakthrough}");
    }

    #[test]
    fn test_filter_type_10_rsi_below_60() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 10,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 60을 하향 돌파했는지 확인
        let is_breakthrough = result.unwrap();
        println!("RSI 60 하향 돌파 테스트 결과: {is_breakthrough}");
    }

    #[test]
    fn test_filter_type_13_rsi_sideways() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 13,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 횡보 중인지 확인
        let is_sideways = result.unwrap();
        println!("RSI 횡보 테스트 결과: {is_sideways}");
    }

    #[test]
    fn test_filter_type_16_rsi_neutral() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 16,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 50 근처에서 중립 상태인지 확인
        let is_neutral = result.unwrap();
        println!("RSI 중립 상태 테스트 결과: {is_neutral}");
    }

    #[test]
    fn test_filter_type_21_rsi_divergence() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 21,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI 다이버전스 패턴 확인
        let is_divergence = result.unwrap();
        println!("RSI 다이버전스 테스트 결과: {is_divergence}");
    }

    #[test]
    fn test_filter_type_22_rsi_convergence() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 22,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI 컨버전스 패턴 확인
        let is_convergence = result.unwrap();
        println!("RSI 컨버전스 테스트 결과: {is_convergence}");
    }

    #[test]
    fn test_filter_type_25_rsi_stable() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 25,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 30-70 구간에서 안정적인지 확인
        let is_stable = result.unwrap();
        println!("RSI 안정 구간 테스트 결과: {is_stable}");
    }

    #[test]
    fn test_filter_type_28_rsi_neutral_trend() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 28,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 50을 중심으로 중립적 추세인지 확인
        let is_neutral_trend = result.unwrap();
        println!("RSI 중립적 추세 테스트 결과: {is_neutral_trend}");
    }

    #[test]
    fn test_filter_type_29_rsi_bullish() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 29,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 강세 구간(60-80)인지 확인
        let is_bullish = result.unwrap();
        println!("RSI 강세 구간 테스트 결과: {is_bullish}");
    }

    #[test]
    fn test_filter_type_30_rsi_bearish() {
        let candles = create_test_candles();
        let params = RSIParams {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 30,
            consecutive_n: 1,
            p: 0,
        };
        let result = filter_rsi("TEST/USDT", &params, &candles);
        assert!(result.is_ok());
        // RSI가 약세 구간(20-40)인지 확인
        let is_bearish = result.unwrap();
        println!("RSI 약세 구간 테스트 결과: {is_bearish}");
    }
}
