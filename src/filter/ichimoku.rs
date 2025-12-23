use super::{IchimokuFilterType, IchimokuParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::ichimoku_analyzer::IchimokuAnalyzer;
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
        "이치모쿠 필터 적용 - 전환선: {}, 기준선: {}, 선행스팬B: {}, 타입: {:?}, 연속성: {}",
        params.tenkan_period,
        params.kijun_period,
        params.senkou_span_b_period,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 검증
    utils::validate_period(params.tenkan_period, "Ichimoku tenkan_period")?;
    utils::validate_period(params.kijun_period, "Ichimoku kijun_period")?;
    utils::validate_period(params.senkou_span_b_period, "Ichimoku senkou_span_b_period")?;

    // 필터링 로직
    let required_length = params.senkou_span_b_period + params.kijun_period + params.consecutive_n; // 데이터 필요량
    if !utils::check_sufficient_candles(candles.len(), required_length, coin) {
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candle_store = utils::create_candle_store(candles);

    // IchimokuParams 생성
    let ichimoku_params = IndicatorIchimokuParams {
        tenkan_period: params.tenkan_period,
        kijun_period: params.kijun_period,
        senkou_period: params.senkou_span_b_period,
    };
    let indicator_params = vec![ichimoku_params];

    // IchimokuAnalyzer 생성
    let analyzer = IchimokuAnalyzer::new(&indicator_params, &candle_store);

    log::debug!("코인 {coin} 이치모쿠 분석기 생성 완료");

    let result = match params.filter_type {
        IchimokuFilterType::PriceAboveCloud => {
            analyzer.is_price_above_cloud(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::PriceBelowCloud => {
            analyzer.is_price_below_cloud(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::TenkanAboveKijun => {
            analyzer.is_tenkan_above_kijun(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::GoldenCross => {
            analyzer.is_golden_cross_signal(params.consecutive_n, 1, &ichimoku_params, params.p)
        }
        IchimokuFilterType::DeadCross => {
            analyzer.is_dead_cross_signal(params.consecutive_n, 1, &ichimoku_params, params.p)
        }
        IchimokuFilterType::CloudBreakoutUp => analyzer.is_cloud_breakout_up_signal(
            params.consecutive_n,
            1,
            &ichimoku_params,
            params.p,
        ),
        IchimokuFilterType::CloudBreakdown => {
            analyzer.is_cloud_breakdown_signal(params.consecutive_n, 1, &ichimoku_params, params.p)
        }
        IchimokuFilterType::BuySignal => {
            analyzer.is_buy_signal(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::SellSignal => {
            analyzer.is_sell_signal(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::CloudThickening => {
            analyzer.is_cloud_thickening(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::PerfectAlignment => {
            analyzer.is_price_above_cloud(&ichimoku_params, params.consecutive_n, params.p)
                && analyzer.is_tenkan_above_kijun(&ichimoku_params, params.consecutive_n, params.p)
        }
        IchimokuFilterType::PerfectReverseAlignment => {
            analyzer.is_price_below_cloud(&ichimoku_params, params.consecutive_n, params.p)
                && analyzer.is_all(
                    |data| data.is_tenkan_below_kijun(&ichimoku_params),
                    params.consecutive_n,
                    params.p,
                )
        }
        IchimokuFilterType::StrongBuySignal => {
            analyzer.is_buy_signal(&ichimoku_params, params.consecutive_n, params.p)
        }
    };

    Ok(result)
}
