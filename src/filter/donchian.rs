use super::{DonchianFilterType, DonchianParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::donchian_analyzer::{DonchianAnalyzer, DonchianAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_donchian<C: Candle + 'static>(
    symbol: &str,
    params: &DonchianParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let required = helper::required_with_offsets(
        params.period,
        params.consecutive_n,
        params.p,
        matches!(
            params.filter_type,
            DonchianFilterType::BreakoutAbove | DonchianFilterType::BreakoutBelow
        ),
        "Donchian required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = DonchianAnalyzer::new(
        candle_store,
        DonchianAnalyzerParams {
            period: params.period,
        },
    );
    let result = match params.filter_type {
        DonchianFilterType::BreakoutAbove => {
            helper::matches_previous(&analyzer, params.consecutive_n, params.p, |_, previous| {
                current_price > previous.donchian.upper()
            })
        }
        DonchianFilterType::BreakoutBelow => {
            helper::matches_previous(&analyzer, params.consecutive_n, params.p, |_, previous| {
                current_price < previous.donchian.lower()
            })
        }
        DonchianFilterType::InsideChannel => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.donchian.contains_price(current_price)
            })
        }
        DonchianFilterType::AboveMiddle => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price > data.donchian.middle()
            })
        }
        DonchianFilterType::BelowMiddle => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price < data.donchian.middle()
            })
        }
        DonchianFilterType::NearUpperEdge => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.donchian.percent_b(current_price) >= 1.0 - params.edge_threshold
            })
        }
        DonchianFilterType::NearLowerEdge => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.donchian.percent_b(current_price) <= params.edge_threshold
            })
        }
    };
    Ok(result)
}

pub(crate) fn validate_params(params: &DonchianParams) -> Result<()> {
    utils::validate_period(params.period, "Donchian period")?;
    helper::validate_indicator_capacity(params.period, 2, 0, "Donchian period")?;
    helper::validate_common(params.consecutive_n, "Donchian consecutive_n")?;
    utils::validate_ratio_threshold(params.edge_threshold, "Donchian edge_threshold")
}
