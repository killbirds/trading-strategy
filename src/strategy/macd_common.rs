use super::Strategy;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

// analyzer에서 MACDAnalyzer 관련 구조체 가져오기
pub use crate::analyzer::macd_analyzer::{MACDAnalyzer, MACDAnalyzerData};

/// MACD 전략 공통 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct MACDStrategyConfigBase {
    /// 빠른 EMA 기간
    pub fast_period: usize,
    /// 느린 EMA 기간
    pub slow_period: usize,
    /// 시그널 라인 기간
    pub signal_period: usize,
    /// 히스토그램 임계값
    pub histogram_threshold: f64,
    /// 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
    pub confirm_period: usize,
}

impl ConfigValidation for MACDStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.fast_period < 2 {
            return Err(ConfigError::ValidationError(
                "빠른 EMA 기간은 2 이상이어야 합니다".to_string(),
            ));
        }

        if self.slow_period <= self.fast_period {
            return Err(ConfigError::ValidationError(format!(
                "느린 EMA 기간({})은 빠른 EMA 기간({})보다 커야 합니다",
                self.slow_period, self.fast_period
            )));
        }

        if self.signal_period < 1 {
            return Err(ConfigError::ValidationError(
                "시그널 EMA 기간은 1 이상이어야 합니다".to_string(),
            ));
        }

        if self.confirm_period < 1 {
            return Err(ConfigError::ValidationError(
                "신호 확인 기간은 1 이상이어야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl MACDStrategyConfigBase {
    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str, _is_long_strategy: bool) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {e}")),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
        is_long_strategy: bool,
    ) -> Result<MACDStrategyConfigBase, String> {
        // 빠른 EMA 기간 설정
        let fast_period = match config.get("fast_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "빠른 EMA 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("빠른 EMA 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("fast_period 설정이 필요합니다".to_string()),
        };

        // 느린 EMA 기간 설정
        let slow_period = match config.get("slow_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "느린 EMA 기간 파싱 오류".to_string())?;

                if period <= fast_period {
                    return Err(format!(
                        "느린 EMA 기간({period})은 빠른 EMA 기간({fast_period})보다 커야 합니다"
                    ));
                }

                period
            }
            None => return Err("slow_period 설정이 필요합니다".to_string()),
        };

        // 시그널 EMA 기간 설정
        let signal_period = match config.get("signal_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "시그널 EMA 기간 파싱 오류".to_string())?;

                if period < 1 {
                    return Err("시그널 EMA 기간은 1 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("signal_period 설정이 필요합니다".to_string()),
        };

        // 히스토그램 임계값 설정
        let histogram_threshold = match config.get("histogram_threshold") {
            Some(threshold_str) => {
                let threshold = threshold_str
                    .parse::<f64>()
                    .map_err(|_| "히스토그램 임계값 파싱 오류".to_string())?;

                // 롱 전략인 경우 임계값 검증
                if is_long_strategy && threshold < 0.0 {
                    return Err(format!(
                        "롱 전략의 히스토그램 임계값({threshold})은 0 이상이어야 합니다"
                    ));
                }

                // 숏 전략인 경우 임계값 검증
                if !is_long_strategy && threshold > 0.0 {
                    return Err(format!(
                        "숏 전략의 히스토그램 임계값({threshold})은 0보다 작아야 합니다"
                    ));
                }

                threshold
            }
            None => return Err("histogram_threshold 설정이 필요합니다".to_string()),
        };

        // 신호 확인 기간 설정
        let confirm_period = match config.get("confirm_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "신호 확인 기간 파싱 오류".to_string())?;

                if period < 1 {
                    return Err("신호 확인 기간은 1 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("confirm_period 설정이 필요합니다".to_string()),
        };

        let result = MACDStrategyConfigBase {
            fast_period,
            slow_period,
            signal_period,
            histogram_threshold,
            confirm_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// MACD 전략을 위한 공통 트레이트
pub trait MACDStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &MACDAnalyzer<C>;

    /// 설정의 confirm_period 반환
    fn config_confirm_period(&self) -> usize;

    /// 설정의 histogram_threshold 반환
    fn config_histogram_threshold(&self) -> f64;
}
