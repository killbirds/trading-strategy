use super::{VWAPFilterType, VWAPParams, utils};
use crate::analyzer::vwap_analyzer::VWAPAnalyzer;
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
    if !utils::check_sufficient_candles(candles.len(), required_length, coin) {
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candle_store = utils::create_candle_store(candles);

    // VWAP 매개변수 설정
    let vwap_params = IndicatorVWAPParams {
        period: params.period,
    };

    // VWAPAnalyzer 생성
    let analyzer = VWAPAnalyzer::new(&[vwap_params], &candle_store);

    log::debug!("코인 {coin} VWAP 분석기 생성 완료");

    let result = match params.filter_type {
        VWAPFilterType::PriceAboveVWAP => {
            analyzer.is_price_above_vwap(&vwap_params, params.consecutive_n, params.p)
        }
        VWAPFilterType::PriceBelowVWAP => {
            analyzer.is_price_below_vwap(&vwap_params, params.consecutive_n, params.p)
        }
        VWAPFilterType::PriceNearVWAP => analyzer.is_price_near_vwap(
            params.consecutive_n,
            &vwap_params,
            params.threshold,
            params.p,
        ),
        VWAPFilterType::VWAPBreakoutUp => {
            analyzer.is_vwap_breakout_up_signal(params.consecutive_n, 1, &vwap_params, params.p)
        }
        VWAPFilterType::VWAPBreakdown => {
            analyzer.is_vwap_breakdown_signal(params.consecutive_n, 1, &vwap_params, params.p)
        }
        VWAPFilterType::VWAPRebound => analyzer.is_vwap_rebound_signal(
            params.consecutive_n,
            1,
            &vwap_params,
            params.threshold,
            params.p,
        ),
        VWAPFilterType::DivergingFromVWAP => analyzer.is_diverging_from_vwap_signal(
            params.consecutive_n,
            1,
            &vwap_params,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::ConvergingToVWAP => analyzer.is_converging_to_vwap_signal(
            params.consecutive_n,
            1,
            &vwap_params,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::StrongUptrend => {
            analyzer.is_price_above_vwap(&vwap_params, params.consecutive_n, params.p)
        }
        VWAPFilterType::StrongDowntrend => {
            analyzer.is_price_below_vwap(&vwap_params, params.consecutive_n, params.p)
        }
        VWAPFilterType::TrendStrengthening => analyzer.is_diverging_from_vwap_signal(
            params.consecutive_n,
            1,
            &vwap_params,
            params.consecutive_n,
            params.p,
        ),
        VWAPFilterType::TrendWeakening => analyzer.is_converging_to_vwap_signal(
            params.consecutive_n,
            1,
            &vwap_params,
            params.consecutive_n,
            params.p,
        ),
    };

    Ok(result)
}
