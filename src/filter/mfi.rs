use super::{MFIFilterType, MFIParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::mfi_analyzer::{MFIAnalyzer, MFIAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_mfi<C: Candle + 'static>(
    symbol: &str,
    params: &MFIParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let base = helper::checked_required_add(params.period, 1, "MFI period")?;
    let required = helper::required_with_offsets(
        base,
        params.consecutive_n,
        params.p,
        false,
        "MFI required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = MFIAnalyzer::new(
        candle_store,
        MFIAnalyzerParams {
            period: params.period,
        },
    );
    Ok(match params.filter_type {
        MFIFilterType::Overbought => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.mfi.value() >= params.overbought
            })
        }
        MFIFilterType::Oversold => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.mfi.value() <= params.oversold
            })
        }
        MFIFilterType::AboveThreshold => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.mfi.value() > params.threshold
            })
        }
        MFIFilterType::BelowThreshold => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.mfi.value() < params.threshold
            })
        }
    })
}

pub(crate) fn validate_params(params: &MFIParams) -> Result<()> {
    utils::validate_period(params.period, "MFI period")?;
    helper::validate_indicator_capacity(params.period, 2, 1, "MFI period")?;
    helper::validate_common(params.consecutive_n, "MFI consecutive_n")?;
    utils::validate_percentage_threshold(params.overbought, "MFI overbought")?;
    utils::validate_percentage_threshold(params.oversold, "MFI oversold")?;
    utils::validate_percentage_threshold(params.threshold, "MFI threshold")
}
