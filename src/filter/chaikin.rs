use super::{ChaikinFilterType, ChaikinParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::chaikin_analyzer::{ChaikinAnalyzer, ChaikinAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_chaikin<C: Candle + 'static>(
    symbol: &str,
    params: &ChaikinParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let base = params.cmf_period.max(params.slow_period);
    let required = helper::required_with_offsets(
        base,
        params.consecutive_n,
        params.p,
        false,
        "Chaikin required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = ChaikinAnalyzer::new(
        candle_store,
        ChaikinAnalyzerParams {
            cmf_period: params.cmf_period,
            fast_period: params.fast_period,
            slow_period: params.slow_period,
        },
    );
    Ok(match params.filter_type {
        ChaikinFilterType::CMFPositive => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.chaikin.cmf() >= params.cmf_threshold
            })
        }
        ChaikinFilterType::CMFNegative => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.chaikin.cmf() <= -params.cmf_threshold
            })
        }
        ChaikinFilterType::ADOSCPositive => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.chaikin.adosc() > 0.0
            })
        }
        ChaikinFilterType::ADOSCNegative => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.chaikin.adosc() < 0.0
            })
        }
    })
}

pub(crate) fn validate_params(params: &ChaikinParams) -> Result<()> {
    utils::validate_period(params.cmf_period, "Chaikin cmf_period")?;
    utils::validate_period(params.fast_period, "Chaikin fast_period")?;
    utils::validate_period(params.slow_period, "Chaikin slow_period")?;
    helper::validate_indicator_capacity(
        params.cmf_period.max(params.slow_period),
        2,
        0,
        "Chaikin periods",
    )?;
    utils::validate_period_order(
        params.fast_period,
        "fast_period",
        params.slow_period,
        "slow_period",
        "Chaikin",
    )?;
    helper::validate_common(params.consecutive_n, "Chaikin consecutive_n")?;
    utils::validate_non_negative_number(params.cmf_threshold, "Chaikin cmf_threshold")
}
