use super::{AroonFilterType, AroonParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::aroon_analyzer::{AroonAnalyzer, AroonAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_aroon<C: Candle + 'static>(
    symbol: &str,
    params: &AroonParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let required = helper::required_with_offsets(
        params.period,
        params.consecutive_n,
        params.p,
        false,
        "Aroon required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = AroonAnalyzer::new(
        candle_store,
        AroonAnalyzerParams {
            period: params.period,
        },
    );
    Ok(match params.filter_type {
        AroonFilterType::BullishTrend => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.aroon.up() >= params.strong_threshold
                    && data.aroon.down() <= params.weak_threshold
            })
        }
        AroonFilterType::BearishTrend => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.aroon.down() >= params.strong_threshold
                    && data.aroon.up() <= params.weak_threshold
            })
        }
    })
}

pub(crate) fn validate_params(params: &AroonParams) -> Result<()> {
    utils::validate_period(params.period, "Aroon period")?;
    helper::validate_indicator_capacity(params.period, 2, 0, "Aroon period")?;
    helper::validate_common(params.consecutive_n, "Aroon consecutive_n")?;
    utils::validate_percentage_threshold(params.strong_threshold, "Aroon strong_threshold")?;
    utils::validate_percentage_threshold(params.weak_threshold, "Aroon weak_threshold")
}
