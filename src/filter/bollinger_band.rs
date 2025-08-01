use super::BollingerBandParams;
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
        _ => false,
    };

    Ok(result)
}
