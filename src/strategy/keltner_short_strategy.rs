use super::keltner_common::{
    KeltnerAnalyzer, KeltnerAnalyzerParams, KeltnerStrategyCommon, KeltnerStrategyConfigBase,
};
use super::{Strategy, StrategyType};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::{ConfigResult, ConfigValidation};
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct KeltnerShortStrategyConfig {
    pub count: usize,
    pub period: usize,
    pub multiplier: f64,
}

impl Default for KeltnerShortStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl ConfigValidation for KeltnerShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        KeltnerStrategyConfigBase {
            count: self.count,
            period: self.period,
            multiplier: self.multiplier,
        }
        .validate()
    }
}

impl KeltnerShortStrategyConfig {
    fn from_json(json: &str) -> Result<KeltnerShortStrategyConfig, String> {
        let config = KeltnerStrategyConfigBase::from_json::<KeltnerShortStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<KeltnerShortStrategyConfig, String> {
        let base = KeltnerStrategyConfigBase::from_hash_map(config)?;
        let result = KeltnerShortStrategyConfig {
            count: base.count,
            period: base.period,
            multiplier: base.multiplier,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct KeltnerShortStrategy<C: Candle> {
    config: KeltnerShortStrategyConfig,
    ctx: KeltnerAnalyzer<C>,
}

impl<C: Candle> Display for KeltnerShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Keltner숏전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> KeltnerShortStrategy<C> {
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<KeltnerShortStrategy<C>, String> {
        let config = KeltnerShortStrategyConfig::from_json(json_config)?;
        Self::new_with_strategy_config(storage, config)
    }

    pub fn new_with_strategy_config(
        storage: &CandleStore<C>,
        config: KeltnerShortStrategyConfig,
    ) -> Result<KeltnerShortStrategy<C>, String> {
        info!("Keltner 숏 전략 설정: {config:?}");

        let ctx = KeltnerAnalyzer::new(
            storage,
            KeltnerAnalyzerParams {
                period: config.period,
                multiplier: config.multiplier,
            },
        );
        Ok(KeltnerShortStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<KeltnerShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => KeltnerShortStrategyConfig::from_hash_map(&cfg)?,
            None => KeltnerShortStrategyConfig::default(),
        };

        Self::new_with_strategy_config(storage, strategy_config)
    }
}

impl<C: Candle + 'static> KeltnerStrategyCommon<C> for KeltnerShortStrategy<C> {
    fn context(&self) -> &KeltnerAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }

    fn config_period(&self) -> usize {
        self.config.period
    }
}

impl<C: Candle + 'static> Strategy<C> for KeltnerShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn evaluate(
        &self,
        current_price: f64,
        position_state: crate::model::PositionState,
    ) -> crate::model::Signal {
        crate::strategy::evaluate_signal(
            PositionType::Short,
            position_state,
            || self.is_current_price_breakout_below(current_price),
            || self.is_current_price_above_previous_middle(current_price),
        )
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::KeltnerShort
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_short_strategy_config_from_json_uses_defaults() {
        let config = KeltnerShortStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
        assert_eq!(config.multiplier, 2.0);
    }
}
