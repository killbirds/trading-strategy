use super::Strategy;
use super::StrategyType;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

// 공통 모듈 가져오기
use super::rsi_common::{RSIAnalyzer, RSIStrategyCommon, RSIStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;

/// RSI 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct RSIShortStrategyConfig {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상단 경계 (과매수 기준)
    pub rsi_upper: f64,
    /// RSI 하단 경계 (과매도 기준)
    pub rsi_lower: f64,
    /// RSI 신호 확인 기간
    pub rsi_count: usize,
    /// 이동평균 계산 방식
    pub ma: crate::indicator::ma::MAType,
    /// 이동평균 기간 목록
    pub ma_periods: Vec<usize>,
}

impl Default for RSIShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        RSIShortStrategyConfig {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            rsi_count: 3,
            ma: crate::indicator::ma::MAType::EMA,
            ma_periods: vec![5, 20, 60],
        }
    }
}

impl RSIShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        let base = RSIStrategyConfigBase {
            rsi_count: self.rsi_count,
            rsi_lower: self.rsi_lower,
            rsi_upper: self.rsi_upper,
            rsi_period: self.rsi_period,
            ma: self.ma,
            ma_periods: self.ma_periods.clone(),
        };

        base.validate()
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<RSIShortStrategyConfig, String> {
        let config = RSIStrategyConfigBase::from_json::<RSIShortStrategyConfig>(json)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<RSIShortStrategyConfig, String> {
        let base_config = RSIStrategyConfigBase::from_hash_map(config)?;

        let result = RSIShortStrategyConfig {
            rsi_count: base_config.rsi_count,
            rsi_lower: base_config.rsi_lower,
            rsi_upper: base_config.rsi_upper,
            rsi_period: base_config.rsi_period,
            ma: base_config.ma,
            ma_periods: base_config.ma_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// RSI 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct RSIShortStrategy<C: Candle> {
    /// 전략 설정
    config: RSIShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: RSIAnalyzer<C>,
}

impl<C: Candle> Display for RSIShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[RSI숏전략] 설정: {{RSI기간: {}, 상한: {}, 하한: {}}}, 컨텍스트: {}",
            self.config.rsi_period, self.config.rsi_upper, self.config.rsi_lower, self.ctx
        )
    }
}

impl<C: Candle + 'static> RSIShortStrategy<C> {
    /// 새 RSI 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<RSIShortStrategy<C>, String> {
        let config = RSIShortStrategyConfig::from_json(json_config)?;
        info!("RSI 숏 전략 설정: {config:?}");

        let ctx = RSIAnalyzer::new(config.rsi_period, &config.ma, &config.ma_periods, storage);

        Ok(RSIShortStrategy { config, ctx })
    }

    /// 새 RSI 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<RSIShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => RSIShortStrategyConfig::from_hash_map(&cfg)?,
            None => RSIShortStrategyConfig::default(),
        };

        info!("RSI 숏 전략 설정: {strategy_config:?}");

        let ctx = RSIAnalyzer::new(
            strategy_config.rsi_period,
            &strategy_config.ma,
            &strategy_config.ma_periods,
            storage,
        );

        Ok(RSIShortStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> RSIStrategyCommon<C> for RSIShortStrategy<C> {
    fn context(&self) -> &RSIAnalyzer<C> {
        &self.ctx
    }

    fn config_rsi_lower(&self) -> f64 {
        self.config.rsi_lower
    }

    fn config_rsi_upper(&self) -> f64 {
        self.config.rsi_upper
    }

    fn config_rsi_count(&self) -> usize {
        self.config.rsi_count
    }
}

impl<C: Candle + 'static> Strategy<C> for RSIShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 이동평균이 정규 배열이면 숏 진입 금지 (상승 추세)
        if self.ctx.is_ma_regular_arrangement(1) {
            return false;
        }

        // RSI가 과매수 구간을 돌파했을 때 숏 진입 신호
        self.is_rsi_overbought()
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // 이동평균이 역배열이면 숏 청산 금지 (하락 추세)
        if self.ctx.is_ma_reverse_arrangement(1) {
            return false;
        }

        // RSI가 과매도 구간을 돌파했을 때 숏 청산 신호
        self.is_rsi_oversold()
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::RSIShort
    }
}
