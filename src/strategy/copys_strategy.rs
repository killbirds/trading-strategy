use super::Strategy;
use super::StrategyType;
use super::copys_common::{CopysStrategyCommon, CopysStrategyConfigBase, CopysStrategyContext};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// CopyS 전략 설정
#[derive(Debug, Deserialize)]
pub struct CopysStrategyConfig {
    #[serde(flatten)]
    pub base: CopysStrategyConfigBase,
}

impl Default for CopysStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        CopysStrategyConfig {
            base: CopysStrategyConfigBase {
                rsi_period: 14,
                rsi_upper: 70.0,
                rsi_lower: 30.0,
                bband_period: 20,
                bband_multiplier: 2.0,
            },
        }
    }
}

impl CopysStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        // 기본 설정 유효성 검사
        self.base.validate()?;

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<CopysStrategyConfig, String> {
        match CopysStrategyConfigBase::from_json::<CopysStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<CopysStrategyConfig, String> {
        // RSI 관련 설정
        let rsi_period = match config.get("rsi_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("RSI 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(lower_str) => {
                let lower = lower_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 하한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&lower) {
                    return Err(format!("RSI 하한값({})은 0과 100 사이여야 합니다", lower));
                }

                lower
            }
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(upper_str) => {
                let upper = upper_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 상한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&upper) {
                    return Err(format!("RSI 상한값({})은 0과 100 사이여야 합니다", upper));
                }

                upper
            }
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하한값({})은 상한값({})보다 작아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        // 볼린저 밴드 관련 설정
        let bband_period = match config.get("bband_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저 밴드 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("볼린저 밴드 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("bband_period 설정이 필요합니다".to_string()),
        };

        let bband_multiplier = match config.get("bband_multiplier") {
            Some(multiplier_str) => {
                let multiplier = multiplier_str
                    .parse::<f64>()
                    .map_err(|_| "볼린저 밴드 승수 파싱 오류".to_string())?;

                if multiplier <= 0.0 {
                    return Err("볼린저 밴드 승수는 0보다 커야 합니다".to_string());
                }

                multiplier
            }
            None => return Err("bband_multiplier 설정이 필요합니다".to_string()),
        };

        Ok(CopysStrategyConfig {
            base: CopysStrategyConfigBase {
                rsi_period,
                rsi_lower,
                rsi_upper,
                bband_period,
                bband_multiplier,
            },
        })
    }
}

pub struct CopysStrategy<C: Candle> {
    config: CopysStrategyConfig,
    ctx: CopysStrategyContext<C>,
    bband_analyzer: BBandAnalyzer<C>,
}

impl<C: Candle> Display for CopysStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Copys전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({})}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.bband_period,
            self.config.base.bband_multiplier,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> CopysStrategy<C> {
    /// 새 CopyS A 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<CopysStrategy<C>, String> {
        let config = CopysStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 CopyS 전략 인스턴스 생성
    pub fn new(
        storage: &CandleStore<C>,
        config: CopysStrategyConfig,
    ) -> Result<CopysStrategy<C>, String> {
        info!("CopyS 전략 설정: {:?}", config);

        let ma_type = crate::indicator::ma::MAType::EMA;
        // 이미지 참고: 5일/20일/60일/120일/200일/240일 이평선 설정
        let ma_periods = [5, 20, 60, 120, 200, 240];
        let ctx = CopysStrategyContext::new(config.base.rsi_period, &ma_type, &ma_periods, storage);

        // 볼린저밴드 분석기 생성
        let bband_analyzer = BBandAnalyzer::new(
            config.base.bband_period,
            config.base.bband_multiplier,
            storage,
        );

        Ok(CopysStrategy {
            config,
            ctx,
            bband_analyzer,
        })
    }

    /// 새 CopyS 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<CopysStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => CopysStrategyConfig::from_hash_map(&cfg)?,
            None => CopysStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }

    /// 볼린저밴드 분석기 참조 반환
    pub fn bband_analyzer(&self) -> &BBandAnalyzer<C> {
        &self.bband_analyzer
    }
}

impl<C: Candle + 'static> CopysStrategyCommon<C> for CopysStrategy<C> {
    fn context(&self) -> &CopysStrategyContext<C> {
        &self.ctx
    }

    fn bband_analyzer(&self) -> &BBandAnalyzer<C> {
        &self.bband_analyzer
    }

    fn config_rsi_lower(&self) -> f64 {
        self.config.base.rsi_lower
    }

    fn config_rsi_upper(&self) -> f64 {
        self.config.base.rsi_upper
    }

    fn config_rsi_count(&self) -> usize {
        3 // 하드코딩된 값 (추후 설정에 추가 가능)
    }

    fn config_bband_period(&self) -> usize {
        self.config.base.bband_period
    }

    fn config_bband_multiplier(&self) -> f64 {
        self.config.base.bband_multiplier
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle.clone());
        self.bband_analyzer.next(candle.clone());
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 새로운 매수 신호: RSI 과매도 + 볼린저밴드 하단 + 이평선 지지
        self.check_buy_signal(self.config_rsi_count())
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // 새로운 매도 신호: RSI 과매수 + 볼린저밴드 상단 + 이평선 저항
        self.check_sell_signal(self.config_rsi_count())
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Copys
    }
}
