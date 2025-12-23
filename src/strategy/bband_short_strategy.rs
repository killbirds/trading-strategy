use super::Strategy;
use super::StrategyType;
use super::bband_common::{BBandAnalyzer, BBandStrategyConfigBase};
use super::config_utils;
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::{ConfigResult, ConfigValidation};
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 볼린저밴드 숏 전략 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct BBandShortStrategyConfig {
    /// 판정 기간
    pub count: usize,
    /// 볼린저밴드 계산 기간
    pub period: usize,
    /// 표준편차 배수
    pub multiplier: f64,
    /// 밴드 폭 감소 확인 기간 (스퀴즈 감지용, 기본값: 5)
    #[serde(default = "default_narrowing_period")]
    pub narrowing_period: usize,
    /// 좁은 상태 유지 기간 (스퀴즈 감지용, 기본값: 5)
    #[serde(default = "default_squeeze_period")]
    pub squeeze_period: usize,
    /// 스퀴즈 조건을 위한 최소 밴드 폭 (비율, 기본값: 0.02)
    #[serde(default = "default_squeeze_threshold")]
    pub squeeze_threshold: f64,
}

fn default_narrowing_period() -> usize {
    5
}

fn default_squeeze_period() -> usize {
    5
}

fn default_squeeze_threshold() -> f64 {
    0.02
}

impl Default for BBandShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        BBandShortStrategyConfig {
            count: 3,
            period: 20,
            multiplier: 2.0,
            narrowing_period: 5,
            squeeze_period: 5,
            squeeze_threshold: 0.02,
        }
    }
}

impl ConfigValidation for BBandShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        let base = BBandStrategyConfigBase {
            count: self.count,
            period: self.period,
            multiplier: self.multiplier,
            narrowing_period: self.narrowing_period,
            squeeze_period: self.squeeze_period,
            squeeze_threshold: self.squeeze_threshold,
        };
        base.validate()
    }
}

impl BBandShortStrategyConfig {
    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<BBandShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<BBandShortStrategyConfig, String> {
        let config = BBandStrategyConfigBase::from_json::<BBandShortStrategyConfig>(json)?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BBandShortStrategyConfig, String> {
        let base_config = BBandStrategyConfigBase::from_hash_map(config)?;

        // 공통 유틸리티를 사용하여 선택적 설정 파싱
        let narrowing_period =
            config_utils::parse_usize(config, "narrowing_period", Some(1), false)?.unwrap_or(5);

        let squeeze_period =
            config_utils::parse_usize(config, "squeeze_period", Some(1), false)?.unwrap_or(5);

        let squeeze_threshold =
            config_utils::parse_f64(config, "squeeze_threshold", Some((0.0, f64::MAX)), false)?
                .unwrap_or(0.02);

        let result = BBandShortStrategyConfig {
            count: base_config.count,
            period: base_config.period,
            multiplier: base_config.multiplier,
            narrowing_period,
            squeeze_period,
            squeeze_threshold,
        };

        result.validate().map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// 볼린저밴드 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct BBandShortStrategy<C: Candle> {
    config: BBandShortStrategyConfig,
    ctx: BBandAnalyzer<C>,
}

impl<C: Candle> Display for BBandShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[볼린저밴드숏전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}, 스퀴즈: {}/{}/{:.3}}}, 컨텍스트: {}",
            self.config.period,
            self.config.multiplier,
            self.config.count,
            self.config.narrowing_period,
            self.config.squeeze_period,
            self.config.squeeze_threshold,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> BBandShortStrategy<C> {
    /// 새 볼린저밴드 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<BBandShortStrategy<C>, String>` - 초기화된 볼린저밴드 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BBandShortStrategy<C>, String> {
        let config = BBandShortStrategyConfig::from_json(json_config)?;
        info!("볼린저밴드 숏 전략 설정: {config:?}");
        let ctx = BBandAnalyzer::new(config.period, config.multiplier, storage);

        Ok(BBandShortStrategy { config, ctx })
    }

    /// 새 볼린저밴드 숏 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 (HashMap 형태)
    ///
    /// # Returns
    /// * `Result<BBandShortStrategy<C>, String>` - 초기화된 볼린저밴드 숏 전략 인스턴스 또는 오류
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BBandShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => BBandShortStrategyConfig::from_hash_map(&cfg)?,
            None => BBandShortStrategyConfig::default(),
        };

        info!("볼린저밴드 숏 전략 설정: {strategy_config:?}");
        let ctx = BBandAnalyzer::new(strategy_config.period, strategy_config.multiplier, storage);

        Ok(BBandShortStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> Strategy<C> for BBandShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 숏 진입: 스퀴즈 후 상단 밴드 돌파 시 숏 진입 신호
        // 스퀴즈 패턴 확인 (밴드 폭이 좁아지다가 좁은 상태 유지)
        let squeeze_pattern = self.ctx.is_narrowing_then_squeeze_pattern(
            self.config.narrowing_period,
            self.config.squeeze_period,
            self.config.squeeze_threshold,
        );

        // 상단 밴드 돌파 확인
        let breaks_upper = self
            .ctx
            .is_break_through_upper_band_from_below(self.config.count, 0);

        // 밴드 폭이 충분히 넓은지 확인 (스퀴즈 후 확대 시작)
        let band_width_sufficient = self.ctx.is_band_width_sufficient(0);

        squeeze_pattern && breaks_upper && band_width_sufficient
    }

    fn should_exit(&self, _candle: &C) -> bool {
        // 숏 청산: 하단 밴드 하향 돌파 또는 중간 밴드 아래로 하락
        let breaks_lower = self
            .ctx
            .is_break_through_lower_band_from_below(self.config.count, 0);

        let below_middle = self.ctx.is_below_middle_band(1, 0);

        breaks_lower || below_middle
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::BBandShort
    }
}
