use super::{KAMAFilterType, KAMAParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::kama_analyzer::{KAMAAnalyzer, KAMAAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_kama<C: Candle + 'static>(
    symbol: &str,
    params: &KAMAParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let base = helper::checked_required_add(params.period, 1, "KAMA period")?;
    let needs_previous = matches!(
        params.filter_type,
        KAMAFilterType::PriceCrossAbove
            | KAMAFilterType::PriceCrossBelow
            | KAMAFilterType::Rising
            | KAMAFilterType::Falling
    );
    let required = helper::required_with_offsets(
        base,
        params.consecutive_n,
        params.p,
        needs_previous,
        "KAMA required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = KAMAAnalyzer::new(
        candle_store,
        KAMAAnalyzerParams {
            period: params.period,
            fast_period: params.fast_period,
            slow_period: params.slow_period,
        },
    );
    Ok(match params.filter_type {
        KAMAFilterType::PriceAbove => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price > data.kama.value()
            })
        }
        KAMAFilterType::PriceBelow => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                current_price < data.kama.value()
            })
        }
        KAMAFilterType::PriceCrossAbove => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| {
                current_price > current.kama.value()
                    && previous.candle.close_price() <= previous.kama.value()
            },
        ),
        KAMAFilterType::PriceCrossBelow => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| {
                current_price < current.kama.value()
                    && previous.candle.close_price() >= previous.kama.value()
            },
        ),
        KAMAFilterType::Rising => {
            helper::matches_rising(&analyzer, params.consecutive_n, params.p, |data| {
                data.kama.value()
            })
        }
        KAMAFilterType::Falling => {
            helper::matches_falling(&analyzer, params.consecutive_n, params.p, |data| {
                data.kama.value()
            })
        }
        KAMAFilterType::ERAboveThreshold => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.kama.efficiency_ratio() >= params.er_threshold
            })
        }
        KAMAFilterType::ERBelowThreshold => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.kama.efficiency_ratio() <= params.er_low_threshold
            })
        }
    })
}

pub(crate) fn validate_params(params: &KAMAParams) -> Result<()> {
    utils::validate_period(params.period, "KAMA period")?;
    helper::validate_indicator_capacity(params.period, 2, 1, "KAMA period")?;
    utils::validate_period(params.fast_period, "KAMA fast_period")?;
    utils::validate_period(params.slow_period, "KAMA slow_period")?;
    utils::validate_period_order(
        params.fast_period,
        "fast_period",
        params.slow_period,
        "slow_period",
        "KAMA",
    )?;
    helper::validate_common(params.consecutive_n, "KAMA consecutive_n")?;
    utils::validate_ratio_threshold(params.er_threshold, "KAMA er_threshold")?;
    utils::validate_ratio_threshold(params.er_low_threshold, "KAMA er_low_threshold")
}
