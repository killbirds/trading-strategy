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
pub struct KeltnerStrategyConfig {
    pub count: usize,
    pub period: usize,
    pub multiplier: f64,
}

impl Default for KeltnerStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl ConfigValidation for KeltnerStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        KeltnerStrategyConfigBase {
            count: self.count,
            period: self.period,
            multiplier: self.multiplier,
        }
        .validate()
    }
}

impl KeltnerStrategyConfig {
    fn from_json(json: &str) -> Result<KeltnerStrategyConfig, String> {
        let config = KeltnerStrategyConfigBase::from_json::<KeltnerStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(config: &HashMap<String, String>) -> Result<KeltnerStrategyConfig, String> {
        let base = KeltnerStrategyConfigBase::from_hash_map(config)?;
        let result = KeltnerStrategyConfig {
            count: base.count,
            period: base.period,
            multiplier: base.multiplier,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct KeltnerStrategy<C: Candle> {
    config: KeltnerStrategyConfig,
    ctx: KeltnerAnalyzer<C>,
}

impl<C: Candle> Display for KeltnerStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Keltner전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> KeltnerStrategy<C> {
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<KeltnerStrategy<C>, String> {
        let config = KeltnerStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    pub fn new(
        storage: &CandleStore<C>,
        config: KeltnerStrategyConfig,
    ) -> Result<KeltnerStrategy<C>, String> {
        info!("Keltner 전략 설정: {config:?}");

        let ctx = KeltnerAnalyzer::new(
            storage,
            KeltnerAnalyzerParams {
                period: config.period,
                multiplier: config.multiplier,
            },
        );
        Ok(KeltnerStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<KeltnerStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => KeltnerStrategyConfig::from_hash_map(&cfg)?,
            None => KeltnerStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> KeltnerStrategyCommon<C> for KeltnerStrategy<C> {
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

impl<C: Candle + 'static> Strategy<C> for KeltnerStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, current_price: f64) -> bool {
        self.is_current_price_breakout_above(current_price)
    }

    fn should_exit(&self, current_price: f64) -> bool {
        self.is_current_price_below_previous_middle(current_price)
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Keltner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_strategy_config_from_json_uses_defaults() {
        let config = KeltnerStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
        assert_eq!(config.multiplier, 2.0);
    }

    #[test]
    fn test_keltner_strategy_config_rejects_invalid_values() {
        assert!(KeltnerStrategyConfig::from_json(r#"{"count":0}"#).is_err());
        assert!(KeltnerStrategyConfig::from_json(r#"{"period":0}"#).is_err());
        assert!(KeltnerStrategyConfig::from_json(r#"{"multiplier":0.0}"#).is_err());
    }
}
