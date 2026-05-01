use super::Result;
use super::{VWAPFilterType, VWAPParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::vwap_analyzer::VWAPAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::vwap::VWAPParams as IndicatorVWAPParams;
use trading_chart::Candle;

/// 개별 코인에 대한 VWAP 필터 적용
pub(crate) fn filter_vwap<C: Candle + 'static>(
    coin: &str,
    params: &VWAPParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    log::debug!(
        "VWAP 필터 적용 - 기간: {}, 타입: {:?}, 연속성: {}, 임계값: {:.2}%",
        params.period,
        params.filter_type,
        params.consecutive_n,
        params.threshold * 100.0
    );

    // 파라미터 검증
    utils::validate_period(params.period, "VWAP")?;

    // 경계 조건 체크
    let required_length = params.period + params.consecutive_n;
    if !utils::check_sufficient_candles(candle_store.len(), required_length, coin) {
        return Ok(false);
    }

    // VWAP 매개변수 설정
    let vwap_params = IndicatorVWAPParams {
        period: params.period,
    };

    // VWAPAnalyzer 생성
    let analyzer = VWAPAnalyzer::new(&[vwap_params], candle_store);

    log::debug!("코인 {coin} VWAP 분석기 생성 완료");

    let result = match params.filter_type {
        VWAPFilterType::PriceAboveVWAP => analyzer.is_all(
            |data| data.vwaps.get(&vwap_params).is_price_above(current_price),
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::PriceBelowVWAP => analyzer.is_all(
            |data| data.vwaps.get(&vwap_params).is_price_below(current_price),
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::PriceNearVWAP => analyzer.is_all(
            |data| {
                data.vwaps
                    .get(&vwap_params)
                    .price_to_vwap_percent(current_price)
                    .abs()
                    < params.threshold
            },
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::VWAPBreakoutUp => analyzer.is_break_through_by_satisfying(
            |data| data.vwaps.get(&vwap_params).is_price_above(current_price),
            params.consecutive_n,
            1,
            params.p,
        ),
        VWAPFilterType::VWAPBreakdown => analyzer.is_break_through_by_satisfying(
            |data| data.vwaps.get(&vwap_params).is_price_below(current_price),
            params.consecutive_n,
            1,
            params.p,
        ),
        VWAPFilterType::VWAPRebound => is_vwap_rebound_with_current_price(
            &analyzer,
            &vwap_params,
            current_price,
            params.threshold,
            params.p,
        ),
        VWAPFilterType::DivergingFromVWAP => is_vwap_distance_diverging(
            &analyzer,
            &vwap_params,
            current_price,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::ConvergingToVWAP => is_vwap_distance_converging(
            &analyzer,
            &vwap_params,
            current_price,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::StrongUptrend => analyzer.is_all(
            |data| data.vwaps.get(&vwap_params).is_price_above(current_price),
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::StrongDowntrend => analyzer.is_all(
            |data| data.vwaps.get(&vwap_params).is_price_below(current_price),
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::TrendStrengthening => is_vwap_distance_diverging(
            &analyzer,
            &vwap_params,
            current_price,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::TrendWeakening => is_vwap_distance_converging(
            &analyzer,
            &vwap_params,
            current_price,
            params.consecutive_n,
            params.p,
        ),
    };

    Ok(result)
}

fn vwap_percent<C: Candle>(
    analyzer: &VWAPAnalyzer<C>,
    params: &IndicatorVWAPParams,
    current_price: f64,
    index: usize,
) -> Option<f64> {
    analyzer
        .items
        .get(index)
        .map(|data| data.vwaps.get(params).price_to_vwap_percent(current_price))
}

fn is_vwap_rebound_with_current_price<C: Candle>(
    analyzer: &VWAPAnalyzer<C>,
    params: &IndicatorVWAPParams,
    current_price: f64,
    threshold: f64,
    p: usize,
) -> bool {
    let Some(current_percent) = vwap_percent(analyzer, params, current_price, p) else {
        return false;
    };
    let Some(previous_percent) = vwap_percent(analyzer, params, current_price, p + 1) else {
        return false;
    };
    let Some(more_previous_percent) = vwap_percent(analyzer, params, current_price, p + 2) else {
        return false;
    };

    let up_rebound = current_percent > previous_percent
        && previous_percent.abs() < threshold
        && more_previous_percent < previous_percent;
    let down_rebound = current_percent < previous_percent
        && previous_percent.abs() < threshold
        && more_previous_percent > previous_percent;

    up_rebound || down_rebound
}

fn is_vwap_distance_diverging<C: Candle>(
    analyzer: &VWAPAnalyzer<C>,
    params: &IndicatorVWAPParams,
    current_price: f64,
    n: usize,
    p: usize,
) -> bool {
    if n < 2 || analyzer.items.len() < p + n + 1 {
        return false;
    }

    (0..n - 1).all(|i| {
        let current = vwap_percent(analyzer, params, current_price, p + i)
            .map(f64::abs)
            .unwrap_or_default();
        let next = vwap_percent(analyzer, params, current_price, p + i + 1)
            .map(f64::abs)
            .unwrap_or_default();
        current > next
    })
}

fn is_vwap_distance_converging<C: Candle>(
    analyzer: &VWAPAnalyzer<C>,
    params: &IndicatorVWAPParams,
    current_price: f64,
    n: usize,
    p: usize,
) -> bool {
    if n < 2 || analyzer.items.len() < p + n + 1 {
        return false;
    }

    (0..n - 1).all(|i| {
        let current = vwap_percent(analyzer, params, current_price, p + i)
            .map(f64::abs)
            .unwrap_or_default();
        let next = vwap_percent(analyzer, params, current_price, p + i + 1)
            .map(f64::abs)
            .unwrap_or_default();
        current < next
    })
}
