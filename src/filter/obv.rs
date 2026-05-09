use super::{OBVFilterType, OBVParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::obv_analyzer::OBVAnalyzer;
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_obv<C: Candle + 'static>(
    symbol: &str,
    params: &OBVParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let required = helper::required_with_offsets(
        1,
        params.consecutive_n,
        params.p,
        true,
        "OBV required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = OBVAnalyzer::new(candle_store);
    Ok(match params.filter_type {
        OBVFilterType::Rising => {
            helper::matches_rising(&analyzer, params.consecutive_n, params.p, |data| {
                data.obv.value()
            })
        }
        OBVFilterType::Falling => {
            helper::matches_falling(&analyzer, params.consecutive_n, params.p, |data| {
                data.obv.value()
            })
        }
    })
}

pub(crate) fn validate_params(params: &OBVParams) -> Result<()> {
    helper::validate_common(params.consecutive_n, "OBV consecutive_n")
}
