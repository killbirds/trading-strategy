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
pub struct ParabolicSARStrategyConfig {
    pub count: usize,
    pub step: f64,
    pub max_step: f64,
}

impl Default for ParabolicSARStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            step: 0.02,
            max_step: 0.2,
        }
    }
}

impl ConfigValidation for ParabolicSARStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        ParabolicSARStrategyConfigBase {
            count: self.count,
            step: self.step,
            max_step: self.max_step,
        }
        .validate()
    }
}

impl ParabolicSARStrategyConfig {
    fn from_json(json: &str) -> Result<ParabolicSARStrategyConfig, String> {
        let config = ParabolicSARStrategyConfigBase::from_json::<ParabolicSARStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ParabolicSARStrategyConfig, String> {
        let base = ParabolicSARStrategyConfigBase::from_hash_map(config)?;
        let result = ParabolicSARStrategyConfig {
            count: base.count,
            step: base.step,
            max_step: base.max_step,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct ParabolicSARStrategy<C: Candle> {
    config: ParabolicSARStrategyConfig,
    ctx: ParabolicSARAnalyzer<C>,
}

impl<C: Candle> Display for ParabolicSARStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Parabolic SAR전략] 설정: {{step: {}, max_step: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.step, self.config.max_step, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> ParabolicSARStrategy<C> {
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<ParabolicSARStrategy<C>, String> {
        let config = ParabolicSARStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    pub fn new(
        storage: &CandleStore<C>,
        config: ParabolicSARStrategyConfig,
    ) -> Result<ParabolicSARStrategy<C>, String> {
        info!("Parabolic SAR 전략 설정: {config:?}");

        let ctx = ParabolicSARAnalyzer::new(
            storage,
            ParabolicSARAnalyzerParams {
                step: config.step,
                max_step: config.max_step,
            },
        );
        Ok(ParabolicSARStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<ParabolicSARStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => ParabolicSARStrategyConfig::from_hash_map(&cfg)?,
            None => ParabolicSARStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> ParabolicSARStrategyCommon<C> for ParabolicSARStrategy<C> {
    fn context(&self) -> &ParabolicSARAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }
}

impl<C: Candle + 'static> Strategy<C> for ParabolicSARStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, current_price: f64) -> bool {
        self.has_confirmed_bullish_trend(current_price)
    }

    fn should_exit(&self, current_price: f64) -> bool {
        self.is_current_bearish() || self.is_price_below_current_sar(current_price)
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::ParabolicSAR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parabolic_sar_strategy_config_from_json_uses_defaults() {
        let config = ParabolicSARStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.step, 0.02);
        assert_eq!(config.max_step, 0.2);
    }

    #[test]
    fn test_parabolic_sar_strategy_config_rejects_invalid_values() {
        assert!(ParabolicSARStrategyConfig::from_json(r#"{"count":0}"#).is_err());
        assert!(ParabolicSARStrategyConfig::from_json(r#"{"step":0.0}"#).is_err());
        assert!(ParabolicSARStrategyConfig::from_json(r#"{"max_step":0.0}"#).is_err());
        assert!(ParabolicSARStrategyConfig::from_json(r#"{"step":0.3,"max_step":0.2}"#).is_err());
    }
}
