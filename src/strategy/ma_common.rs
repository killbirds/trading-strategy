use crate::analyzer::AnalyzerOps;
use crate::indicator::ma::MAType;
use crate::strategy::Strategy;
use crate::strategy::split_safe;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

// analyzer에서 MA 관련 구조체 가져오기
pub use crate::analyzer::ma_analyzer::{MAAnalyzer, MAAnalyzerData};

/// 이동평균(MA) 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct MAStrategyConfigBase {
    /// 이동평균 계산 방식 (SMA, EMA 등)
    pub ma: MAType,
    /// 이동평균 기간 목록 (짧은 것부터 긴 것 순)
    pub ma_periods: Vec<usize>,
    /// 골든 크로스/데드 크로스 판정 조건: 이전 기간
    pub cross_previous_periods: usize,
}

impl MAStrategyConfigBase {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.ma_periods.is_empty() {
            return Err("이동평균 기간이 지정되지 않았습니다".to_string());
        }

        // 기간이 오름차순으로 정렬되어 있는지 확인
        for i in 1..self.ma_periods.len() {
            if self.ma_periods[i] <= self.ma_periods[i - 1] {
                return Err(format!(
                    "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                    self.ma_periods
                ));
            }
        }

        if self.cross_previous_periods == 0 {
            return Err("크로스 판정 기간은 0보다 커야 합니다".to_string());
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
    pub fn from_hash_map(config: &HashMap<String, String>) -> Result<MAStrategyConfigBase, String> {
        // ma 유형 설정
        let ma = match config.get("ma") {
            Some(ma_type) => match ma_type.to_lowercase().as_str() {
                "sma" => MAType::SMA,
                "ema" => MAType::EMA,
                "wma" => MAType::WMA,
                _ => return Err(format!("알 수 없는 이동평균 유형: {}", ma_type)),
            },
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods) => {
                let periods_vec = split_safe::<usize>(periods)
                    .map_err(|e| format!("이동평균 기간 파싱 오류: {}", e))?;

                if periods_vec.is_empty() {
                    return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                }

                // 기간이 오름차순으로 정렬되어 있는지 확인
                for i in 1..periods_vec.len() {
                    if periods_vec[i] <= periods_vec[i - 1] {
                        return Err(format!(
                            "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                            periods_vec
                        ));
                    }
                }

                periods_vec
            }
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        // 크로스 판정 기간 설정
        let cross_previous_periods = match config.get("cross_previous_periods") {
            Some(cross_periods) => {
                let periods = cross_periods
                    .parse::<usize>()
                    .map_err(|_| "크로스 판정 기간 파싱 오류".to_string())?;

                if periods == 0 {
                    return Err("크로스 판정 기간은 0보다 커야 합니다".to_string());
                }

                periods
            }
            None => return Err("cross_previous_periods 설정이 필요합니다".to_string()),
        };

        let result = MAStrategyConfigBase {
            ma,
            ma_periods,
            cross_previous_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// MA 전략을 위한 공통 트레이트
pub trait MAStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &MAAnalyzer<C>;

    /// 설정 참조 반환
    fn config_cross_previous_periods(&self) -> usize;

    /// 주어진 함수 조건에 따라 교차 여부 확인
    fn check_cross_condition(
        &self,
        condition_fn: impl Fn(&MAAnalyzerData<C>) -> bool + Copy,
    ) -> bool {
        self.context().is_break_through_by_satisfying(
            condition_fn,
            1,
            self.config_cross_previous_periods(),
        )
    }
}
