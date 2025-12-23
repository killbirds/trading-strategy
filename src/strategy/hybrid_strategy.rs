use super::Strategy;
use super::StrategyType;
use super::hybrid_common::{
    HybridAnalyzer, HybridStrategyCommon, HybridStrategyConfigBase, SignalCache,
};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use log::info;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 하이브리드 전략 설정
#[derive(Debug, Deserialize)]
pub struct HybridStrategyConfig {
    #[serde(flatten)]
    pub base: HybridStrategyConfigBase,
    /// 진입 신호 임계값 (기본값: 0.0)
    #[serde(default = "default_entry_threshold")]
    pub entry_threshold: f64,
    /// 청산 신호 임계값 (기본값: 0.2)
    #[serde(default = "default_exit_threshold")]
    pub exit_threshold: f64,
}

fn default_entry_threshold() -> f64 {
    0.0
}

fn default_exit_threshold() -> f64 {
    0.2
}

impl Default for HybridStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        HybridStrategyConfig {
            base: HybridStrategyConfigBase::default(),
            entry_threshold: default_entry_threshold(),
            exit_threshold: default_exit_threshold(),
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

        let entry_threshold = config
            .get("entry_threshold")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("진입 임계값 파싱 오류: {e}"))
            })
            .transpose()?
            .unwrap_or_else(default_entry_threshold);

        let exit_threshold = config
            .get("exit_threshold")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("청산 임계값 파싱 오류: {e}"))
            })
            .transpose()?
            .unwrap_or_else(default_exit_threshold);

        Ok(HybridStrategyConfig {
            base: base_config,
            entry_threshold,
            exit_threshold,
        })
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
    /// 성능 최적화를 위한 캐시
    cache: RefCell<SignalCache>,
}

impl<C: Candle + Clone> Display for HybridStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[하이브리드전략] 설정: {{RSI: {}(상:{}/하:{}), MACD: {}/{}/{}, 진입임계:{:.2}, 청산임계:{:.2}}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.macd_fast_period,
            self.config.base.macd_slow_period,
            self.config.base.macd_signal_period,
            self.config.entry_threshold,
            self.config.exit_threshold,
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
        info!("하이브리드 전략 설정: {config:?}");
        let ctx = HybridAnalyzer::new(
            &config.base.ma_type,
            config.base.ma_period,
            config.base.macd_fast_period,
            config.base.macd_slow_period,
            config.base.macd_signal_period,
            config.base.rsi_period,
            storage,
        );

        Ok(HybridStrategy {
            config,
            ctx,
            cache: RefCell::new(SignalCache::default()),
        })
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

impl<C: Candle + Clone + 'static> HybridStrategy<C> {
    /// 캐시를 리셋하고 새로운 데이터에 대한 준비
    fn reset_cache(&self) {
        let mut cache = self.cache.borrow_mut();
        cache.reset(self.ctx.items.len());
    }

    /// 매수 신호 강도 계산 - 최적화된 버전
    fn calculate_buy_signal_strength_cached(&self) -> f64 {
        if self.ctx.items.len() < 2 {
            return 0.0;
        }

        let items_len = self.ctx.items.len();
        let cache = self.cache.borrow_mut();

        if let Some(strength) = cache.get_buy_signal_strength(items_len) {
            return strength;
        }

        drop(cache);
        let strength = self.calculate_buy_signal_strength();

        let mut cache = self.cache.borrow_mut();
        cache.set_buy_signal_strength(strength, items_len);
        strength
    }

    /// 매도 신호 강도 계산 - 최적화된 버전
    fn calculate_sell_signal_strength_cached(&self, profit_percentage: f64) -> f64 {
        if self.ctx.items.is_empty() {
            return 0.0;
        }

        let items_len = self.ctx.items.len();
        let cache = self.cache.borrow_mut();

        if let Some(strength) = cache.get_sell_signal_strength(items_len) {
            return strength;
        }

        drop(cache);
        let strength = self.calculate_sell_signal_strength(profit_percentage);

        let mut cache = self.cache.borrow_mut();
        cache.set_sell_signal_strength(strength, items_len);
        strength
    }
}

impl<C: Candle + Clone + 'static> Strategy<C> for HybridStrategy<C> {
    fn next(&mut self, candle: C) {
        self.reset_cache();
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        let signal_strength = self.calculate_buy_signal_strength_cached();
        signal_strength >= self.config.entry_threshold
    }

    fn should_exit(&self, _candle: &C) -> bool {
        if self.ctx.items.is_empty() {
            return false;
        }

        let signal_strength = self.calculate_sell_signal_strength_cached(0.0);
        signal_strength >= self.config.exit_threshold
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Hybrid
    }
}
