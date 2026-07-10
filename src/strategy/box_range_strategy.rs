use super::Strategy;
use super::StrategyType;
use super::box_range_common::{
    BoxRangeAnalyzer, BoxRangeStrategyCommon, BoxRangeStrategyConfigBase,
};
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

/// 박스권 돌파 롱 전략 설정
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BoxRangeStrategyConfig {
    /// 돌파 전 확인할 박스권 데이터 수
    pub count: usize,
    /// 박스권 계산 기간
    pub period: usize,
    /// 박스권 판정 최대 폭 비율
    pub max_width_ratio: f64,
}

impl Default for BoxRangeStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
            max_width_ratio: 0.05,
        }
    }
}

impl ConfigValidation for BoxRangeStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        BoxRangeStrategyConfigBase {
            count: self.count,
            period: self.period,
            max_width_ratio: self.max_width_ratio,
        }
        .validate()
    }
}

impl BoxRangeStrategyConfig {
    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<BoxRangeStrategyConfig, String> {
        let config = BoxRangeStrategyConfigBase::from_json::<BoxRangeStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BoxRangeStrategyConfig, String> {
        let base = BoxRangeStrategyConfigBase::from_hash_map(config)?;
        let result = BoxRangeStrategyConfig {
            count: base.count,
            period: base.period,
            max_width_ratio: base.max_width_ratio,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// 박스권 상단 돌파 기반 롱 전략
#[derive(Debug)]
pub struct BoxRangeStrategy<C: Candle> {
    config: BoxRangeStrategyConfig,
    ctx: BoxRangeAnalyzer<C>,
}

impl<C: Candle> Display for BoxRangeStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[박스권전략] 설정: {{기간: {}, 최대폭비율: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.max_width_ratio, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BoxRangeStrategy<C> {
    /// 새 박스권 롱 전략 인스턴스 생성 (JSON 설정 사용)
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BoxRangeStrategy<C>, String> {
        let config = BoxRangeStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 박스권 롱 전략 인스턴스 생성
    pub fn new(
        storage: &CandleStore<C>,
        config: BoxRangeStrategyConfig,
    ) -> Result<BoxRangeStrategy<C>, String> {
        info!("박스권 전략 설정: {config:?}");

        let ctx = BoxRangeAnalyzer::new(config.period, config.max_width_ratio, storage);
        Ok(BoxRangeStrategy { config, ctx })
    }

    /// 새 박스권 롱 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BoxRangeStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => BoxRangeStrategyConfig::from_hash_map(&cfg)?,
            None => BoxRangeStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> BoxRangeStrategyCommon<C> for BoxRangeStrategy<C> {
    fn context(&self) -> &BoxRangeAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }
}

impl<C: Candle + 'static> Strategy<C> for BoxRangeStrategy<C> {
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
        StrategyType::BoxRange
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_range_strategy_config_from_json_uses_defaults() {
        let config = BoxRangeStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
        assert_eq!(config.max_width_ratio, 0.05);
    }

    #[test]
    fn test_box_range_strategy_config_rejects_invalid_thresholds() {
        assert!(BoxRangeStrategyConfig::from_json(r#"{"period":0}"#).is_err());
        assert!(BoxRangeStrategyConfig::from_json(r#"{"max_width_ratio":0.0}"#).is_err());
        assert!(BoxRangeStrategyConfig::from_json(r#"{"count":0}"#).is_err());
    }
}
