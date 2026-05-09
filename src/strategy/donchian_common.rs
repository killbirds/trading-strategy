use super::Strategy;
use super::config_utils;
use crate::indicator::{MAX_INDICATOR_CAPACITY, checked_indicator_capacity};
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

pub use crate::analyzer::donchian_analyzer::{
    DonchianAnalyzer, DonchianAnalyzerData, DonchianAnalyzerParams,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct DonchianStrategyConfigBase {
    pub count: usize,
    pub period: usize,
}

impl ConfigValidation for DonchianStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "Donchian 확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.period == 0 {
            return Err(ConfigError::ValidationError(
                "Donchian 계산 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        if self.count > MAX_INDICATOR_CAPACITY {
            return Err(ConfigError::ValidationError(
                "Donchian 확인 캔들 수가 너무 큽니다".to_string(),
            ));
        }

        checked_indicator_capacity("Donchian Channel", self.period, 2, 0)
            .map_err(ConfigError::ValidationError)?;

        Ok(())
    }
}

impl DonchianStrategyConfigBase {
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str::<T>(json).map_err(|e| format!("JSON 설정 역직렬화 실패: {e}"))
    }

    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<DonchianStrategyConfigBase, String> {
        let count = config_utils::parse_usize(config, "count", Some(1), true)?
            .ok_or("count 설정이 필요합니다")?;
        let period = config_utils::parse_usize(config, "period", Some(1), true)?
            .ok_or("period 설정이 필요합니다")?;

        let result = DonchianStrategyConfigBase { count, period };
        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

pub trait DonchianStrategyCommon<C: Candle + 'static>: Strategy<C> {
    fn context(&self) -> &DonchianAnalyzer<C>;

    fn config_count(&self) -> usize;

    fn config_period(&self) -> usize;

    fn previous_donchian_data(&self) -> Option<&DonchianAnalyzerData<C>> {
        self.context().items.get(1)
    }

    fn has_prior_channels(&self) -> bool {
        let count = self.config_count();
        let required_len = match count.checked_add(1) {
            Some(value) => value,
            None => return false,
        };

        if self.context().items.len() < required_len {
            return false;
        }

        self.context()
            .items
            .iter()
            .skip(1)
            .take(count)
            .all(|data| data.donchian.sample_count() >= self.config_period())
    }

    fn is_current_price_breakout_above(&self, current_price: f64) -> bool {
        self.has_prior_channels()
            && self
                .previous_donchian_data()
                .map(|data| current_price > data.donchian.upper())
                .unwrap_or(false)
    }

    fn is_current_price_breakout_below(&self, current_price: f64) -> bool {
        self.has_prior_channels()
            && self
                .previous_donchian_data()
                .map(|data| current_price < data.donchian.lower())
                .unwrap_or(false)
    }

    fn is_current_price_below_previous_middle(&self, current_price: f64) -> bool {
        self.previous_donchian_data()
            .map(|data| current_price < data.donchian.middle())
            .unwrap_or(false)
    }

    fn is_current_price_above_previous_middle(&self, current_price: f64) -> bool {
        self.previous_donchian_data()
            .map(|data| current_price > data.donchian.middle())
            .unwrap_or(false)
    }
}
