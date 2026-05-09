use super::{
    ChoppinessFilterType, ChoppinessParams, Result, standalone_indicator as helper, utils,
};
use crate::analyzer::choppiness_analyzer::{ChoppinessAnalyzer, ChoppinessAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_choppiness<C: Candle + 'static>(
    symbol: &str,
    params: &ChoppinessParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let required = helper::required_with_offsets(
        params.period,
        params.consecutive_n,
        params.p,
        false,
        "Choppiness required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = ChoppinessAnalyzer::new(
        candle_store,
        ChoppinessAnalyzerParams {
            period: params.period,
        },
    );
    Ok(match params.filter_type {
        ChoppinessFilterType::Trending => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.choppiness.value() <= params.trending_threshold
            })
        }
        ChoppinessFilterType::Ranging => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.choppiness.value() >= params.ranging_threshold
            })
        }
    })
}

pub(crate) fn validate_params(params: &ChoppinessParams) -> Result<()> {
    utils::validate_period(params.period, "Choppiness period")?;
    helper::validate_indicator_capacity(params.period, 2, 0, "Choppiness period")?;
    helper::validate_common(params.consecutive_n, "Choppiness consecutive_n")?;
    utils::validate_percentage_threshold(
        params.trending_threshold,
        "Choppiness trending_threshold",
    )?;
    utils::validate_percentage_threshold(params.ranging_threshold, "Choppiness ranging_threshold")
}
