use super::donchian_common::{
    DonchianAnalyzer, DonchianAnalyzerParams, DonchianStrategyCommon, DonchianStrategyConfigBase,
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
pub struct DonchianStrategyConfig {
    pub count: usize,
    pub period: usize,
}

impl Default for DonchianStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
        }
    }
}

impl ConfigValidation for DonchianStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        DonchianStrategyConfigBase {
            count: self.count,
            period: self.period,
        }
        .validate()
    }
}

impl DonchianStrategyConfig {
    fn from_json(json: &str) -> Result<DonchianStrategyConfig, String> {
        let config = DonchianStrategyConfigBase::from_json::<DonchianStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(config: &HashMap<String, String>) -> Result<DonchianStrategyConfig, String> {
        let base = DonchianStrategyConfigBase::from_hash_map(config)?;
        let result = DonchianStrategyConfig {
            count: base.count,
            period: base.period,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct DonchianStrategy<C: Candle> {
    config: DonchianStrategyConfig,
    ctx: DonchianAnalyzer<C>,
}

impl<C: Candle> Display for DonchianStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Donchian전략] 설정: {{기간: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> DonchianStrategy<C> {
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<DonchianStrategy<C>, String> {
        let config = DonchianStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    pub fn new(
        storage: &CandleStore<C>,
        config: DonchianStrategyConfig,
    ) -> Result<DonchianStrategy<C>, String> {
        info!("Donchian 전략 설정: {config:?}");

        let ctx = DonchianAnalyzer::new(
            storage,
            DonchianAnalyzerParams {
                period: config.period,
            },
        );
        Ok(DonchianStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<DonchianStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => DonchianStrategyConfig::from_hash_map(&cfg)?,
            None => DonchianStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> DonchianStrategyCommon<C> for DonchianStrategy<C> {
    fn context(&self) -> &DonchianAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }

    fn config_period(&self) -> usize {
        self.config.period
    }
}

impl<C: Candle + 'static> Strategy<C> for DonchianStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn evaluate(
        &self,
        current_price: f64,
        position_state: crate::model::PositionState,
    ) -> crate::model::Signal {
        crate::strategy::evaluate_signal(
            PositionType::Long,
            position_state,
            || self.is_current_price_breakout_above(current_price),
            || self.is_current_price_below_previous_middle(current_price),
        )
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Donchian
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_strategy_config_from_json_uses_defaults() {
        let config = DonchianStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
    }

    #[test]
    fn test_donchian_strategy_config_rejects_invalid_values() {
        assert!(DonchianStrategyConfig::from_json(r#"{"count":0}"#).is_err());
        assert!(DonchianStrategyConfig::from_json(r#"{"period":0}"#).is_err());
    }
}
