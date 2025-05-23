use super::Strategy;
use super::StrategyType;
use super::hybrid_common::{HybridAnalyzer, HybridStrategyCommon, HybridStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 하이브리드 전략 설정
#[derive(Debug, Deserialize)]
pub struct HybridStrategyConfig {
    #[serde(flatten)]
    pub base: HybridStrategyConfigBase,
}

impl Default for HybridStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        HybridStrategyConfig {
            base: HybridStrategyConfigBase::default(),
        }
    }
}

impl HybridStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<HybridStrategyConfig, String> {
        match HybridStrategyConfigBase::from_json::<HybridStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<HybridStrategyConfig, String> {
        let base_config = HybridStrategyConfigBase::from_hash_map(config)?;
        Ok(HybridStrategyConfig { base: base_config })
    }
}

/// 하이브리드 전략 구현
///
/// 여러 지표를 결합해 시장 상승 상황에 적응적으로 대응하는 전략
#[derive(Debug)]
pub struct HybridStrategy<C: Candle + Clone> {
    /// 전략 설정
    config: HybridStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: HybridAnalyzer<C>,
}

impl<C: Candle + Clone> Display for HybridStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[하이브리드전략] 설정: {{RSI: {}(상:{}/하:{}), MACD: {}/{}/{}}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.macd_fast_period,
            self.config.base.macd_slow_period,
            self.config.base.macd_signal_period,
            self.ctx
        )
    }
}

impl<C: Candle + Clone + 'static> HybridStrategy<C> {
    /// 새 하이브리드 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<HybridStrategy<C>, String> {
        let config = HybridStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 하이브리드 전략 인스턴스 생성
    pub fn new(
        storage: &CandleStore<C>,
        config: HybridStrategyConfig,
    ) -> Result<HybridStrategy<C>, String> {
        info!("하이브리드 전략 설정: {:?}", config);
        let ctx = HybridAnalyzer::new(
            &config.base.ma_type,
            config.base.ma_period,
            config.base.macd_fast_period,
            config.base.macd_slow_period,
            config.base.macd_signal_period,
            config.base.rsi_period,
            storage,
        );

        Ok(HybridStrategy { config, ctx })
    }

    /// 새 하이브리드 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<HybridStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => HybridStrategyConfig::from_hash_map(&cfg)?,
            None => HybridStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }
}

impl<C: Candle + Clone + 'static> HybridStrategyCommon<C> for HybridStrategy<C> {
    fn context(&self) -> &HybridAnalyzer<C> {
        &self.ctx
    }

    fn config_base(&self) -> &HybridStrategyConfigBase {
        &self.config.base
    }
}

impl<C: Candle + Clone + 'static> Strategy<C> for HybridStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next_data(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 여러 지표를 종합한 매수 신호를 기반으로 결정
        let signal_strength = self.calculate_buy_signal_strength();

        // 신호 강도가 0.3 이상인 경우에만 매수 (임계값을 낮춤)
        signal_strength >= 0.3
    }

    fn should_exit(&self, _candle: &C) -> bool {
        if self.ctx.items.is_empty() {
            return false;
        }

        // 여러 지표를 종합한 매도 신호를 기반으로 결정
        let signal_strength = self.calculate_sell_signal_strength(0.0);

        // 신호 강도가 0.2 이상인 경우에만 매도 (임계값을 더 낮춤)
        signal_strength >= 0.2
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Hybrid
    }
}
