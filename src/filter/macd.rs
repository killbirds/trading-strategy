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
                analyzer.is_all(|data| data.is_macd_above_signal(), params.consecutive_n)
            }
        }
        // 1: MACD 라인이 시그널 라인 아래에 있는 경우 (하락 추세)
        1 => {
            if analyzer.items.len() < params.consecutive_n {
                false
            } else {
                analyzer.is_all(|data| data.is_macd_below_signal(), params.consecutive_n)
            }
        }
        // 2: MACD 라인이 시그널 라인을 상향 돌파한 경우 (매수 신호)
        2 => analyzer.is_macd_crossed_above_signal(params.consecutive_n, 1),
        // 3: MACD 라인이 시그널 라인을 하향 돌파한 경우 (매도 신호)
        3 => analyzer.is_macd_crossed_below_signal(params.consecutive_n, 1),
        // 4: 히스토그램이 임계값보다 큰 경우 (강한 상승)
        4 => analyzer.is_histogram_above_threshold(params.threshold, params.consecutive_n),
        // 5: 히스토그램이 임계값보다 작은 경우 (강한 하락)
        5 => analyzer.is_histogram_below_threshold(params.threshold, params.consecutive_n),
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
        _ => false,
    };

    Ok(result)
}
