use super::Strategy;
use super::split;
use crate::indicator::ma::MAType;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

// analyzer에서 ThreeRSIAnalyzer 관련 구조체 가져오기
pub use crate::analyzer::three_rsi_analyzer::{ThreeRSIAnalyzer, ThreeRSIAnalyzerData};

/// 세 개의 RSI를 사용하는 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct ThreeRSIStrategyConfigBase {
    /// 세 가지 RSI 기간
    pub rsi_periods: Vec<usize>,
    /// 이동평균 유형
    pub ma: MAType,
    /// 이동평균 계산 기간
    pub ma_period: usize,
    /// ADX 계산 기간
    pub adx_period: usize,
}

impl Default for ThreeRSIStrategyConfigBase {
    /// 기본 설정값 반환
    fn default() -> Self {
        ThreeRSIStrategyConfigBase {
            rsi_periods: vec![6, 14, 26],
            ma: MAType::EMA,
            ma_period: 50,
            adx_period: 14,
        }
    }
}

impl ThreeRSIStrategyConfigBase {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_periods.is_empty() {
            return Err("RSI 기간은 최소 하나 이상 지정해야 합니다".to_string());
        }

        for period in &self.rsi_periods {
            if *period == 0 {
                return Err("RSI 기간은 0보다 커야 합니다".to_string());
            }
        }

        if self.ma_period == 0 {
            return Err("이동평균 기간은 0보다 커야 합니다".to_string());
        }

        if self.adx_period == 0 {
            return Err("ADX 기간은 0보다 커야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ThreeRSIStrategyConfigBase, String> {
        // RSI 기간 설정
        let rsi_periods = match config.get("rsi_periods") {
            Some(periods_str) => match split(periods_str) {
                Ok(periods) => {
                    if periods.is_empty() {
                        return Err("RSI 기간은 최소 하나 이상 지정해야 합니다".to_string());
                    }
                    for period in &periods {
                        if *period == 0 {
                            return Err("RSI 기간은 0보다 커야 합니다".to_string());
                        }
                    }
                    periods
                }
                Err(e) => return Err(format!("RSI 기간 파싱 오류: {}", e)),
            },
            None => return Err("rsi_periods 설정이 필요합니다".to_string()),
        };

        // 이동평균 유형 설정
        let ma = match config.get("ma").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_period = match config.get("ma_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "이동평균 기간 파싱 오류".to_string())?;

                if period == 0 {
                    return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                }

                period
            }
            None => return Err("ma_period 설정이 필요합니다".to_string()),
        };

        // ADX 기간 설정
        let adx_period = match config.get("adx_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "ADX 기간 파싱 오류".to_string())?;

                if period == 0 {
                    return Err("ADX 기간은 0보다 커야 합니다".to_string());
                }

                period
            }
            None => return Err("adx_period 설정이 필요합니다".to_string()),
        };

        let result = ThreeRSIStrategyConfigBase {
            rsi_periods,
            ma,
            ma_period,
            adx_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// ThreeRSI 전략의 공통 트레이트
pub trait ThreeRSIStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &ThreeRSIAnalyzer<C>;
}
