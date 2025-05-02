use super::Strategy;
use super::StrategyType;
use super::ma_common::{MAStrategyCommon, MAStrategyConfigBase, MAStrategyContext};
use crate::candle_store::CandleStore;
use crate::config_loader::{ConfigError, ConfigResult, ConfigValidation};
use crate::model::PositionType;
use crate::model::TradePosition;
use crate::strategy::context::StrategyContextOps;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use trading_chart::Candle;

/// 이동평균(MA) 전략 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct MAStrategyConfig {
    /// 이동평균 계산 방식 (SMA, EMA 등)
    pub ma: crate::indicator::ma::MAType,
    /// 이동평균 기간 목록 (짧은 것부터 긴 것 순)
    pub ma_periods: Vec<usize>,
    /// 골든 크로스/데드 크로스 판정 조건: 이전 기간
    pub cross_previous_periods: usize,
}

impl Default for MAStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        MAStrategyConfig {
            ma: crate::indicator::ma::MAType::EMA,
            ma_periods: vec![5, 20, 60],
            cross_previous_periods: 15,
        }
    }
}

impl ConfigValidation for MAStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        // MA 전략 설정 유효성 검사 로직 구현
        if self.ma_periods.is_empty() {
            return Err(ConfigError::ValidationError(
                "이동평균 기간이 지정되지 않았습니다".to_string(),
            ));
        }

        // 기간이 오름차순으로 정렬되어 있는지 확인
        for i in 1..self.ma_periods.len() {
            if self.ma_periods[i] <= self.ma_periods[i - 1] {
                return Err(ConfigError::ValidationError(format!(
                    "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                    self.ma_periods
                )));
            }
        }

        if self.cross_previous_periods == 0 {
            return Err(ConfigError::ValidationError(
                "크로스 판정 기간은 0보다 커야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl MAStrategyConfig {
    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<MAStrategyConfig, String> {
        let config = MAStrategyConfigBase::from_json::<MAStrategyConfig>(json)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<MAStrategyConfig, String> {
        let base_config = MAStrategyConfigBase::from_hash_map(config)?;

        let result = MAStrategyConfig {
            ma: base_config.ma,
            ma_periods: base_config.ma_periods,
            cross_previous_periods: base_config.cross_previous_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// 이동평균 기반 트레이딩 전략
#[derive(Debug)]
pub struct MAStrategy<C: Candle> {
    /// 전략 설정
    config: MAStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: MAStrategyContext<C>,
}

impl<C: Candle> Display for MAStrategy<C> {
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
            "[MA전략] 설정: {{타입: {:?}, 기간: [{}], 크로스확인: {}}}, 컨텍스트: {}",
            self.config.ma, periods, self.config.cross_previous_periods, self.ctx
        )
    }
}

impl<C: Candle + 'static> MAStrategyCommon<C> for MAStrategy<C> {
    fn context(&self) -> &MAStrategyContext<C> {
        &self.ctx
    }

    fn config_cross_previous_periods(&self) -> usize {
        self.config.cross_previous_periods
    }
}

impl<C: Candle + 'static> MAStrategy<C> {
    /// 새 MA 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<MAStrategy<C>, String>` - 초기화된 MA 전략 인스턴스 또는 오류
    pub fn from_json(storage: &CandleStore<C>, json_config: &str) -> Result<MAStrategy<C>, String> {
        let config = MAStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 MA 전략 인스턴스 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정
    ///
    /// # Returns
    /// * `Result<MAStrategy<C>, String>` - 초기화된 MA 전략 인스턴스
    pub fn new(
        storage: &CandleStore<C>,
        config: MAStrategyConfig,
    ) -> Result<MAStrategy<C>, String> {
        info!("MA 전략 설정: {:?}", config);
        let ctx = MAStrategyContext::new(&config.ma, &config.ma_periods, storage);

        Ok(MAStrategy { config, ctx })
    }

    /// 새 MA 전략 생성 (설정을 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MAStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => MAStrategyConfig::from_hash_map(&cfg)?,
            None => MAStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }

    /// 설정 파일에서 전략 인스턴스 생성
    pub fn from_config_file(
        storage: &CandleStore<C>,
        config_path: &Path,
    ) -> Result<MAStrategy<C>, String> {
        // 설정 파일 로드
        let config = match crate::config_loader::ConfigLoader::load_from_file::<MAStrategyConfig>(
            config_path,
            crate::config_loader::ConfigFormat::Auto,
        ) {
            Ok(cfg) => cfg,
            Err(e) => return Err(format!("설정 파일 로드 오류: {}", e)),
        };

        info!("MA 전략 설정 로드됨: {:?}", config);

        Self::new(storage, config)
    }
}

impl<C: Candle + 'static> Strategy<C> for MAStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 골든 크로스 발생시 롱 진입 신호
        self.ctx
            .is_ma_regular_arrangement_golden_cross(1, self.config.cross_previous_periods)
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 단기 이동평균이 장기 이동평균보다 낮아질 때(데드 크로스) 롱 청산
        self.check_cross_condition(|data| {
            let short_ma = data.mas.get_from_index(0).get();
            let long_ma = data.mas.get_from_index(2).get();
            short_ma < long_ma
        })
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::MA
    }
}
