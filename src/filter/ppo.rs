use super::{PPOFilterType, PPOParams, Result, standalone_indicator as helper, utils};
use crate::analyzer::ppo_analyzer::{PPOAnalyzer, PPOAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

pub(crate) fn filter_ppo<C: Candle + 'static>(
    symbol: &str,
    params: &PPOParams,
    candle_store: &CandleStore<C>,
    _current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let base =
        helper::checked_required_add(params.slow_period, params.signal_period, "PPO periods")?;
    let required = helper::required_with_offsets(
        base,
        params.consecutive_n,
        params.p,
        false,
        "PPO required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = PPOAnalyzer::new(
        candle_store,
        PPOAnalyzerParams {
            fast_period: params.fast_period,
            slow_period: params.slow_period,
            signal_period: params.signal_period,
        },
    );
    Ok(match params.filter_type {
        PPOFilterType::AboveSignal => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.ppo() > data.ppo.signal()
            })
        }
        PPOFilterType::BelowSignal => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.ppo() < data.ppo.signal()
            })
        }
        PPOFilterType::AboveZero => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.ppo() > 0.0
            })
        }
        PPOFilterType::BelowZero => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.ppo() < 0.0
            })
        }
        PPOFilterType::HistogramPositive => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.histogram() > 0.0
            })
        }
        PPOFilterType::HistogramNegative => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.ppo.histogram() < 0.0
            })
        }
    })
}

pub(crate) fn validate_params(params: &PPOParams) -> Result<()> {
    utils::validate_period(params.fast_period, "PPO fast_period")?;
    utils::validate_period(params.slow_period, "PPO slow_period")?;
    utils::validate_period(params.signal_period, "PPO signal_period")?;
    utils::validate_period_order(
        params.fast_period,
        "fast_period",
        params.slow_period,
        "slow_period",
        "PPO",
    )?;
    helper::validate_common(params.consecutive_n, "PPO consecutive_n")
}
