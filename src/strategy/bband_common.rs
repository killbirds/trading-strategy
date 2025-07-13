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
                let error_msg = format!("JSON 설정 역직렬화 실패: {}", e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<BBandStrategyConfigBase, String> {
        debug!("볼린저 밴드 전략 HashMap 설정 파싱 시작: {:?}", config);

        // 확인 캔들 수 설정
        let count = match config.get("count") {
            Some(count_str) => {
                debug!("count 설정 파싱: {}", count_str);
                let count = match count_str.parse::<usize>() {
                    Ok(c) => c,
                    Err(e) => {
                        let error_msg = format!("확인 캔들 수 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if count == 0 {
                    let error_msg = "확인 캔들 수는 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                count
            }
            None => {
                let error_msg = "count 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // 볼린저 밴드 계산 기간 설정
        let period = match config.get("period") {
            Some(period_str) => {
                debug!("period 설정 파싱: {}", period_str);
                let period = match period_str.parse::<usize>() {
                    Ok(p) => p,
                    Err(e) => {
                        let error_msg = format!("볼린저 밴드 계산 기간 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if period < 2 {
                    let error_msg = "볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                period
            }
            None => {
                let error_msg = "period 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // 볼린저 밴드 승수 설정
        let multiplier = match config.get("multiplier") {
            Some(multiplier_str) => {
                debug!("multiplier 설정 파싱: {}", multiplier_str);
                let multiplier = match multiplier_str.parse::<f64>() {
                    Ok(m) => m,
                    Err(e) => {
                        let error_msg = format!("볼린저 밴드 승수 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if multiplier <= 0.0 {
                    let error_msg = "볼린저 밴드 승수는 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                multiplier
            }
            None => {
                let error_msg = "multiplier 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // 밴드 폭 감소 확인 기간 설정 (선택적)
        let narrowing_period = match config.get("narrowing_period") {
            Some(narrowing_period_str) => {
                debug!("narrowing_period 설정 파싱: {}", narrowing_period_str);
                let narrowing_period = match narrowing_period_str.parse::<usize>() {
                    Ok(np) => np,
                    Err(e) => {
                        let error_msg = format!("밴드 폭 감소 확인 기간 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if narrowing_period == 0 {
                    let error_msg = "밴드 폭 감소 확인 기간은 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                narrowing_period
            }
            None => {
                debug!("narrowing_period 설정이 없음, 기본값 사용: 5");
                5 // 기본값
            }
        };

        // 좁은 상태 유지 기간 설정 (선택적)
        let squeeze_period = match config.get("squeeze_period") {
            Some(squeeze_period_str) => {
                debug!("squeeze_period 설정 파싱: {}", squeeze_period_str);
                let squeeze_period = match squeeze_period_str.parse::<usize>() {
                    Ok(sp) => sp,
                    Err(e) => {
                        let error_msg = format!("좁은 상태 유지 기간 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if squeeze_period == 0 {
                    let error_msg = "좁은 상태 유지 기간은 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                squeeze_period
            }
            None => {
                debug!("squeeze_period 설정이 없음, 기본값 사용: 5");
                5 // 기본값
            }
        };

        // 스퀴즈 임계값 설정 (선택적)
        let squeeze_threshold = match config.get("squeeze_threshold") {
            Some(squeeze_threshold_str) => {
                debug!("squeeze_threshold 설정 파싱: {}", squeeze_threshold_str);
                let squeeze_threshold = match squeeze_threshold_str.parse::<f64>() {
                    Ok(st) => st,
                    Err(e) => {
                        let error_msg = format!("스퀴즈 임계값 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if squeeze_threshold < 0.0 {
                    let error_msg = "스퀴즈 임계값은 0 이상이어야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                squeeze_threshold
            }
            None => {
                debug!("squeeze_threshold 설정이 없음, 기본값 사용: 0.02");
                0.02 // 기본값 (2%)
            }
        };

        let result = BBandStrategyConfigBase {
            count,
            period,
            multiplier,
            narrowing_period,
            squeeze_period,
            squeeze_threshold,
        };

        debug!("볼린저 밴드 전략 설정 생성 완료: {:?}", result);

        // 유효성 검사
        if let Err(e) = result.validate() {
            error!("볼린저 밴드 전략 설정 유효성 검사 실패: {}", e);
            return Err(e.to_string());
        }

        info!("볼린저 밴드 전략 설정 로드 완료: {:?}", result);
        Ok(result)
    }
}
