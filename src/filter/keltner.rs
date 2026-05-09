use super::{KeltnerFilterType, KeltnerParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::keltner_analyzer::{KeltnerAnalyzer, KeltnerAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_keltner<C: Candle + 'static>(
    symbol: &str,
    params: &KeltnerParams,
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
            KeltnerFilterType::BreakoutAbove | KeltnerFilterType::BreakoutBelow
        ),
        "Keltner required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = KeltnerAnalyzer::new(
        candle_store,
        KeltnerAnalyzerParams {
            period: params.period,
            multiplier: params.multiplier,
        },
    );
    let result = match params.filter_type {
        KeltnerFilterType::BreakoutAbove => {
            helper::matches_previous(&analyzer, params.consecutive_n, params.p, |_, previous| {
                current_price > previous.keltner.upper()
            })
        }
        KeltnerFilterType::BreakoutBelow => {
            helper::matches_previous(&analyzer, params.consecutive_n, params.p, |_, previous| {
                current_price < previous.keltner.lower()
            })
        }
        KeltnerFilterType::InsideChannel => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price >= data.keltner.lower() && current_price <= data.keltner.upper()
            })
        }
        KeltnerFilterType::AboveMiddle => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price > data.keltner.middle()
            })
        }
        KeltnerFilterType::BelowMiddle => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price < data.keltner.middle()
            })
        }
        KeltnerFilterType::NearUpperEdge => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                helper::channel_position(current_price, data.keltner.lower(), data.keltner.upper())
                    >= 1.0 - params.edge_threshold
            })
        }
        KeltnerFilterType::NearLowerEdge => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                helper::channel_position(current_price, data.keltner.lower(), data.keltner.upper())
                    <= params.edge_threshold
            })
        }
    };
    Ok(result)
}

pub(crate) fn validate_params(params: &KeltnerParams) -> Result<()> {
    utils::validate_period(params.period, "Keltner period")?;
    helper::validate_indicator_capacity(params.period, 2, 0, "Keltner period")?;
    utils::validate_positive_number(params.multiplier, "Keltner multiplier")?;
    helper::validate_common(params.consecutive_n, "Keltner consecutive_n")?;
    utils::validate_ratio_threshold(params.edge_threshold, "Keltner edge_threshold")
}
