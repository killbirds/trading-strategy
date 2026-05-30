use super::Strategy;
use super::config_utils;
use crate::indicator::ma::MAType;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

// analyzer에서 RSIAnalyzer 관련 구조체 가져오기
pub use crate::analyzer::rsi_analyzer::{RSIAnalyzer, RSIAnalyzerData};

/// RSI 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct RSIStrategyConfigBase {
    /// RSI 판단에 필요한 연속 데이터 수
    pub rsi_count: usize,
    /// RSI 하단 기준값 (매수 신호용)
    pub rsi_lower: f64,
    /// RSI 상단 기준값 (매도 신호용)
    pub rsi_upper: f64,
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// 이동평균 유형 (EMA, SMA 등)
    pub ma: MAType,
    /// 이동평균 기간 목록 (여러 이동평균선 사용)
    pub ma_periods: Vec<usize>,
}

impl ConfigValidation for RSIStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.rsi_period < 2 {
            return Err(ConfigError::ValidationError(
                "RSI 기간은 2 이상이어야 합니다".to_string(),
            ));
        }

        if self.rsi_lower >= self.rsi_upper {
            return Err(ConfigError::ValidationError(format!(
                "RSI 하한({})은 상한({})보다 작아야 합니다",
                self.rsi_lower, self.rsi_upper
            )));
        }

        if self.rsi_count == 0 {
            return Err(ConfigError::ValidationError(
                "RSI 판정 횟수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.ma_periods.is_empty() {
            return Err(ConfigError::ValidationError(
                "이동평균 기간이 지정되지 않았습니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl RSIStrategyConfigBase {
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
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {e}")),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<RSIStrategyConfigBase, String> {
        // 공통 유틸리티를 사용하여 RSI 설정 파싱
        let (rsi_period, rsi_lower, rsi_upper) = config_utils::parse_rsi_config(config)?;

        // RSI 카운트 설정
        let rsi_count = config_utils::parse_usize(config, "rsi_count", Some(1), true)?
            .ok_or("rsi_count 설정이 필요합니다")?;

        // MA 유형 설정
        let ma = config_utils::parse_ma_type(config, None, true)?.ok_or("ma 설정이 필요합니다")?;

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods) => {
                let periods_vec = crate::strategy::split_safe::<usize>(periods)
                    .map_err(|e| format!("이동평균 기간 파싱 오류: {e}"))?;

                if periods_vec.is_empty() {
                    return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                }

                periods_vec
            }
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        let result = RSIStrategyConfigBase {
            rsi_count,
            rsi_lower,
            rsi_upper,
            rsi_period,
            ma,
            ma_periods,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// RSI 전략을 위한 공통 트레이트
pub trait RSIStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &RSIAnalyzer<C>;

    /// 설정의 rsi_lower 반환
    fn config_rsi_lower(&self) -> f64;

    /// 설정의 rsi_upper 반환
    fn config_rsi_upper(&self) -> f64;

    /// 설정의 rsi_count 반환
    fn config_rsi_count(&self) -> usize;

    /// RSI가 과매수 영역인지 확인
    fn is_rsi_overbought(&self) -> bool {
        if self.context().items.len() < self.config_rsi_count() {
            return false;
        }

        // 과매수 판단: RSI가 상단 임계값을 넘어서면 과매수로 판단
        self.context()
            .items
            .iter()
            .take(self.config_rsi_count())
            .all(|item| item.rsi.value > self.config_rsi_upper())
    }

    /// RSI가 과매도 영역인지 확인
    fn is_rsi_oversold(&self) -> bool {
        if self.context().items.len() < self.config_rsi_count() {
            return false;
        }

        // 과매도 판단: RSI가 하단 임계값 아래로 내려가면 과매도로 판단
        self.context()
            .items
            .iter()
            .take(self.config_rsi_count())
            .all(|item| item.rsi.value < self.config_rsi_lower())
    }
}
