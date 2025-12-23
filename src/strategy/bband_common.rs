use super::config_utils;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use log::{debug, error, info};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;

// analyzer에서 BBandAnalyzer 및 BBandAnalyzerData 가져오기
pub use crate::analyzer::bband_analyzer::{BBandAnalyzer, BBandAnalyzerData};

/// 볼린저 밴드 전략 공통 설정 베이스
#[derive(Debug, Deserialize, Serialize)]
pub struct BBandStrategyConfigBase {
    /// 확인 캔들 수
    pub count: usize,
    /// 볼린저 밴드 계산 기간
    pub period: usize,
    /// 볼린저 밴드 승수 (표준편차 배수)
    pub multiplier: f64,
    /// 밴드 폭 감소 확인 기간
    pub narrowing_period: usize,
    /// 좁은 상태 유지 기간
    pub squeeze_period: usize,
    /// 스퀴즈 조건을 위한 최소 밴드 폭 (비율)
    pub squeeze_threshold: f64,
}

impl ConfigValidation for BBandStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.period < 2 {
            return Err(ConfigError::ValidationError(
                "볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string(),
            ));
        }

        if self.multiplier <= 0.0 {
            return Err(ConfigError::ValidationError(
                "볼린저 밴드 승수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.narrowing_period == 0 {
            return Err(ConfigError::ValidationError(
                "밴드 폭 감소 확인 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        if self.squeeze_period == 0 {
            return Err(ConfigError::ValidationError(
                "좁은 상태 유지 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        if self.squeeze_threshold < 0.0 {
            return Err(ConfigError::ValidationError(
                "스퀴즈 임계값은 0 이상이어야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl BBandStrategyConfigBase {
    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<BBandStrategyConfigBase, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("볼린저 밴드 전략 JSON 설정 파싱 시작");
        match serde_json::from_str::<T>(json) {
            Ok(config) => {
                debug!("볼린저 밴드 전략 JSON 설정 파싱 성공");
                Ok(config)
            }
            Err(e) => {
                let error_msg = format!("JSON 설정 역직렬화 실패: {e}");
                error!("{error_msg}");
                Err(error_msg)
            }
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<BBandStrategyConfigBase, String> {
        debug!("볼린저 밴드 전략 HashMap 설정 파싱 시작: {config:?}");

        // 공통 유틸리티를 사용하여 설정 파싱
        let count = config_utils::parse_usize(config, "count", Some(1), true)
            .map_err(|e| {
                error!("{e}");
                e
            })?
            .ok_or_else(|| {
                let error_msg = "count 설정이 필요합니다".to_string();
                error!("{error_msg}");
                error_msg
            })?;

        let period = config_utils::parse_usize(config, "period", Some(2), true)
            .map_err(|e| {
                error!("{e}");
                e
            })?
            .ok_or_else(|| {
                let error_msg = "period 설정이 필요합니다".to_string();
                error!("{error_msg}");
                error_msg
            })?;

        let multiplier = config_utils::parse_f64(config, "multiplier", Some((0.0, f64::MAX)), true)
            .map_err(|e| {
                error!("{e}");
                e
            })?
            .ok_or_else(|| {
                let error_msg = "multiplier 설정이 필요합니다".to_string();
                error!("{error_msg}");
                error_msg
            })?;

        if multiplier <= 0.0 {
            let error_msg = "볼린저 밴드 승수는 0보다 커야 합니다".to_string();
            error!("{error_msg}");
            return Err(error_msg);
        }

        // 공통 유틸리티를 사용하여 선택적 설정 파싱
        let narrowing_period =
            config_utils::parse_usize(config, "narrowing_period", Some(1), false)
                .map_err(|e| {
                    error!("{e}");
                    e
                })?
                .unwrap_or(5);

        let squeeze_period = config_utils::parse_usize(config, "squeeze_period", Some(1), false)
            .map_err(|e| {
                error!("{e}");
                e
            })?
            .unwrap_or(5);

        let squeeze_threshold =
            config_utils::parse_f64(config, "squeeze_threshold", Some((0.0, f64::MAX)), false)
                .map_err(|e| {
                    error!("{e}");
                    e
                })?
                .unwrap_or(0.02);

        let result = BBandStrategyConfigBase {
            count,
            period,
            multiplier,
            narrowing_period,
            squeeze_period,
            squeeze_threshold,
        };

        debug!("볼린저 밴드 전략 설정 생성 완료: {result:?}");

        // 유효성 검사
        if let Err(e) = result.validate() {
            error!("볼린저 밴드 전략 설정 유효성 검사 실패: {e}");
            return Err(e.to_string());
        }

        info!("볼린저 밴드 전략 설정 로드 완료: {result:?}");
        Ok(result)
    }
}
