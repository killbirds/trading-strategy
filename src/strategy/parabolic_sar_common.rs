use super::Strategy;
use super::config_utils;
use crate::indicator::MAX_INDICATOR_CAPACITY;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

pub use crate::analyzer::parabolic_sar_analyzer::{
    ParabolicSARAnalyzer, ParabolicSARAnalyzerData, ParabolicSARAnalyzerParams,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ParabolicSARStrategyConfigBase {
    pub count: usize,
    pub step: f64,
    pub max_step: f64,
}

impl ConfigValidation for ParabolicSARStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "Parabolic SAR 확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.count > MAX_INDICATOR_CAPACITY {
            return Err(ConfigError::ValidationError(
                "Parabolic SAR 확인 캔들 수가 너무 큽니다".to_string(),
            ));
        }

        if !self.step.is_finite() || self.step <= 0.0 {
            return Err(ConfigError::ValidationError(
                "Parabolic SAR step은 유한한 양수여야 합니다".to_string(),
            ));
        }

        if !self.max_step.is_finite() || self.max_step <= 0.0 {
            return Err(ConfigError::ValidationError(
                "Parabolic SAR max_step은 유한한 양수여야 합니다".to_string(),
            ));
        }

        if self.step > self.max_step {
            return Err(ConfigError::ValidationError(
                "Parabolic SAR step은 max_step보다 작거나 같아야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl ParabolicSARStrategyConfigBase {
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str::<T>(json).map_err(|e| format!("JSON 설정 역직렬화 실패: {e}"))
    }

    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ParabolicSARStrategyConfigBase, String> {
        let count = config_utils::parse_usize(config, "count", Some(1), true)?
            .ok_or("count 설정이 필요합니다")?;
        let step = config_utils::parse_f64(config, "step", Some((0.0, f64::MAX)), true)?
            .ok_or("step 설정이 필요합니다")?;
        let max_step = config_utils::parse_f64(config, "max_step", Some((0.0, f64::MAX)), true)?
            .ok_or("max_step 설정이 필요합니다")?;

        let result = ParabolicSARStrategyConfigBase {
            count,
            step,
            max_step,
        };
        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub trait ParabolicSARStrategyCommon<C: Candle + 'static>: Strategy<C> {
    fn context(&self) -> &ParabolicSARAnalyzer<C>;

    fn config_count(&self) -> usize;

    fn current_parabolic_sar_data(&self) -> Option<&ParabolicSARAnalyzerData<C>> {
        self.context().items.first()
    }

    fn has_confirmed_bullish_trend(&self, current_price: f64) -> bool {
        self.context().items.len() >= self.config_count()
            && self
                .context()
                .items
                .iter()
                .take(self.config_count())
                .all(|data| data.parabolic_sar.is_long())
            && self
                .current_parabolic_sar_data()
                .map(|data| current_price > data.parabolic_sar.value())
                .unwrap_or(false)
    }

    fn has_confirmed_bearish_trend(&self, current_price: f64) -> bool {
        self.context().items.len() >= self.config_count()
            && self
                .context()
                .items
                .iter()
                .take(self.config_count())
                .all(|data| !data.parabolic_sar.is_long())
            && self
                .current_parabolic_sar_data()
                .map(|data| current_price < data.parabolic_sar.value())
                .unwrap_or(false)
    }

    fn is_price_below_current_sar(&self, current_price: f64) -> bool {
        self.current_parabolic_sar_data()
            .map(|data| current_price < data.parabolic_sar.value())
            .unwrap_or(false)
    }

    fn is_price_above_current_sar(&self, current_price: f64) -> bool {
        self.current_parabolic_sar_data()
            .map(|data| current_price > data.parabolic_sar.value())
            .unwrap_or(false)
    }

    fn is_current_bearish(&self) -> bool {
        self.current_parabolic_sar_data()
            .map(|data| !data.parabolic_sar.is_long())
            .unwrap_or(false)
    }

    fn is_current_bullish(&self) -> bool {
        self.current_parabolic_sar_data()
            .map(|data| data.parabolic_sar.is_long())
            .unwrap_or(false)
    }
}
