use super::IchimokuParams;
use crate::analyzer::ichimoku_analyzer::IchimokuAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ichimoku::IchimokuParams as IndicatorIchimokuParams;
use anyhow::Result;
use trading_chart::Candle;

/// 이치모쿠 계산 결과 구조체
#[derive(Debug, Clone)]
pub struct IchimokuValues {
    pub tenkan: f64,        // 전환선
    pub kijun: f64,         // 기준선
    pub senkou_span_a: f64, // 선행스팬 A
    pub senkou_span_b: f64, // 선행스팬 B
}

/// 개별 코인에 대한 이치모쿠 필터 적용
pub fn filter_ichimoku<C: Candle + 'static>(
    coin: &str,
    params: &IchimokuParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "이치모쿠 필터 적용 - 전환선: {}, 기준선: {}, 선행스팬B: {}, 타입: {}, 연속성: {}",
        params.tenkan_period,
        params.kijun_period,
        params.senkou_span_b_period,
        params.filter_type,
        params.consecutive_n
    );

    // 필터링 로직
    let required_length = params.senkou_span_b_period + params.kijun_period + params.consecutive_n; // 데이터 필요량
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

    // IchimokuParams 생성
    let ichimoku_params = IndicatorIchimokuParams {
        tenkan_period: params.tenkan_period,
        kijun_period: params.kijun_period,
        senkou_period: params.senkou_span_b_period,
    };
    let indicator_params = vec![ichimoku_params];

    // IchimokuAnalyzer 생성
    let analyzer = IchimokuAnalyzer::new(&indicator_params, &candle_store);

    log::debug!("코인 {} 이치모쿠", coin);

    let result = match params.filter_type {
        // 0: 가격이 구름대 위에 있는 경우 (상승 추세)
        0 => analyzer.is_price_above_cloud(&ichimoku_params, params.consecutive_n),
        // 1: 가격이 구름대 아래에 있는 경우 (하락 추세)
        1 => analyzer.is_price_below_cloud(&ichimoku_params, params.consecutive_n),
        // 2: 전환선이 기준선 위에 있는 경우 (골든 크로스)
        2 => analyzer.is_tenkan_above_kijun(&ichimoku_params, params.consecutive_n),
        // 3: 골든 크로스 발생 - 전환선이 기준선을 상향 돌파
        3 => analyzer.is_golden_cross(&ichimoku_params),
        // 4: 데드 크로스 발생 - 전환선이 기준선을 하향 돌파
        4 => analyzer.is_dead_cross(&ichimoku_params),
        // 5: 구름 돌파 - 가격이 구름을 상향 돌파
        5 => analyzer.is_cloud_breakout_up(&ichimoku_params),
        // 6: 구름 붕괴 - 가격이 구름을 하향 돌파
        6 => analyzer.is_cloud_breakdown(&ichimoku_params),
        // 7: 매수 신호 - 강한 상승 트렌드
        7 => analyzer.is_buy_signal(&ichimoku_params, params.consecutive_n),
        // 8: 매도 신호 - 강한 하락 트렌드
        8 => analyzer.is_sell_signal(&ichimoku_params, params.consecutive_n),
        // 9: 구름이 두꺼워지는 추세
        9 => analyzer.is_cloud_thickening(&ichimoku_params, params.consecutive_n),
        _ => false,
    };

    Ok(result)
}
