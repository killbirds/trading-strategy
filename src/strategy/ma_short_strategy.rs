use super::Strategy;
use super::StrategyType;
use crate::analyzer::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::strategy::ma_common::{MAAnalyzer, MAStrategyCommon, MAStrategyConfigBase};
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 이동평균(MA) 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct MAShortStrategyConfig {
    /// 이동평균 계산 방식 (SMA, EMA 등)
    pub ma: crate::indicator::ma::MAType,
    /// 이동평균 기간 목록 (짧은 것부터 긴 것 순)
    pub ma_periods: Vec<usize>,
    /// 데드 크로스/골든 크로스 판정 조건: 이전 기간
    pub cross_previous_periods: usize,
}

impl Default for MAShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        MAShortStrategyConfig {
            ma: crate::indicator::ma::MAType::EMA,
            ma_periods: vec![5, 20, 60],
            cross_previous_periods: 15,
        }
    }
}

impl MAShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        let base = MAStrategyConfigBase {
            ma: self.ma,
            ma_periods: self.ma_periods.clone(),
            cross_previous_periods: self.cross_previous_periods,
        };
        base.validate()
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<MAShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<MAShortStrategyConfig, String> {
        let config = MAStrategyConfigBase::from_json::<MAShortStrategyConfig>(json)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<MAShortStrategyConfig, String> {
        let base_config = MAStrategyConfigBase::from_hash_map(config)?;

        let result = MAShortStrategyConfig {
            ma: base_config.ma,
            ma_periods: base_config.ma_periods,
            cross_previous_periods: base_config.cross_previous_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// 이동평균 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct MAShortStrategy<C: Candle> {
    /// 전략 설정
    config: MAShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: MAAnalyzer<C>,
}

impl<C: Candle> Display for MAShortStrategy<C> {
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
            "[MA숏전략] 설정: {{타입: {:?}, 기간: [{}], 크로스확인: {}}}, 컨텍스트: {}",
            self.config.ma, periods, self.config.cross_previous_periods, self.ctx
        )
    }
}

impl<C: Candle + 'static> MAStrategyCommon<C> for MAShortStrategy<C> {
    fn context(&self) -> &MAAnalyzer<C> {
        &self.ctx
    }

    fn config_cross_previous_periods(&self) -> usize {
        self.config.cross_previous_periods
    }
}

impl<C: Candle + 'static> MAShortStrategy<C> {
    /// 새 MA 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<MAShortStrategy<C>, String>` - 초기화된 MA 숏 전략 인스턴스 또는 오류
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<MAShortStrategy<C>, String> {
        let config = MAShortStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 MA 숏 전략 인스턴스 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정
    ///
    /// # Returns
    /// * `Result<MAShortStrategy<C>, String>` - 초기화된 MA 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        config: MAShortStrategyConfig,
    ) -> Result<MAShortStrategy<C>, String> {
        info!("MA 숏 전략 설정: {config:?}");
        let ctx = MAAnalyzer::new(&config.ma, &config.ma_periods, storage);

        Ok(MAShortStrategy { config, ctx })
    }

    /// 새 MA 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MAShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => MAShortStrategyConfig::from_hash_map(&cfg)?,
            None => MAShortStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> Strategy<C> for MAShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 데드 크로스 발생시 숏 진입 신호
        self.ctx
            .is_ma_reverse_arrangement_dead_cross(1, self.config.cross_previous_periods, 0)
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // 단기 이동평균이 장기 이동평균보다 높아질 때(골든 크로스) 숏 청산
        self.check_cross_condition(|data| {
            let short_ma = data.mas.get_by_key_index(0).get();
            let long_ma = data.mas.get_by_key_index(data.mas.len() - 1).get();
            short_ma > long_ma
        })
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::MAShort
    }
}
