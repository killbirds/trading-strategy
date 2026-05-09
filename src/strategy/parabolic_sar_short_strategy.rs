use super::parabolic_sar_common::{
    ParabolicSARAnalyzer, ParabolicSARAnalyzerParams, ParabolicSARStrategyCommon,
    ParabolicSARStrategyConfigBase,
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
pub struct ParabolicSARShortStrategyConfig {
    pub count: usize,
    pub step: f64,
    pub max_step: f64,
}

impl Default for ParabolicSARShortStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            step: 0.02,
            max_step: 0.2,
        }
    }
}

impl ConfigValidation for ParabolicSARShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        ParabolicSARStrategyConfigBase {
            count: self.count,
            step: self.step,
            max_step: self.max_step,
        }
        .validate()
    }
}

impl ParabolicSARShortStrategyConfig {
    fn from_json(json: &str) -> Result<ParabolicSARShortStrategyConfig, String> {
        let config =
            ParabolicSARStrategyConfigBase::from_json::<ParabolicSARShortStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ParabolicSARShortStrategyConfig, String> {
        let base = ParabolicSARStrategyConfigBase::from_hash_map(config)?;
        let result = ParabolicSARShortStrategyConfig {
            count: base.count,
            step: base.step,
            max_step: base.max_step,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct ParabolicSARShortStrategy<C: Candle> {
    config: ParabolicSARShortStrategyConfig,
    ctx: ParabolicSARAnalyzer<C>,
}

impl<C: Candle> Display for ParabolicSARShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Parabolic SAR숏전략] 설정: {{step: {}, max_step: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.step, self.config.max_step, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> ParabolicSARShortStrategy<C> {
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<ParabolicSARShortStrategy<C>, String> {
        let config = ParabolicSARShortStrategyConfig::from_json(json_config)?;
        Self::new_with_strategy_config(storage, config)
    }

    pub fn new_with_strategy_config(
        storage: &CandleStore<C>,
        config: ParabolicSARShortStrategyConfig,
    ) -> Result<ParabolicSARShortStrategy<C>, String> {
        info!("Parabolic SAR 숏 전략 설정: {config:?}");

        let ctx = ParabolicSARAnalyzer::new(
            storage,
            ParabolicSARAnalyzerParams {
                step: config.step,
                max_step: config.max_step,
            },
        );
        Ok(ParabolicSARShortStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<ParabolicSARShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => ParabolicSARShortStrategyConfig::from_hash_map(&cfg)?,
            None => ParabolicSARShortStrategyConfig::default(),
        };

        Self::new_with_strategy_config(storage, strategy_config)
    }
}

impl<C: Candle + 'static> ParabolicSARStrategyCommon<C> for ParabolicSARShortStrategy<C> {
    fn context(&self) -> &ParabolicSARAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }
}

impl<C: Candle + 'static> Strategy<C> for ParabolicSARShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, current_price: f64) -> bool {
        self.has_confirmed_bearish_trend(current_price)
    }

    fn should_exit(&self, current_price: f64) -> bool {
        self.is_current_bullish() || self.is_price_above_current_sar(current_price)
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::ParabolicSARShort
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parabolic_sar_short_strategy_config_from_json_uses_defaults() {
        let config = ParabolicSARShortStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.step, 0.02);
        assert_eq!(config.max_step, 0.2);
    }
}
