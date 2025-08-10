use super::MACDParams;
use crate::analyzer::AnalyzerOps;
use crate::analyzer::macd_analyzer::MACDAnalyzer;
use crate::candle_store::CandleStore;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 MACD 필터 적용
pub fn filter_macd<C: Candle + 'static>(
    coin: &str,
    params: &MACDParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "MACD 필터 적용 - 빠른 기간: {}, 느린 기간: {}, 시그널 기간: {}, 타입: {}, 연속성: {}",
        params.fast_period,
        params.slow_period,
        params.signal_period,
        params.filter_type,
        params.consecutive_n
    );

    // 필터링 로직
    let required_length = params.slow_period + params.signal_period + params.consecutive_n; // 최소 필요 캔들 수
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

    // MACDAnalyzer 생성
    let analyzer = MACDAnalyzer::new(
        params.fast_period,
        params.slow_period,
        params.signal_period,
        &candle_store,
    );

    log::debug!("코인 {coin} MACD",);

    let result = match params.filter_type {
        // 0: MACD 라인이 시그널 라인 위에 있는 경우 (상승 추세)
        0 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| data.is_macd_above_signal(),
                    params.consecutive_n,
                    params.p,
                )
            }
        }
        // 1: MACD 라인이 시그널 라인 아래에 있는 경우 (하락 추세)
        1 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(
                    |data| data.is_macd_below_signal(),
                    params.consecutive_n,
                    params.p,
                )
            }
        }
        // 2: MACD 라인이 시그널 라인을 상향 돌파한 경우 (매수 신호)
        2 => analyzer.is_macd_crossed_above_signal(params.consecutive_n, 1),
        // 3: MACD 라인이 시그널 라인을 하향 돌파한 경우 (매도 신호)
        3 => analyzer.is_macd_crossed_below_signal(params.consecutive_n, 1),
        // 4: 히스토그램이 임계값보다 큰 경우 (강한 상승)
        4 => {
            analyzer.is_histogram_above_threshold(params.threshold, params.consecutive_n, params.p)
        }
        // 5: 히스토그램이 임계값보다 작은 경우 (강한 하락)
        5 => {
            analyzer.is_histogram_below_threshold(params.threshold, params.consecutive_n, params.p)
        }
        // 6: MACD 라인이 제로라인을 상향 돌파 (강한 상승 신호)
        6 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_macd = analyzer.items[0].macd.macd_line;
                let previous_macd = analyzer.items[1].macd.macd_line;
                current_macd > 0.0 && previous_macd <= 0.0
            }
        }
        // 7: MACD 라인이 제로라인을 하향 돌파 (강한 하락 신호)
        7 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_macd = analyzer.items[0].macd.macd_line;
                let previous_macd = analyzer.items[1].macd.macd_line;
                current_macd < 0.0 && previous_macd >= 0.0
            }
        }
        // 8: 히스토그램이 양수에서 음수로 전환 (모멘텀 약화)
        8 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_hist = analyzer.items[0].macd.histogram;
                let previous_hist = analyzer.items[1].macd.histogram;
                current_hist < 0.0 && previous_hist >= 0.0
            }
        }
        // 9: 히스토그램이 음수에서 양수로 전환 (모멘텀 강화)
        9 => {
            if analyzer.items.len() < 2 {
                false
            } else {
                let current_hist = analyzer.items[0].macd.histogram;
                let previous_hist = analyzer.items[1].macd.histogram;
                current_hist > 0.0 && previous_hist <= 0.0
            }
        }
        // 10: MACD와 시그널 모두 제로라인 위에 있음 (강한 상승 추세)
        10 => {
            if analyzer.items.is_empty() {
                false
            } else {
                let current_data = &analyzer.items[0];
                current_data.macd.macd_line > 0.0 && current_data.macd.signal_line > 0.0
            }
        }
        // 11: MACD와 시그널 모두 제로라인 아래에 있음 (강한 하락 추세)
        11 => {
            if analyzer.items.is_empty() {
                false
            } else {
                let current_data = &analyzer.items[0];
                current_data.macd.macd_line < 0.0 && current_data.macd.signal_line < 0.0
            }
        }
        // 12: MACD 라인이 상승 중 (연속 상승)
        12 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut ascending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[i].macd.macd_line <= analyzer.items[i + 1].macd.macd_line {
                        ascending = false;
                        break;
                    }
                }
                ascending
            }
        }
        // 13: MACD 라인이 하락 중 (연속 하락)
        13 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut descending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[i].macd.macd_line >= analyzer.items[i + 1].macd.macd_line {
                        descending = false;
                        break;
                    }
                }
                descending
            }
        }
        // 14: 히스토그램이 확대 중 (모멘텀 강화)
        14 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut expanding = true;
                for i in 0..params.consecutive_n {
                    let current_hist = analyzer.items[i].macd.histogram.abs();
                    let previous_hist = analyzer.items[i + 1].macd.histogram.abs();
                    if current_hist <= previous_hist {
                        expanding = false;
                        break;
                    }
                }
                expanding
            }
        }
        // 15: 히스토그램이 축소 중 (모멘텀 약화)
        15 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut contracting = true;
                for i in 0..params.consecutive_n {
                    let current_hist = analyzer.items[i].macd.histogram.abs();
                    let previous_hist = analyzer.items[i + 1].macd.histogram.abs();
                    if current_hist >= previous_hist {
                        contracting = false;
                        break;
                    }
                }
                contracting
            }
        }
        // 16: MACD 다이버전스 (가격 상승, MACD 하락)
        16 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_price = analyzer.items[0].candle.close_price();
                let previous_price = analyzer.items[2].candle.close_price();
                let current_macd = analyzer.items[0].macd.macd_line;
                let previous_macd = analyzer.items[2].macd.macd_line;

                // 가격은 상승했지만 MACD는 하락 (음의 다이버전스)
                current_price > previous_price && current_macd < previous_macd
            }
        }
        // 17: MACD 컨버전스 (가격 하락, MACD 상승)
        17 => {
            if analyzer.items.len() < 3 {
                false
            } else {
                let current_price = analyzer.items[0].candle.close_price();
                let previous_price = analyzer.items[2].candle.close_price();
                let current_macd = analyzer.items[0].macd.macd_line;
                let previous_macd = analyzer.items[2].macd.macd_line;

                // 가격은 하락했지만 MACD는 상승 (양의 다이버전스)
                current_price < previous_price && current_macd > previous_macd
            }
        }
        // 18: MACD가 과매수 구간 (극도 상승)
        18 => {
            if analyzer.items.is_empty() {
                false
            } else {
                let current_macd = analyzer.items[0].macd.macd_line;
                let avg_price = analyzer.items[0].candle.close_price();
                let macd_ratio = current_macd / avg_price;

                // MACD가 가격의 2% 이상일 때 과매수로 판단
                macd_ratio >= 0.02
            }
        }
        // 19: MACD가 과매도 구간 (극도 하락)
        19 => {
            if analyzer.items.is_empty() {
                false
            } else {
                let current_macd = analyzer.items[0].macd.macd_line;
                let avg_price = analyzer.items[0].candle.close_price();
                let macd_ratio = current_macd / avg_price;

                // MACD가 가격의 -2% 이하일 때 과매도로 판단
                macd_ratio <= -0.02
            }
        }
        // 20: MACD가 횡보 중 (변화율이 임계값 이하)
        20 => {
            if analyzer.items.len() < params.consecutive_n + 1 {
                false
            } else {
                let mut sideways = true;
                for i in 0..params.consecutive_n {
                    let current_macd = analyzer.items[i].macd.macd_line;
                    let previous_macd = analyzer.items[i + 1].macd.macd_line;
                    let change_rate = (current_macd - previous_macd).abs() / previous_macd.abs();

                    // 변화율이 5% 이상이면 횡보가 아님
                    if change_rate > 0.05 {
                        sideways = false;
                        break;
                    }
                }
                sideways
            }
        }
        _ => false,
    };

    Ok(result)
}
