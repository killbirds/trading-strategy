use super::Strategy;
use super::StrategyType;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::model::TradePosition;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 더미 전략 설정
#[derive(Debug, Deserialize)]
pub struct DummyStrategyConfig {
    /// 예제 설정 값
    example_value: String,
}

impl DummyStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.example_value.is_empty() {
            return Err("example_value는 비어있을 수 없습니다".to_string());
        }

        Ok(())
    }
}

impl Default for DummyStrategyConfig {
    fn default() -> Self {
        DummyStrategyConfig {
            example_value: "기본값".to_string(),
        }
    }
}

pub struct DummyStrategy<C: Candle> {
    /// 전략 설정
    config: DummyStrategyConfig,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Candle> Default for DummyStrategy<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Candle> DummyStrategy<C> {
    pub fn new() -> DummyStrategy<C> {
        DummyStrategy {
            config: DummyStrategyConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_with_storage(_storage: &CandleStore<C>) -> DummyStrategy<C> {
        Self::new()
    }

    pub fn new_with_config(
        _storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<DummyStrategy<C>, String> {
        match config {
            Some(cfg) => {
                let example_value = cfg
                    .get("example_value")
                    .ok_or_else(|| "example_value 설정이 필요합니다".to_string())?
                    .clone();

                let config = DummyStrategyConfig { example_value };
                config.validate()?;

                Ok(DummyStrategy {
                    config,
                    _phantom: std::marker::PhantomData,
                })
            }
            None => Ok(DummyStrategy {
                config: DummyStrategyConfig::default(),
                _phantom: std::marker::PhantomData,
            }),
        }
    }
}

impl<C: Candle> Display for DummyStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[더미전략] 설정: {{예제값: {}}}",
            self.config.example_value
        )
    }
}

impl<C: Candle> Strategy<C> for DummyStrategy<C> {
    fn next(&mut self, _candle: C) {}

    fn should_enter(&self, _candle: &C) -> bool {
        false
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        false
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Dummy
    }
}
