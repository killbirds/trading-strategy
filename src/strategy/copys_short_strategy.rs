use super::Strategy;
use super::StrategyType;
use super::copys_common::{CopysStrategyCommon, CopysStrategyConfigBase, CopysStrategyContext};
use crate::analyzer::atr_analyzer::ATRAnalyzer;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::supertrend_analyzer::SuperTrendAnalyzer;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// Copys 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct CopysShortStrategyConfig {
    #[serde(flatten)]
    pub base: CopysStrategyConfigBase,
    /// RSI 조건 판정 횟수
    pub rsi_count: usize,
    /// ATR 계산 기간
    pub atr_period: usize,
    /// ATR 배수 (손절가 계산용)
    pub atr_multiplier: f64,
    /// 슈퍼트렌드 계산 기간
    pub st_period: usize,
    /// 슈퍼트렌드 배수
    pub st_multiplier: f64,
}

impl Default for CopysShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        CopysShortStrategyConfig {
            base: CopysStrategyConfigBase {
                rsi_period: 14,
                rsi_upper: 70.0,
                rsi_lower: 30.0,
                bband_period: 20,
                bband_multiplier: 2.0,
            },
            rsi_count: 3,
            atr_period: 14,
            atr_multiplier: 1.0,
            st_period: 10,
            st_multiplier: 3.0,
        }
    }
}

impl CopysShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        // 기본 설정 유효성 검사
        self.base.validate()?;

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.atr_period == 0 {
            return Err("ATR 기간은 0보다 커야 합니다".to_string());
        }

        if self.atr_multiplier <= 0.0 {
            return Err("ATR 배수는 0보다 커야 합니다".to_string());
        }

        if self.st_period < 2 {
            return Err("슈퍼트렌드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.st_multiplier <= 0.0 {
            return Err("슈퍼트렌드 배수는 0보다 커야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<CopysShortStrategyConfig, String> {
        match CopysStrategyConfigBase::from_json::<CopysShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<CopysShortStrategyConfig, String> {
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

        let rsi_count = match config.get("rsi_count") {
            Some(count_str) => {
                let count = count_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 판정 횟수 파싱 오류".to_string())?;

                if count == 0 {
                    return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
                }

                count
            }
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

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

        // ATR 관련 설정
        let atr_period = match config.get("atr_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "ATR 기간 파싱 오류".to_string())?;

                if period == 0 {
                    return Err("ATR 기간은 0보다 커야 합니다".to_string());
                }

                period
            }
            None => 14, // 기본값
        };

        let atr_multiplier = match config.get("atr_multiplier") {
            Some(multiplier_str) => {
                let multiplier = multiplier_str
                    .parse::<f64>()
                    .map_err(|_| "ATR 배수 파싱 오류".to_string())?;

                if multiplier <= 0.0 {
                    return Err("ATR 배수는 0보다 커야 합니다".to_string());
                }

                multiplier
            }
            None => 1.0, // 기본값
        };

        // 슈퍼트렌드 관련 설정
        let st_period = match config.get("st_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "슈퍼트렌드 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("슈퍼트렌드 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => 10, // 기본값
        };

        let st_multiplier = match config.get("st_multiplier") {
            Some(multiplier_str) => {
                let multiplier = multiplier_str
                    .parse::<f64>()
                    .map_err(|_| "슈퍼트렌드 배수 파싱 오류".to_string())?;

                if multiplier <= 0.0 {
                    return Err("슈퍼트렌드 배수는 0보다 커야 합니다".to_string());
                }

                multiplier
            }
            None => 3.0, // 기본값
        };

        Ok(CopysShortStrategyConfig {
            base: CopysStrategyConfigBase {
                rsi_period,
                rsi_lower,
                rsi_upper,
                bband_period,
                bband_multiplier,
            },
            rsi_count,
            atr_period,
            atr_multiplier,
            st_period,
            st_multiplier,
        })
    }
}

/// Copys 기반 숏 전략
pub struct CopysShortStrategy<C: Candle> {
    /// 전략 설정
    config: CopysShortStrategyConfig,
    /// 전략 컨텍스트
    ctx: CopysStrategyContext<C>,
    /// ATR 분석기
    atr_analyzer: ATRAnalyzer<C>,
    /// 슈퍼트렌드 분석기
    supertrend_analyzer: SuperTrendAnalyzer<C>,
}

impl<C: Candle> Display for CopysShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Copys숏전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({}), ATR: {}({}), ST: {}({})}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.bband_period,
            self.config.base.bband_multiplier,
            self.config.atr_period,
            self.config.atr_multiplier,
            self.config.st_period,
            self.config.st_multiplier,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> CopysShortStrategy<C> {
    /// 새 코피스 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<CopysShortStrategy<C>, String> {
        let config = CopysShortStrategyConfig::from_json(json_config)?;
        Self::new_with_config_internal(storage, config)
    }

    /// 새 코피스 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<CopysShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => CopysShortStrategyConfig::from_hash_map(&cfg)?,
            None => CopysShortStrategyConfig::default(),
        };

        Self::new_with_config_internal(storage, strategy_config)
    }

    // 내부 설정 구현
    fn new_with_config_internal(
        storage: &CandleStore<C>,
        config: CopysShortStrategyConfig,
    ) -> Result<CopysShortStrategy<C>, String> {
        info!("코피스 숏 전략 설정: {:?}", config);

        let ma_type = crate::indicator::ma::MAType::EMA;
        let ma_periods = [10, 20, 60];
        let ctx = CopysStrategyContext::new(config.base.rsi_period, &ma_type, &ma_periods, storage);

        // ATR 분석기 생성
        let atr_periods = [config.atr_period];
        let atr_analyzer = ATRAnalyzer::new(&atr_periods, storage);

        // 슈퍼트렌드 분석기 생성
        let st_settings = [(config.st_period, config.st_multiplier)];
        let supertrend_analyzer = SuperTrendAnalyzer::new(&st_settings, storage);

        Ok(CopysShortStrategy {
            config,
            ctx,
            atr_analyzer,
            supertrend_analyzer,
        })
    }

    /// ATR 분석기 참조 반환
    pub fn atr_analyzer(&self) -> &ATRAnalyzer<C> {
        &self.atr_analyzer
    }

    /// 슈퍼트렌드 분석기 참조 반환
    pub fn supertrend_analyzer(&self) -> &SuperTrendAnalyzer<C> {
        &self.supertrend_analyzer
    }

    /// 슈퍼트렌드 기반 매수 신호 확인 (숏 포지션에서는 청산 신호)
    pub fn check_supertrend_buy_signal(&self) -> bool {
        // 슈퍼트렌드가 상승 추세로 전환되었는지 확인
        self.supertrend_analyzer
            .is_price_crossing_above_supertrend(&self.config.st_period, &self.config.st_multiplier)
    }

    /// 슈퍼트렌드 기반 매도 신호 확인 (숏 포지션에서는 진입 신호)
    pub fn check_supertrend_sell_signal(&self) -> bool {
        // 슈퍼트렌드가 하락 추세로 전환되었는지 확인
        self.supertrend_analyzer
            .is_price_crossing_below_supertrend(&self.config.st_period, &self.config.st_multiplier)
    }

    /// ATR 이용 손절가 계산
    pub fn calculate_stop_loss(&self, candle: &C, position_type: PositionType) -> f64 {
        let atr_value = if !self.atr_analyzer.items.is_empty() {
            self.atr_analyzer.items[0].get_atr(self.config.atr_period)
        } else {
            0.0
        };

        match position_type {
            PositionType::Long => {
                // 롱 포지션의 손절가: 현재가 - (ATR * 배수)
                candle.close_price() - (atr_value * self.config.atr_multiplier)
            }
            PositionType::Short => {
                // 숏 포지션의 손절가: 현재가 + (ATR * 배수)
                candle.close_price() + (atr_value * self.config.atr_multiplier)
            }
        }
    }
}

impl<C: Candle + 'static> CopysStrategyCommon<C> for CopysShortStrategy<C> {
    fn context(&self) -> &CopysStrategyContext<C> {
        &self.ctx
    }

    fn config_rsi_lower(&self) -> f64 {
        self.config.base.rsi_lower
    }

    fn config_rsi_upper(&self) -> f64 {
        self.config.base.rsi_upper
    }

    fn config_rsi_count(&self) -> usize {
        self.config.rsi_count
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle.clone());
        self.atr_analyzer.next(candle.clone());
        self.supertrend_analyzer.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // RSI 신호와, 슈퍼트렌드 하락 신호 결합 (둘 중 하나라도 만족하면 진입)
        self.check_sell_signal(self.config.rsi_count) || self.check_supertrend_sell_signal()
    }

    fn should_exit(&self, _holdings: &TradePosition, candle: &C) -> bool {
        // RSI 신호와 슈퍼트렌드 상승 신호 결합
        let rsi_signal = self.check_buy_signal(self.config.rsi_count);
        let st_signal = self.check_supertrend_buy_signal();

        // 손절가 확인
        let stop_loss_triggered = if let Some(stop_loss) = _holdings.stop_loss {
            match _holdings.position_type {
                PositionType::Long => candle.close_price() <= stop_loss,
                PositionType::Short => candle.close_price() >= stop_loss,
            }
        } else {
            false
        };

        // 세 조건 중 하나라도 만족하면 청산
        rsi_signal || st_signal || stop_loss_triggered
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::CopysShort
    }

    fn set_stop_loss(&self, holdings: &mut TradePosition, candle: &C) {
        // ATR 기반 손절가 설정
        let stop_loss = self.calculate_stop_loss(candle, holdings.position_type);
        holdings.stop_loss = Some(stop_loss);
    }
}
