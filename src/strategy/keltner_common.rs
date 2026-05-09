use super::Strategy;
use super::config_utils;
use crate::indicator::{MAX_INDICATOR_CAPACITY, checked_indicator_capacity};
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

pub use crate::analyzer::keltner_analyzer::{
    KeltnerAnalyzer, KeltnerAnalyzerData, KeltnerAnalyzerParams,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct KeltnerStrategyConfigBase {
    pub count: usize,
    pub period: usize,
    pub multiplier: f64,
}

impl ConfigValidation for KeltnerStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "Keltner 확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.period == 0 {
            return Err(ConfigError::ValidationError(
                "Keltner 계산 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        if !self.multiplier.is_finite() || self.multiplier <= 0.0 {
            return Err(ConfigError::ValidationError(
                "Keltner 승수는 유한한 양수여야 합니다".to_string(),
            ));
        }

        if self.count > MAX_INDICATOR_CAPACITY {
            return Err(ConfigError::ValidationError(
                "Keltner 확인 캔들 수가 너무 큽니다".to_string(),
            ));
        }

        checked_indicator_capacity("Keltner Channel", self.period, 2, 0)
            .map_err(ConfigError::ValidationError)?;

        Ok(())
    }
}

impl KeltnerStrategyConfigBase {
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str::<T>(json).map_err(|e| format!("JSON 설정 역직렬화 실패: {e}"))
    }

    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<KeltnerStrategyConfigBase, String> {
        let count = config_utils::parse_usize(config, "count", Some(1), true)?
            .ok_or("count 설정이 필요합니다")?;
        let period = config_utils::parse_usize(config, "period", Some(1), true)?
            .ok_or("period 설정이 필요합니다")?;
        let multiplier =
            config_utils::parse_f64(config, "multiplier", Some((0.0, f64::MAX)), true)?
                .ok_or("multiplier 설정이 필요합니다")?;

        let result = KeltnerStrategyConfigBase {
            count,
            period,
            multiplier,
        };
        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub trait KeltnerStrategyCommon<C: Candle + 'static>: Strategy<C> {
    fn context(&self) -> &KeltnerAnalyzer<C>;

    fn config_count(&self) -> usize;

    fn config_period(&self) -> usize;

    fn previous_keltner_data(&self) -> Option<&KeltnerAnalyzerData<C>> {
        self.context().items.get(1)
    }

    fn has_prior_channels(&self) -> bool {
        self.config_period()
            .checked_add(self.config_count())
            .map(|required_len| self.context().items.len() >= required_len)
            .unwrap_or(false)
    }

    fn is_current_price_breakout_above(&self, current_price: f64) -> bool {
        self.has_prior_channels()
            && self
                .previous_keltner_data()
                .map(|data| current_price > data.keltner.upper())
                .unwrap_or(false)
    }

    fn is_current_price_breakout_below(&self, current_price: f64) -> bool {
        self.has_prior_channels()
            && self
                .previous_keltner_data()
                .map(|data| current_price < data.keltner.lower())
                .unwrap_or(false)
    }

    fn is_current_price_below_previous_middle(&self, current_price: f64) -> bool {
        self.previous_keltner_data()
            .map(|data| current_price < data.keltner.middle())
            .unwrap_or(false)
    }

    fn is_current_price_above_previous_middle(&self, current_price: f64) -> bool {
        self.previous_keltner_data()
            .map(|data| current_price > data.keltner.middle())
            .unwrap_or(false)
    }
}
