use super::Strategy;
use super::StrategyType;
use super::bband_common::{BBandAnalyzer, BBandStrategyCommon, BBandStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::config_loader::{ConfigResult, ConfigValidation};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
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
}

impl Default for BBandShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        BBandShortStrategyConfig {
            count: 3,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl ConfigValidation for BBandShortStrategyConfig {
    fn validate(&self) -> ConfigResult<()> {
        let base = BBandStrategyConfigBase {
            count: self.count,
            period: self.period,
            multiplier: self.multiplier,
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
        config.validate()?;
        Ok(config)
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BBandShortStrategyConfig, String> {
        let base_config = BBandStrategyConfigBase::from_hash_map(config)?;

        let result = BBandShortStrategyConfig {
            count: base_config.count,
            period: base_config.period,
            multiplier: base_config.multiplier,
        };

        result.validate()?;
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
            "[볼린저밴드숏전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BBandStrategyCommon<C> for BBandShortStrategy<C> {
    fn context(&self) -> &BBandAnalyzer<C> {
        &self.ctx
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
        info!("볼린저밴드 숏 전략 설정: {:?}", config);
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

        info!("볼린저밴드 숏 전략 설정: {:?}", strategy_config);
        let ctx = BBandAnalyzer::new(strategy_config.period, strategy_config.multiplier, storage);

        Ok(BBandShortStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// 밴드 폭이 충분히 넓은지 확인
    fn is_band_width_sufficient(&self) -> bool {
        self.ctx.is_greater_than_target(
            |data| (data.bband.upper() - data.bband.lower()) / data.bband.average(),
            0.02,
            1,
        )
    }

    /// 설정 파일에서 전략 인스턴스 생성
    pub fn from_config_file(
        storage: &CandleStore<C>,
        config_path: &Path,
    ) -> Result<BBandShortStrategy<C>, String> {
        let config =
            match BBandStrategyConfigBase::from_file::<BBandShortStrategyConfig>(config_path) {
                Ok(cfg) => cfg,
                Err(e) => return Err(format!("설정 파일 로드 오류: {}", e)),
            };
        info!("볼린저밴드 숏 전략 설정 로드됨: {:?}", config);

        let ctx = BBandAnalyzer::new(config.period, config.multiplier, storage);
        Ok(BBandShortStrategy { config, ctx })
    }
}

impl<C: Candle + 'static> Strategy<C> for BBandShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 가격이 상단 밴드를 상향 돌파했을 때 숏 진입 신호
        let is_buy = self.ctx.is_break_through_by_greater_than_candle(
            |data| data.bband.upper(),
            |candle| candle.high_price(),
            1,
            self.config.count,
        );

        // 밴드 폭이 충분히 넓은지 확인
        is_buy && self.is_band_width_sufficient()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 가격이 하단 밴드를 하향 돌파했을 때 숏 청산 신호
        self.ctx.is_break_through_by_less_than_candle(
            |data| data.bband.lower(),
            |candle| candle.low_price(),
            1,
            self.config.count,
        )
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::BBandShort
    }
}
