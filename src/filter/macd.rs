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

    log::debug!("코인 {} MACD", coin,);

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
        _ => false,
    };

    Ok(result)
}
