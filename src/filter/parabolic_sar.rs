use super::{
    FilterError, ParabolicSARFilterType, ParabolicSARParams, Result,
    standalone_indicator as helper, utils,
};
use crate::analyzer::parabolic_sar_analyzer::{ParabolicSARAnalyzer, ParabolicSARAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_parabolic_sar<C: Candle + 'static>(
    symbol: &str,
    params: &ParabolicSARParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let needs_previous = matches!(
        params.filter_type,
        ParabolicSARFilterType::Reversal
            | ParabolicSARFilterType::PriceCrossAbove
            | ParabolicSARFilterType::PriceCrossBelow
    );
    let required = helper::required_with_offsets(
        2,
        params.consecutive_n,
        params.p,
        needs_previous,
        "ParabolicSAR required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = ParabolicSARAnalyzer::new(
        candle_store,
        ParabolicSARAnalyzerParams {
            step: params.step,
            max_step: params.max_step,
        },
    );
    Ok(match params.filter_type {
        ParabolicSARFilterType::Bullish => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.parabolic_sar.is_long()
            })
        }
        ParabolicSARFilterType::Bearish => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                !data.parabolic_sar.is_long()
            })
        }
        ParabolicSARFilterType::Reversal => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| current.parabolic_sar.is_long() != previous.parabolic_sar.is_long(),
        ),
        ParabolicSARFilterType::PriceCrossAbove => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| {
                current_price > current.parabolic_sar.value()
                    && previous.candle.close_price() <= previous.parabolic_sar.value()
            },
        ),
        ParabolicSARFilterType::PriceCrossBelow => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| {
                current_price < current.parabolic_sar.value()
                    && previous.candle.close_price() >= previous.parabolic_sar.value()
            },
        ),
    })
}

pub(crate) fn validate_params(params: &ParabolicSARParams) -> Result<()> {
    utils::validate_positive_number(params.step, "ParabolicSAR step")?;
    utils::validate_positive_number(params.max_step, "ParabolicSAR max_step")?;
    if params.step > params.max_step {
        return Err(FilterError::InvalidPeriodOrder {
            param_name: "ParabolicSAR".to_string(),
            left_name: "step".to_string(),
            left: (params.step * 10_000.0) as usize,
            right_name: "max_step".to_string(),
            right: (params.max_step * 10_000.0) as usize,
        });
    }
    helper::validate_common(params.consecutive_n, "ParabolicSAR consecutive_n")
}
