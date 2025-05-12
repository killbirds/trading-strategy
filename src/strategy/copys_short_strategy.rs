use super::Strategy;
use super::StrategyType;
use super::copys_common::{CopysStrategyCommon, CopysStrategyConfigBase, CopysStrategyContext};
use super::split;
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
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
    /// 볼린저밴드 조건 판정 횟수
    pub bband_count: usize,
    /// 이동평균 계산 방식
    pub ma: MAType,
    /// 이동평균 기간 목록
    pub ma_periods: Vec<usize>,
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
            bband_count: 2,
            ma: MAType::EMA,
            ma_periods: vec![10, 20, 60],
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

        if self.bband_count == 0 {
            return Err("볼린저밴드 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.ma_periods.is_empty() {
            return Err("이동평균 기간이 지정되지 않았습니다".to_string());
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
        let rsi_count = match config.get("rsi_count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 연속 판정 횟수 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 연속 판정 횟수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(value_str) => value_str
                .parse::<f64>()
                .map_err(|_| "RSI 하한 임계값 파싱 오류".to_string())?,
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(value_str) => value_str
                .parse::<f64>()
                .map_err(|_| "RSI 상한 임계값 파싱 오류".to_string())?,
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "숏 전략에서 RSI 하한값({})은 상한값({})보다 작거나 같아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        let rsi_period = match config.get("rsi_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 계산 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 계산 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        // 볼린저밴드 관련 설정
        let bband_count = match config.get("bband_count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저밴드 연속 판정 횟수 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("볼린저밴드 연속 판정 횟수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_count 설정이 필요합니다".to_string()),
        };

        let bband_period = match config.get("bband_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저밴드 계산 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("볼린저밴드 계산 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_period 설정이 필요합니다".to_string()),
        };

        let bband_multiplier = match config.get("bband_multiplier") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "볼린저밴드 표준편차 승수 파싱 오류".to_string())?;

                if value <= 0.0 {
                    return Err("볼린저밴드 표준편차 승수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_multiplier 설정이 필요합니다".to_string()),
        };

        // 이동평균 관련 설정
        let ma = match config.get("ma").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        let ma_periods = match config.get("ma_periods") {
            Some(periods_str) => match split(periods_str) {
                Ok(periods) => {
                    if periods.is_empty() {
                        return Err("이동평균 기간이 비어 있습니다".to_string());
                    }

                    for &period in &periods {
                        if period == 0 {
                            return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                        }
                    }

                    periods
                }
                Err(e) => return Err(format!("이동평균 기간 파싱 오류: {}", e)),
            },
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
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
            bband_count,
            ma,
            ma_periods,
        })
    }
}

/// Copys 기반 숏 전략
#[derive(Debug)]
pub struct CopysShortStrategy<C: Candle> {
    /// 전략 설정
    config: CopysShortStrategyConfig,
    /// 전략 컨텍스트
    ctx: CopysStrategyContext<C>,
}

impl<C: Candle> Display for CopysShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let periods = self
            .config
            .ma_periods
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(
            f,
            "[Copys숏전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({}), MA타입: {:?}({})}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.bband_period,
            self.config.base.bband_multiplier,
            self.config.ma,
            periods,
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
        info!("코피스 숏 전략 설정: {:?}", config);

        let mut ctx = CopysStrategyContext::new(
            config.base.rsi_period,
            &config.ma,
            &config.ma_periods,
            storage,
        );
        ctx.init_from_storage(storage);

        Ok(CopysShortStrategy { config, ctx })
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

        info!("코피스 숏 전략 설정: {:?}", strategy_config);

        let ctx = CopysStrategyContext::new(
            strategy_config.base.rsi_period,
            &strategy_config.ma,
            &strategy_config.ma_periods,
            storage,
        );

        Ok(CopysShortStrategy {
            config: strategy_config,
            ctx,
        })
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
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 이동평균선이 정상 배열이면 거래하지 않음 (역배열을 원함)
        if self.ctx.is_ma_regular_arrangement(1) {
            false
        } else {
            // RSI가 상한선보다 높으면 숏 진입 (롱 전략과 반대로 높은 값에서 진입)
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.value > self.config.base.rsi_upper,
                1,
                self.config.rsi_count,
            )
        }
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 이동평균선이 역배열이면 거래하지 않음 (정상 배열을 원함)
        if self.ctx.is_ma_reverse_arrangement(1) {
            false
        } else {
            // RSI가 하한선보다 낮으면 숏 청산
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.value < self.config.base.rsi_lower,
                1,
                self.config.rsi_count,
            )
        }
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::CopysShort
    }
}
