use super::Strategy;
use super::StrategyType;
use super::bband_common::{BBandAnalyzer, BBandStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::{ConfigResult, ConfigValidation};
use log::{debug, error, info};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 볼린저 밴드 전략 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct BBandStrategyConfig {
    /// 확인 캔들 수
    pub count: usize,
    /// 볼린저 밴드 계산 기간
    pub period: usize,
    /// 볼린저 밴드 승수 (표준편차 배수)
    pub multiplier: f64,
}

impl ConfigValidation for BBandStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        let base = BBandStrategyConfigBase {
            count: self.count,
            period: self.period,
            multiplier: self.multiplier,
        };
        base.validate()
    }
}

impl Default for BBandStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        BBandStrategyConfig {
            count: 2,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl BBandStrategyConfig {
    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<BBandStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<BBandStrategyConfig, String> {
        let config = BBandStrategyConfigBase::from_json::<BBandStrategyConfig>(json)?;
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BBandStrategyConfig, String> {
        let base_config = BBandStrategyConfigBase::from_hash_map(config)?;

        let result = BBandStrategyConfig {
            count: base_config.count,
            period: base_config.period,
            multiplier: base_config.multiplier,
        };

        result.validate()?;
        Ok(result)
    }
}

/// 볼린저 밴드 기반 트레이딩 전략
#[derive(Debug)]
pub struct BBandStrategy<C: Candle> {
    /// 전략 설정
    config: BBandStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: BBandAnalyzer<C>,
}

impl<C: Candle> Display for BBandStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[볼린저밴드전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BBandStrategy<C> {
    /// 새 볼린저밴드 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<BBandStrategy<C>, String>` - 초기화된 볼린저밴드 전략 인스턴스 또는 오류
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BBandStrategy<C>, String> {
        debug!("볼린저 밴드 전략 초기화 시작 (JSON 설정 사용)");
        let config = match BBandStrategyConfig::from_json(json_config) {
            Ok(cfg) => {
                debug!("JSON 설정 파싱 성공: {:?}", cfg);
                cfg
            }
            Err(e) => {
                error!("볼린저 밴드 전략 JSON 설정 파싱 실패: {}", e);
                return Err(format!("볼린저 밴드 전략 설정 오류: {}", e));
            }
        };

        Self::new(storage, config)
    }

    /// 새 볼린저밴드 전략 인스턴스 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정
    ///
    /// # Returns
    /// * `Result<BBandStrategy<C>, String>` - 초기화된 볼린저밴드 전략 인스턴스
    pub fn new(
        storage: &CandleStore<C>,
        config: BBandStrategyConfig,
    ) -> Result<BBandStrategy<C>, String> {
        info!("볼린저밴드 전략 설정: {:?}", config);
        debug!(
            "볼린저 밴드 컨텍스트 초기화 시작 (기간: {}, 승수: {})",
            config.period, config.multiplier
        );

        let ctx = BBandAnalyzer::new(config.period, config.multiplier, storage);
        debug!("볼린저 밴드 컨텍스트 초기화 완료");

        let strategy = BBandStrategy { config, ctx };
        info!("볼린저 밴드 전략 초기화 완료");
        Ok(strategy)
    }

    /// 새 볼린저 밴드 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BBandStrategy<C>, String> {
        debug!("볼린저 밴드 전략 초기화 시작 (HashMap 설정 사용)");

        let strategy_config = match config {
            Some(cfg) => {
                debug!("HashMap 설정 파싱 시작");
                match BBandStrategyConfig::from_hash_map(&cfg) {
                    Ok(parsed_config) => {
                        debug!("HashMap 설정 파싱 성공: {:?}", parsed_config);
                        parsed_config
                    }
                    Err(e) => {
                        error!("볼린저 밴드 전략 HashMap 설정 파싱 실패: {}", e);
                        return Err(format!("볼린저 밴드 전략 설정 오류: {}", e));
                    }
                }
            }
            None => {
                debug!("설정이 제공되지 않음, 기본 설정 사용");
                BBandStrategyConfig::default()
            }
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + 'static> Strategy<C> for BBandStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        self.ctx.is_below_lower_band(1)
    }

    fn should_exit(&self, _candle: &C) -> bool {
        self.ctx.is_above_middle_band(1)
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::BBand
    }
}
