use super::Strategy;
use super::StrategyType;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::model::TradePosition;
use crate::{ConfigError, ConfigResult, ConfigValidation};
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

// 공통 모듈 가져오기
use super::macd_common::{MACDAnalyzer, MACDStrategyCommon, MACDStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;

/// MACD 숏 전략 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct MACDShortStrategyConfig {
    /// 빠른 EMA 기간
    pub fast_period: usize,
    /// 느린 EMA 기간
    pub slow_period: usize,
    /// 시그널 라인 기간
    pub signal_period: usize,
    /// 히스토그램 임계값 (0보다 작을 때 숏 진입)
    pub histogram_threshold: f64,
    /// 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
    pub confirm_period: usize,
}

impl Default for MACDShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        MACDShortStrategyConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            histogram_threshold: 0.0,
            confirm_period: 3,
        }
    }
}

impl ConfigValidation for MACDShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        let base = MACDStrategyConfigBase {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            histogram_threshold: self.histogram_threshold,
            confirm_period: self.confirm_period,
        };

        // 숏 전략에서는 히스토그램 임계값이 0보다 작아야 함을 추가 검증
        if self.histogram_threshold > 0.0 {
            return Err(ConfigError::ValidationError(format!(
                "숏 전략의 히스토그램 임계값({})은 0보다 작아야 합니다",
                self.histogram_threshold
            )));
        }

        base.validate()
    }
}

impl MACDShortStrategyConfig {
    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<MACDShortStrategyConfig, String> {
        let config = MACDStrategyConfigBase::from_json::<MACDShortStrategyConfig>(json, false)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<MACDShortStrategyConfig, String> {
        let base_config = MACDStrategyConfigBase::from_hash_map(config, false)?;

        let result = MACDShortStrategyConfig {
            fast_period: base_config.fast_period,
            slow_period: base_config.slow_period,
            signal_period: base_config.signal_period,
            histogram_threshold: base_config.histogram_threshold,
            confirm_period: base_config.confirm_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// MACD 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct MACDShortStrategy<C: Candle> {
    /// 전략 설정
    config: MACDShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: MACDAnalyzer<C>,
}

impl<C: Candle> Display for MACDShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[MACD숏전략] 설정: {{빠른기간: {}, 느린기간: {}, 시그널기간: {}, 임계값: {}}}, 컨텍스트: {}",
            self.config.fast_period,
            self.config.slow_period,
            self.config.signal_period,
            self.config.histogram_threshold,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> MACDShortStrategy<C> {
    /// 새 MACD 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<MACDShortStrategy<C>, String> {
        let config = MACDShortStrategyConfig::from_json(json_config)?;
        info!("MACD 숏 전략 설정: {:?}", config);

        let ctx = MACDAnalyzer::new(
            config.fast_period,
            config.slow_period,
            config.signal_period,
            storage,
        );

        Ok(MACDShortStrategy { config, ctx })
    }

    /// 새 MACD 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MACDShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => MACDShortStrategyConfig::from_hash_map(&cfg)?,
            None => MACDShortStrategyConfig::default(),
        };

        info!("MACD 숏 전략 설정: {:?}", strategy_config);

        let ctx = MACDAnalyzer::new(
            strategy_config.fast_period,
            strategy_config.slow_period,
            strategy_config.signal_period,
            storage,
        );

        Ok(MACDShortStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> MACDStrategyCommon<C> for MACDShortStrategy<C> {
    fn context(&self) -> &MACDAnalyzer<C> {
        &self.ctx
    }

    fn config_confirm_period(&self) -> usize {
        self.config.confirm_period
    }

    fn config_histogram_threshold(&self) -> f64 {
        self.config.histogram_threshold
    }
}

impl<C: Candle + 'static> Strategy<C> for MACDShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // MACD가 시그널 라인을 하향 돌파하고 히스토그램이 임계값보다 작으면 숏 진입 신호
        self.ctx
            .is_macd_crossed_below_signal(1, self.config.confirm_period)
            && self
                .ctx
                .is_histogram_below_threshold(self.config.histogram_threshold, 1)
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // MACD가 시그널 라인을 상향 돌파하면 숏 청산 신호
        self.ctx
            .is_macd_crossed_above_signal(1, self.config.confirm_period)
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::MACDShort
    }
}
