use super::Strategy;
use super::StrategyType;
use super::config_utils;
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

/// Copys 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct CopysShortStrategyConfig {
    #[serde(flatten)]
    pub base: CopysStrategyConfigBase,
    /// RSI 조건 판정 횟수
    pub rsi_count: usize,
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
                ma_distance_threshold: 0.02,
            },
            rsi_count: 3,
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
        // 공통 유틸리티를 사용하여 RSI 설정 파싱
        let (rsi_period, rsi_lower, rsi_upper) = config_utils::parse_rsi_config(config)?;

        // RSI 판정 횟수 설정
        let rsi_count = config_utils::parse_usize(config, "rsi_count", Some(1), true)?
            .ok_or("rsi_count 설정이 필요합니다")?;

        // 볼린저 밴드 관련 설정
        let bband_period = config_utils::parse_usize(config, "bband_period", Some(2), true)?
            .ok_or("bband_period 설정이 필요합니다")?;

        let bband_multiplier =
            config_utils::parse_f64(config, "bband_multiplier", Some((0.0, f64::MAX)), true)?
                .ok_or("bband_multiplier 설정이 필요합니다")?;

        if bband_multiplier <= 0.0 {
            return Err("볼린저 밴드 승수는 0보다 커야 합니다".to_string());
        }

        let ma_distance_threshold =
            config_utils::parse_f64(config, "ma_distance_threshold", Some((0.0, 1.0)), false)?
                .unwrap_or(0.02);

        Ok(CopysShortStrategyConfig {
            base: CopysStrategyConfigBase {
                rsi_period,
                rsi_lower,
                rsi_upper,
                bband_period,
                bband_multiplier,
                ma_distance_threshold,
            },
            rsi_count,
        })
    }
}

/// Copys 기반 숏 전략
pub struct CopysShortStrategy<C: Candle> {
    /// 전략 설정
    config: CopysShortStrategyConfig,
    /// 전략 컨텍스트
    ctx: CopysStrategyContext<C>,
    /// 볼린저밴드 분석기
    bband_analyzer: BBandAnalyzer<C>,
}

impl<C: Candle> Display for CopysShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Copys숏전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({})}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.bband_period,
            self.config.base.bband_multiplier,
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
        info!("코피스 숏 전략 설정: {config:?}");

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

        Ok(CopysShortStrategy {
            config,
            ctx,
            bband_analyzer,
        })
    }

    /// 볼린저밴드 분석기 참조 반환
    pub fn bband_analyzer(&self) -> &BBandAnalyzer<C> {
        &self.bband_analyzer
    }
}

impl<C: Candle + 'static> CopysStrategyCommon<C> for CopysShortStrategy<C> {
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
        self.config.rsi_count
    }

    fn config_bband_period(&self) -> usize {
        self.config.base.bband_period
    }

    fn config_bband_multiplier(&self) -> f64 {
        self.config.base.bband_multiplier
    }

    fn config_ma_distance_threshold(&self) -> f64 {
        self.config.base.ma_distance_threshold
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysShortStrategy<C> {
    fn next(&mut self, candle: C) {
        // 한 번만 클론하여 두 analyzer에 전달
        let candle_clone = candle.clone();
        self.ctx.next(candle);
        self.bband_analyzer.next(candle_clone);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 숏 포지션 진입: RSI 과매수 + 볼린저밴드 상단 + 이평선 저항
        self.check_sell_signal(self.config.rsi_count)
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // 숏 포지션 청산: RSI 과매도 + 볼린저밴드 하단 + 이평선 지지
        self.check_buy_signal(self.config.rsi_count)
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::CopysShort
    }
}
