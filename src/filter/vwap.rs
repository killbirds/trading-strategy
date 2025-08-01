use super::VWAPParams;
use crate::analyzer::vwap_analyzer::VWAPAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::vwap::VWAPParams as IndicatorVWAPParams;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 VWAP 필터 적용
pub fn filter_vwap<C: Candle + 'static>(
    coin: &str,
    params: &VWAPParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "VWAP 필터 적용 - 기간: {}, 타입: {}, 연속성: {}, 임계값: {:.2}%",
        params.period,
        params.filter_type,
        params.consecutive_n,
        params.threshold * 100.0
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

    // VWAP 매개변수 설정
    let vwap_params = IndicatorVWAPParams {
        period: params.period,
    };

    // VWAPAnalyzer 생성
    let analyzer = VWAPAnalyzer::new(&[vwap_params], &candle_store);

    log::debug!("코인 {coin} VWAP");

    let result = match params.filter_type {
        // 0: 가격이 VWAP 위에 있는 경우 (상승 추세)
        0 => analyzer.is_price_above_vwap(&vwap_params, params.consecutive_n, params.p),
        // 1: 가격이 VWAP 아래에 있는 경우 (하락 추세)
        1 => analyzer.is_price_below_vwap(&vwap_params, params.consecutive_n, params.p),
        // 2: 가격이 VWAP의 임계값 이내에 있는 경우 (중립 추세)
        2 => analyzer.is_price_near_vwap(
            params.consecutive_n,
            &vwap_params,
            params.threshold,
            params.p,
        ),
        // 3: VWAP 상향 돌파 (매수 신호)
        3 => analyzer.is_vwap_breakout_up(&vwap_params),
        // 4: VWAP 하향 돌파 (매도 신호)
        4 => analyzer.is_vwap_breakdown(&vwap_params),
        // 5: VWAP 리바운드 (반등 신호)
        5 => analyzer.is_vwap_rebound(&vwap_params, params.threshold),
        // 6: VWAP와 가격 간격 확대 (추세 강화)
        6 => analyzer.is_diverging_from_vwap(&vwap_params, params.consecutive_n),
        // 7: VWAP와 가격 간격 축소 (추세 약화)
        7 => analyzer.is_converging_to_vwap(&vwap_params, params.consecutive_n),
        // 8: 고가와 저가 모두 VWAP 위에 있음 (강한 상승) - 간소화
        8 => analyzer.is_price_above_vwap(&vwap_params, params.consecutive_n, params.p),
        // 9: 고가와 저가 모두 VWAP 아래에 있음 (강한 하락) - 간소화
        9 => analyzer.is_price_below_vwap(&vwap_params, params.consecutive_n, params.p),
        // 10: VWAP와 가격 간격 확대 (추세 강화) - 기존 메서드 재사용
        10 => analyzer.is_diverging_from_vwap(&vwap_params, params.consecutive_n),
        // 11: VWAP와 가격 간격 축소 (추세 약화) - 기존 메서드 재사용
        11 => analyzer.is_converging_to_vwap(&vwap_params, params.consecutive_n),
        _ => false,
    };

    Ok(result)
}
