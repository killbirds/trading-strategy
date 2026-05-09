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
pub struct DonchianShortStrategyConfig {
    pub count: usize,
    pub period: usize,
}

impl Default for DonchianShortStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
        }
    }
}

impl ConfigValidation for DonchianShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        DonchianStrategyConfigBase {
            count: self.count,
            period: self.period,
        }
        .validate()
    }
}

impl DonchianShortStrategyConfig {
    fn from_json(json: &str) -> Result<DonchianShortStrategyConfig, String> {
        let config = DonchianStrategyConfigBase::from_json::<DonchianShortStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<DonchianShortStrategyConfig, String> {
        let base = DonchianStrategyConfigBase::from_hash_map(config)?;
        let result = DonchianShortStrategyConfig {
            count: base.count,
            period: base.period,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct DonchianShortStrategy<C: Candle> {
    config: DonchianShortStrategyConfig,
    ctx: DonchianAnalyzer<C>,
}

impl<C: Candle> Display for DonchianShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Donchian숏전략] 설정: {{기간: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> DonchianShortStrategy<C> {
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<DonchianShortStrategy<C>, String> {
        let config = DonchianShortStrategyConfig::from_json(json_config)?;
        Self::new_with_strategy_config(storage, config)
    }

    pub fn new_with_strategy_config(
        storage: &CandleStore<C>,
        config: DonchianShortStrategyConfig,
    ) -> Result<DonchianShortStrategy<C>, String> {
        info!("Donchian 숏 전략 설정: {config:?}");

        let ctx = DonchianAnalyzer::new(
            storage,
            DonchianAnalyzerParams {
                period: config.period,
            },
        );
        Ok(DonchianShortStrategy { config, ctx })
    }

    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<DonchianShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => DonchianShortStrategyConfig::from_hash_map(&cfg)?,
            None => DonchianShortStrategyConfig::default(),
        };

        Self::new_with_strategy_config(storage, strategy_config)
    }
}

impl<C: Candle + 'static> DonchianStrategyCommon<C> for DonchianShortStrategy<C> {
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

impl<C: Candle + 'static> Strategy<C> for DonchianShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, current_price: f64) -> bool {
        self.is_current_price_breakout_below(current_price)
    }

    fn should_exit(&self, current_price: f64) -> bool {
        self.is_current_price_above_previous_middle(current_price)
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::DonchianShort
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_short_strategy_config_from_json_uses_defaults() {
        let config = DonchianShortStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
    }
}
