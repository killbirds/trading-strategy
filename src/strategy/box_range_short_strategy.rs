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

/// 박스권 이탈 숏 전략 설정
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BoxRangeShortStrategyConfig {
    /// 이탈 전 확인할 박스권 데이터 수
    pub count: usize,
    /// 박스권 계산 기간
    pub period: usize,
    /// 박스권 판정 최대 폭 비율
    pub max_width_ratio: f64,
}

impl Default for BoxRangeShortStrategyConfig {
    fn default() -> Self {
        Self {
            count: 1,
            period: 20,
            max_width_ratio: 0.05,
        }
    }
}

impl ConfigValidation for BoxRangeShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        BoxRangeStrategyConfigBase {
            count: self.count,
            period: self.period,
            max_width_ratio: self.max_width_ratio,
        }
        .validate()
    }
}

impl BoxRangeShortStrategyConfig {
    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<BoxRangeShortStrategyConfig, String> {
        let config = BoxRangeStrategyConfigBase::from_json::<BoxRangeShortStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<BoxRangeShortStrategyConfig, String> {
        let base = BoxRangeStrategyConfigBase::from_hash_map(config)?;
        let result = BoxRangeShortStrategyConfig {
            count: base.count,
            period: base.period,
            max_width_ratio: base.max_width_ratio,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// 박스권 하단 이탈 기반 숏 전략
#[derive(Debug)]
pub struct BoxRangeShortStrategy<C: Candle> {
    config: BoxRangeShortStrategyConfig,
    ctx: BoxRangeAnalyzer<C>,
}

impl<C: Candle> Display for BoxRangeShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[박스권숏전략] 설정: {{기간: {}, 최대폭비율: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.max_width_ratio, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BoxRangeShortStrategy<C> {
    /// 새 박스권 숏 전략 인스턴스 생성 (JSON 설정 사용)
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BoxRangeShortStrategy<C>, String> {
        let config = BoxRangeShortStrategyConfig::from_json(json_config)?;
        Self::from_config(storage, config)
    }

    /// 새 박스권 숏 전략 인스턴스 생성 (JSON 설정 사용)
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BoxRangeShortStrategy<C>, String> {
        Self::new(storage, json_config)
    }

    /// 새 박스권 숏 전략 인스턴스 생성 (검증 완료된 설정 사용)
    pub fn from_config(
        storage: &CandleStore<C>,
        config: BoxRangeShortStrategyConfig,
    ) -> Result<BoxRangeShortStrategy<C>, String> {
        info!("박스권 숏 전략 설정: {config:?}");
        let ctx = BoxRangeAnalyzer::new(config.period, config.max_width_ratio, storage);
        Ok(BoxRangeShortStrategy { config, ctx })
    }

    /// 새 박스권 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BoxRangeShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => BoxRangeShortStrategyConfig::from_hash_map(&cfg)?,
            None => BoxRangeShortStrategyConfig::default(),
        };

        Self::from_config(storage, strategy_config)
    }
}

impl<C: Candle + 'static> BoxRangeStrategyCommon<C> for BoxRangeShortStrategy<C> {
    fn context(&self) -> &BoxRangeAnalyzer<C> {
        &self.ctx
    }

    fn config_count(&self) -> usize {
        self.config.count
    }
}

impl<C: Candle + 'static> Strategy<C> for BoxRangeShortStrategy<C> {
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
        StrategyType::BoxRangeShort
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_range_short_strategy_config_from_json_uses_defaults() {
        let config = BoxRangeShortStrategyConfig::from_json("{}").unwrap();

        assert_eq!(config.count, 1);
        assert_eq!(config.period, 20);
        assert_eq!(config.max_width_ratio, 0.05);
    }

    #[test]
    fn test_box_range_short_strategy_new_keeps_json_constructor() {
        let storage = CandleStore::new(Vec::<crate::tests::TestCandle>::new(), 1000, false);
        let strategy = BoxRangeShortStrategy::new(&storage, "{}").unwrap();

        assert_eq!(strategy.config.count, 1);
        assert_eq!(strategy.config.period, 20);
        assert_eq!(strategy.config.max_width_ratio, 0.05);
    }

    #[test]
    fn test_box_range_short_strategy_from_config_uses_typed_config() {
        let storage = CandleStore::new(Vec::<crate::tests::TestCandle>::new(), 1000, false);
        let strategy = BoxRangeShortStrategy::from_config(
            &storage,
            BoxRangeShortStrategyConfig {
                count: 2,
                period: 10,
                max_width_ratio: 0.03,
            },
        )
        .unwrap();

        assert_eq!(strategy.config.count, 2);
        assert_eq!(strategy.config.period, 10);
        assert_eq!(strategy.config.max_width_ratio, 0.03);
    }
}
