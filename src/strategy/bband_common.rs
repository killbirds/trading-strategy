use crate::config_loader::{ConfigError, ConfigResult, ConfigValidation};
use log::{debug, error, info};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use std::collections::HashMap;
use std::path::Path;

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

        Ok(())
    }
}

impl BBandStrategyConfigBase {
    /// 설정 파일에서 로드
    pub fn from_file<T>(path: &Path) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
    {
        debug!("볼린저 밴드 전략 설정 파일 로드 시작: {}", path.display());
        crate::config_loader::ConfigLoader::load_from_file(
            path,
            crate::config_loader::ConfigFormat::Auto,
        )
    }

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

        let result = BBandStrategyConfigBase {
            count,
            period,
            multiplier,
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
