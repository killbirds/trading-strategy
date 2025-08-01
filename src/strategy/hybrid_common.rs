use super::Strategy;
use crate::indicator::ma::MAType;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use trading_chart::Candle;

// analyzer에서 HybridAnalyzer 관련 구조체 가져오기
pub use crate::analyzer::hybrid_analyzer::{HybridAnalyzer, HybridAnalyzerData};

/// 하이브리드 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct HybridStrategyConfigBase {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
    pub rsi_lower: f64,
    /// RSI 조건 판단 기간
    pub rsi_count: usize,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 승수
    pub bband_multiplier: f64,
    /// 마켓 순위 정렬 기준 이동평균 기간
    pub ma_rank_period: usize,
    /// 이동평균 타입
    pub ma_type: MAType,
    /// 이동평균 기간
    pub ma_period: usize,
    /// MACD 빠른 기간
    pub macd_fast_period: usize,
    /// MACD 느린 기간
    pub macd_slow_period: usize,
    /// MACD 시그널 기간
    pub macd_signal_period: usize,
}

impl Default for HybridStrategyConfigBase {
    /// 기본 설정값 반환
    fn default() -> Self {
        HybridStrategyConfigBase {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            rsi_count: 3,
            bband_period: 20,
            bband_multiplier: 2.0,
            ma_rank_period: 20,
            ma_type: MAType::SMA,
            ma_period: 20,
            macd_fast_period: 12,
            macd_slow_period: 26,
            macd_signal_period: 9,
        }
    }
}

impl HybridStrategyConfigBase {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_period < 2 {
            return Err("RSI 기간은 2 이상이어야 합니다".to_string());
        }

        if self.rsi_lower >= self.rsi_upper {
            return Err(format!(
                "RSI 하한값({})이 상한값({})보다 크거나 같을 수 없습니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 승수는 0보다 커야 합니다".to_string());
        }

        if self.ma_rank_period < 2 {
            return Err("마켓 랭크 이동평균 기간은 2 이상이어야 합니다".to_string());
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
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {e}")),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<HybridStrategyConfigBase, String> {
        // 카운트 설정
        let count = match config.get("count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "카운트 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("카운트는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("count 설정이 필요합니다".to_string()),
        };

        // 이동평균 관련 설정
        let ma_type = match config.get("ma_type").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {unknown}")),
            None => return Err("ma_type 설정이 필요합니다".to_string()),
        };

        let ma_period = match config.get("ma_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "이동평균 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("ma_period 설정이 필요합니다".to_string()),
        };

        // MACD 관련 설정
        let macd_fast_period = match config.get("macd_fast_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 빠른 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 빠른 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_fast_period 설정이 필요합니다".to_string()),
        };

        let macd_slow_period = match config.get("macd_slow_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 느린 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 느린 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_slow_period 설정이 필요합니다".to_string()),
        };

        if macd_fast_period >= macd_slow_period {
            return Err(format!(
                "MACD 빠른 기간({macd_fast_period})은 느린 기간({macd_slow_period})보다 작아야 합니다"
            ));
        }

        let macd_signal_period = match config.get("macd_signal_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 시그널 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 시그널 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_signal_period 설정이 필요합니다".to_string()),
        };

        // RSI 관련 설정
        let rsi_period = match config.get("rsi_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 하한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&value) {
                    return Err(format!("RSI 하한값({value})은 0과 100 사이여야 합니다"));
                }

                value
            }
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 상한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&value) {
                    return Err(format!("RSI 상한값({value})은 0과 100 사이여야 합니다"));
                }

                value
            }
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하한값({rsi_lower})은 상한값({rsi_upper})보다 작아야 합니다"
            ));
        }

        let result = HybridStrategyConfigBase {
            rsi_count: count,
            ma_type,
            ma_period,
            macd_fast_period,
            macd_slow_period,
            macd_signal_period,
            rsi_period,
            rsi_lower,
            rsi_upper,
            bband_period: config
                .get("bband_period")
                .map_or(20, |s| s.parse::<usize>().unwrap()),
            bband_multiplier: config
                .get("bband_multiplier")
                .map_or(2.0, |s| s.parse::<f64>().unwrap()),
            ma_rank_period: config
                .get("ma_rank_period")
                .map_or(20, |s| s.parse::<usize>().unwrap()),
        };

        result.validate()?;
        Ok(result)
    }
}

/// 하이브리드 전략 공통 트레이트
pub trait HybridStrategyCommon<C: Candle + Clone + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &HybridAnalyzer<C>;

    /// 설정 기본값 참조 반환
    fn config_base(&self) -> &HybridStrategyConfigBase;

    /// 매수 신호 강도 계산
    fn calculate_buy_signal_strength(&self) -> f64 {
        let ctx = self.context();

        if ctx.items.len() < 2 {
            return 0.0;
        }

        let current = ctx.items.last().unwrap();
        let previous = &ctx.items[ctx.items.len() - 2];
        let config = self.config_base();

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() > current.ma.get() {
            strength += 1.0;
            count += 1.0;

            // 상승 추세 강화: 가격이 이동평균보다 2% 이상 높으면 추가 점수
            if current.candle.close_price() > current.ma.get() * 1.02 {
                strength += 0.5;
            }
        }

        // 2. MACD 기반 신호
        if current.macd.histogram > 0.0 && previous.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선을 상향 돌파 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선 위에 있음 (약한 매수 신호)
            strength += 0.8; // 0.5에서 0.8로 증가
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi < config.rsi_lower && rsi > previous.rsi.value() {
            // RSI가 과매도 상태에서 반등 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi > config.rsi_lower && rsi < 50.0 {
            // RSI가 과매도 상태를 벗어나 상승 중 (약한 매수 신호)
            strength += 0.8; // 0.5에서 0.8로 증가
            count += 1.0;
        } else if rsi > 50.0 && rsi < 70.0 && rsi > previous.rsi.value() {
            // RSI가 50-70 구간에서 상승 중 (상승 추세 형성) - 범위 확장
            strength += 0.7;
            count += 0.8;
        } else if rsi > 70.0 && rsi < 90.0 && rsi > previous.rsi.value() {
            // RSI가 70-90 구간에서 상승 중 (강한 상승 추세) - 가중치 상향
            strength += 1.0;
            count += 1.0;
        } else if rsi > 90.0 && rsi <= 100.0 && rsi > previous.rsi.value() {
            // RSI가 90-100 구간에서 상승 중 (극강한 상승 추세) - 추가 조건
            strength += 0.3;
            count += 0.4;
        }

        // 4. 가격 변동 패턴 (추가)
        if current.candle.close_price() > previous.candle.close_price() {
            // 종가가 전일 종가보다 높음 (상승 추세)
            strength += 0.5;
            count += 0.5;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / count // 정규화 방식을 단순화하여 더 높은 신호 강도 생성
        } else {
            0.0
        }
    }

    /// 매도 신호 강도 계산
    fn calculate_sell_signal_strength(&self, profit_percentage: f64) -> f64 {
        let ctx = self.context();

        if ctx.items.len() < 2 {
            return 0.0;
        }

        let current = ctx.items.last().unwrap();
        let previous = &ctx.items[ctx.items.len() - 2];
        let config = self.config_base();

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() < current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호
        if current.macd.histogram < 0.0 && previous.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선을 하향 돌파 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선 아래에 있음 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi > config.rsi_upper && rsi < previous.rsi.value() {
            // RSI가 과매수 상태에서 하락 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi < config.rsi_upper && rsi > 50.0 {
            // RSI가 과매수 상태로 접근 중 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 4. 수익률 기반 신호
        if profit_percentage > 5.0 {
            // 5% 이상 수익 (적절한 매도 신호)
            strength += 1.0;
            count += 1.0;
        } else if profit_percentage < -3.0 {
            // 3% 이상 손실 (손절 매도 신호)
            strength += 1.5;
            count += 1.0;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / (count * 2.0) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        }
    }
}
