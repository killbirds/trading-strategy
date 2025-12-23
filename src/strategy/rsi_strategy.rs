use super::Strategy;
use super::StrategyType;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::{ConfigResult, ConfigValidation};
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

// 공통 모듈 가져오기
use super::rsi_common::{RSIAnalyzer, RSIStrategyCommon, RSIStrategyConfigBase};

use crate::analyzer::base::AnalyzerOps;

/// RSI 전략 설정
///
/// RSI(상대강도지수) 기반 트레이딩 전략에 필요한 모든 설정 파라미터를 포함합니다.
#[derive(Debug, Deserialize)]
pub struct RSIStrategyConfig {
    /// RSI 판단에 필요한 연속 데이터 수
    pub rsi_count: usize,
    /// RSI 하단 기준값 (매수 신호용)
    pub rsi_lower: f64,
    /// RSI 상단 기준값 (매도 신호용)
    pub rsi_upper: f64,
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// 이동평균 유형 (EMA, SMA 등)
    pub ma: crate::indicator::ma::MAType,
    /// 이동평균 기간 목록 (여러 이동평균선 사용)
    pub ma_periods: Vec<usize>,
}

impl Default for RSIStrategyConfig {
    fn default() -> Self {
        RSIStrategyConfig {
            rsi_count: 3,
            rsi_lower: 30.0,
            rsi_upper: 70.0,
            rsi_period: 14,
            ma: crate::indicator::ma::MAType::EMA,
            ma_periods: vec![5, 20, 60],
        }
    }
}

impl ConfigValidation for RSIStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
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
}

impl RSIStrategyConfig {
    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<RSIStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<RSIStrategyConfig, String> {
        let config = RSIStrategyConfigBase::from_json::<RSIStrategyConfig>(json)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<RSIStrategyConfig, String> {
        let base_config = RSIStrategyConfigBase::from_hash_map(config)?;

        let result = RSIStrategyConfig {
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

/// RSI 기반 트레이딩 전략
///
/// RSI(상대강도지수)와 이동평균을 사용한 트레이딩 전략을 구현합니다.
#[derive(Debug)]
pub struct RSIStrategy<C: Candle> {
    /// 전략 설정
    config: RSIStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: RSIAnalyzer<C>,
}

impl<C: Candle> Display for RSIStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[RSI전략] 설정: {{RSI기간: {}, 하한: {}, 상한: {}}}, 컨텍스트: {}",
            self.config.rsi_period, self.config.rsi_lower, self.config.rsi_upper, self.ctx
        )
    }
}

impl<C: Candle + 'static> RSIStrategy<C> {
    /// 새 RSI 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<RSIStrategy<C>, String>` - 초기화된 RSI 전략 인스턴스 또는 오류
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<RSIStrategy<C>, String> {
        let config = RSIStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 RSI 전략 인스턴스 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정
    ///
    /// # Returns
    /// * `Result<RSIStrategy<C>, String>` - 초기화된 RSI 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        config: RSIStrategyConfig,
    ) -> Result<RSIStrategy<C>, String> {
        info!("RSI 전략 설정: {config:?}");

        let ctx = RSIAnalyzer::new(config.rsi_period, &config.ma, &config.ma_periods, storage);

        Ok(RSIStrategy { config, ctx })
    }

    /// 새 RSI 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 (HashMap 형태)
    ///
    /// # Returns
    /// * `Result<RSIStrategy<C>, String>` - 초기화된 RSI 전략 인스턴스 또는 오류
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<RSIStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => RSIStrategyConfig::from_hash_map(&cfg)?,
            None => RSIStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> RSIStrategyCommon<C> for RSIStrategy<C> {
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

impl<C: Candle + 'static> Strategy<C> for RSIStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // RSI가 과매도 구간에서 진입
        self.is_rsi_oversold()
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // RSI가 과매수 구간에서 청산
        self.is_rsi_overbought()
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::RSI
    }
}
