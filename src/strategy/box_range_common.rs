use super::Strategy;
use super::config_utils;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

pub use crate::analyzer::box_range_analyzer::{BoxRangeAnalyzer, BoxRangeAnalyzerData};

/// 박스권 전략 공통 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct BoxRangeStrategyConfigBase {
    /// 돌파 전 확인할 박스권 데이터 수
    pub count: usize,
    /// 박스권 계산 기간
    pub period: usize,
    /// 박스권 판정 최대 폭 비율
    pub max_width_ratio: f64,
}

impl ConfigValidation for BoxRangeStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "박스권 확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.period == 0 {
            return Err(ConfigError::ValidationError(
                "박스권 계산 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        if !self.max_width_ratio.is_finite() || self.max_width_ratio <= 0.0 {
            return Err(ConfigError::ValidationError(
                "박스권 최대 폭 비율은 유한한 양수여야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl BoxRangeStrategyConfigBase {
    /// JSON 문자열에서 설정 로드
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str::<T>(json).map_err(|e| format!("JSON 설정 역직렬화 실패: {e}"))
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<BoxRangeStrategyConfigBase, String> {
        let count = config_utils::parse_usize(config, "count", Some(1), true)?
            .ok_or("count 설정이 필요합니다")?;
        let period = config_utils::parse_usize(config, "period", Some(1), true)?
            .ok_or("period 설정이 필요합니다")?;
        let max_width_ratio =
            config_utils::parse_f64(config, "max_width_ratio", Some((0.0, f64::MAX)), true)?
                .ok_or("max_width_ratio 설정이 필요합니다")?;

        let result = BoxRangeStrategyConfigBase {
            count,
            period,
            max_width_ratio,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// 박스권 전략 공통 동작
pub trait BoxRangeStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &BoxRangeAnalyzer<C>;

    /// 돌파 전 확인할 박스권 데이터 수 반환
    fn config_count(&self) -> usize;

    /// 직전 박스권 데이터 반환
    fn previous_box_data(&self) -> Option<&BoxRangeAnalyzerData<C>> {
        self.context().items.get(1)
    }

    /// 진입 전에 필요한 직전 박스권들이 모두 유효한지 확인
    fn has_prior_box_ranges(&self) -> bool {
        let count = self.config_count();

        if self.context().items.len() < count + 1 {
            return false;
        }

        self.context()
            .items
            .iter()
            .skip(1)
            .take(count)
            .all(|data| data.box_range.is_box_range())
    }

    /// 현재가가 직전 박스권 상단을 돌파했는지 확인
    fn is_current_price_breakout_above(&self, current_price: f64) -> bool {
        self.has_prior_box_ranges()
            && self
                .previous_box_data()
                .map(|data| current_price > data.box_range.upper())
                .unwrap_or(false)
    }

    /// 현재가가 직전 박스권 하단을 이탈했는지 확인
    fn is_current_price_breakout_below(&self, current_price: f64) -> bool {
        self.has_prior_box_ranges()
            && self
                .previous_box_data()
                .map(|data| current_price < data.box_range.lower())
                .unwrap_or(false)
    }

    /// 롱 청산용: 현재가가 직전 박스권 중간선 아래로 되돌아왔는지 확인
    fn is_current_price_below_previous_middle(&self, current_price: f64) -> bool {
        self.previous_box_data()
            .map(|data| current_price < data.box_range.middle())
            .unwrap_or(false)
    }

    /// 숏 청산용: 현재가가 직전 박스권 중간선 위로 회복했는지 확인
    fn is_current_price_above_previous_middle(&self, current_price: f64) -> bool {
        self.previous_box_data()
            .map(|data| current_price > data.box_range.middle())
            .unwrap_or(false)
    }
}
