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

    log::debug!(
        "코인 {} 볼린저 밴드 - 상단: {:.2}, 중간: {:.2}, 하단: {:.2}",
        coin,
        upper,
        middle,
        lower
    );

    let result = match params.filter_type {
        // 0: 가격이 상단밴드 위에 있는 경우 (과매수 상태)
        0 => analyzer.is_above_upper_band(params.consecutive_n),
        // 1: 가격이 하단밴드 아래에 있는 경우 (과매도 상태)
        1 => analyzer.is_below_lower_band(params.consecutive_n),
        // 2: 가격이 밴드 내에 있는 경우
        2 => {
            !analyzer.is_above_upper_band(params.consecutive_n)
                && !analyzer.is_below_lower_band(params.consecutive_n)
        }
        // 3: 가격이 밴드 밖에 있는 경우 (변동성 큼)
        3 => {
            analyzer.is_above_upper_band(params.consecutive_n)
                || analyzer.is_below_lower_band(params.consecutive_n)
        }
        // 4: 가격이 중간밴드보다 위에 있는 경우 (상승 추세)
        4 => analyzer.is_above_middle_band(params.consecutive_n),
        // 5: 가격이 중간밴드보다 아래에 있는 경우 (하락 추세)
        5 => analyzer.is_below_middle_band(params.consecutive_n),
        // 6: 밴드 폭이 충분히 넓은지 확인 (변동성 충분)
        6 => analyzer.is_band_width_sufficient(),
        // 7: 하단 밴드 아래에서 위로 돌파한 경우 (상승 반전 신호)
        7 => analyzer.is_break_through_lower_band_from_below(params.consecutive_n),
        _ => false,
    };

    Ok(result)
}
