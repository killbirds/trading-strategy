use super::Strategy;
use super::StrategyType;
use super::hybrid_common::{HybridAnalyzer, HybridStrategyCommon, HybridStrategyConfigBase};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::model::TradePosition;
use log::{debug, info};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 성능 최적화를 위한 캐시 구조체
#[derive(Debug, Default)]
struct SignalCache {
    // 매수 신호 캐시
    buy_signal_strength: Option<f64>,
    // 매도 신호 캐시
    sell_signal_strength: Option<f64>,
    // 마지막 캔들 인덱스 (캐시 무효화에 사용)
    last_candle_index: usize,
}

/// 하이브리드 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct HybridShortStrategyConfig {
    #[serde(flatten)]
    pub base: HybridStrategyConfigBase,

    /// 진입 신호 임계값 (기본값: 0.7)
    #[serde(default = "default_entry_threshold")]
    pub entry_threshold: f64,

    /// 청산 신호 임계값 (기본값: 0.6)
    #[serde(default = "default_exit_threshold")]
    pub exit_threshold: f64,

    /// 손절 수준 (기본값: -7.0)
    #[serde(default = "default_stop_loss")]
    pub stop_loss: f64,

    /// 이익 실현 수준 (기본값: 10.0)
    #[serde(default = "default_take_profit")]
    pub take_profit: f64,
}

fn default_entry_threshold() -> f64 {
    0.7
}
fn default_exit_threshold() -> f64 {
    0.6
}
fn default_stop_loss() -> f64 {
    -7.0
}
fn default_take_profit() -> f64 {
    10.0
}

impl Default for HybridShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        HybridShortStrategyConfig {
            base: HybridStrategyConfigBase::default(),
            entry_threshold: default_entry_threshold(),
            exit_threshold: default_exit_threshold(),
            stop_loss: default_stop_loss(),
            take_profit: default_take_profit(),
        }
    }
}

impl HybridShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        // 기본 설정 유효성 검사
        self.base.validate()?;

        // 임계값 검사
        if self.entry_threshold <= 0.0 || self.entry_threshold > 1.0 {
            return Err("진입 신호 임계값은 0과 1 사이여야 합니다".to_string());
        }
        if self.exit_threshold <= 0.0 || self.exit_threshold > 1.0 {
            return Err("청산 신호 임계값은 0과 1 사이여야 합니다".to_string());
        }

        // 손절/이익실현 레벨 검사
        if self.stop_loss >= 0.0 {
            return Err("손절 수준은 음수여야 합니다".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("이익 실현 수준은 양수여야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<HybridShortStrategyConfig, String> {
        match HybridStrategyConfigBase::from_json::<HybridShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<HybridShortStrategyConfig, String> {
        let base_config = HybridStrategyConfigBase::from_hash_map(config)?;

        // 추가 설정 추출
        let entry_threshold = config
            .get("entry_threshold")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("진입 임계값 파싱 오류: {}", e))
            })
            .transpose()?
            .unwrap_or_else(default_entry_threshold);

        let exit_threshold = config
            .get("exit_threshold")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("청산 임계값 파싱 오류: {}", e))
            })
            .transpose()?
            .unwrap_or_else(default_exit_threshold);

        let stop_loss = config
            .get("stop_loss")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("손절 수준 파싱 오류: {}", e))
            })
            .transpose()?
            .unwrap_or_else(default_stop_loss);

        let take_profit = config
            .get("take_profit")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| format!("이익 실현 수준 파싱 오류: {}", e))
            })
            .transpose()?
            .unwrap_or_else(default_take_profit);

        let strategy_config = HybridShortStrategyConfig {
            base: base_config,
            entry_threshold,
            exit_threshold,
            stop_loss,
            take_profit,
        };

        strategy_config.validate()?;
        Ok(strategy_config)
    }
}

/// 하이브리드 숏 전략 구현
///
/// MACD, RSI, 이동평균선을 결합하여 시장 하락 상황에 적응적으로 대응하는 전략
pub struct HybridShortStrategy<C: Candle + Clone> {
    config: HybridShortStrategyConfig,
    ctx: HybridAnalyzer<C>,

    // 캐시 및 최적화를 위한 필드
    cache: RefCell<SignalCache>,
    // 마지막 사용된 신호 강도 공유
    last_signal_strength: RefCell<f64>,
}

impl<C: Candle + Clone> Display for HybridShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[하이브리드숏전략] 설정: {{RSI: {}(상:{}/하:{}), MACD: {}/{}/{}, 진입임계:{:.2}, 청산임계:{:.2}, 손절:{:.1}, 이익실현:{:.1}}}, 컨텍스트: {}",
            self.config.base.rsi_period,
            self.config.base.rsi_upper,
            self.config.base.rsi_lower,
            self.config.base.macd_fast_period,
            self.config.base.macd_slow_period,
            self.config.base.macd_signal_period,
            self.config.entry_threshold,
            self.config.exit_threshold,
            self.config.stop_loss,
            self.config.take_profit,
            self.ctx
        )
    }
}

impl<C: Candle + Clone + 'static> HybridShortStrategy<C> {
    /// 새 하이브리드 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<HybridShortStrategy<C>, String> {
        let config = HybridShortStrategyConfig::from_json(json_config)?;
        info!("하이브리드 숏 전략 설정: {:?}", config);
        debug!("캔들 데이터 상태: 항목 수={}", storage.len());
        let ctx = HybridAnalyzer::new(
            &config.base.ma_type,
            config.base.ma_period,
            config.base.macd_fast_period,
            config.base.macd_slow_period,
            config.base.macd_signal_period,
            config.base.rsi_period,
            storage,
        );

        Ok(HybridShortStrategy {
            config,
            ctx,
            cache: RefCell::new(SignalCache::default()),
            last_signal_strength: RefCell::new(0.0),
        })
    }

    /// 새 하이브리드 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<HybridShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => HybridShortStrategyConfig::from_hash_map(&cfg)?,
            None => HybridShortStrategyConfig::default(),
        };

        debug!("캔들 데이터 상태: 항목 수={}", storage.len());
        let ctx = HybridAnalyzer::new(
            &strategy_config.base.ma_type,
            strategy_config.base.ma_period,
            strategy_config.base.macd_fast_period,
            strategy_config.base.macd_slow_period,
            strategy_config.base.macd_signal_period,
            strategy_config.base.rsi_period,
            storage,
        );

        Ok(HybridShortStrategy {
            config: strategy_config,
            ctx,
            cache: RefCell::new(SignalCache::default()),
            last_signal_strength: RefCell::new(0.0),
        })
    }

    /// 캐시를 리셋하고 새로운 데이터에 대한 준비
    fn reset_cache(&self) {
        let mut cache = self.cache.borrow_mut();
        cache.buy_signal_strength = None;
        cache.sell_signal_strength = None;
        if self.ctx.items.last().is_some() {
            cache.last_candle_index = self.ctx.items.len();
        }
    }

    /// 매도(숏 진입) 신호 강도 계산 - 최적화된 버전
    fn calculate_sell_signal_strength_optimized(&self, profit_percentage: f64) -> f64 {
        // 아이템이 충분하지 않으면 신호 없음
        if self.ctx.items.len() < 2 {
            return 0.0;
        }

        // 캐시 확인
        let mut cache = self.cache.borrow_mut();
        if cache.sell_signal_strength.is_some() && cache.last_candle_index == self.ctx.items.len() {
            return cache.sell_signal_strength.unwrap();
        }

        let current = self.ctx.items.last().unwrap();
        let previous = &self.ctx.items[self.ctx.items.len() - 2];
        let config = &self.config.base;

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호 (숏이므로 가격이 MA 아래에 있을 때 매도 신호)
        if current.candle.close_price() < current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호 (숏이므로 반대로 해석)
        if current.macd.histogram < 0.0 && previous.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선을 하향 돌파 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선 아래에 있음 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호 (숏이므로 반대로 해석)
        let rsi = current.rsi.value();
        if rsi > config.rsi_upper && rsi < previous.rsi.value() {
            // RSI가 과매수 상태에서 하락 시작 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi > 50.0 && rsi < config.rsi_upper {
            // RSI가 중간값 위이면서 과매수 상태는 아닌 경우 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 4. 추세 확인 (숏 전략에서는 하락 추세가 유리)
        if self.ctx.items.len() >= 5 {
            let price_5_ago = self.ctx.items[self.ctx.items.len() - 5]
                .candle
                .close_price();
            let current_price = current.candle.close_price();

            if current_price < price_5_ago {
                // 5개 캔들 전보다 가격이 낮으면 하락 추세로 판단
                strength += 0.5;
                count += 0.5;
            }
        }

        // 수익률이 이미 좋은 경우 신호 강화 (숏 전략에서는 음수 수익률이 좋은 것)
        if profit_percentage > 5.0 {
            strength += 0.3;
            count += 0.3;
        }

        // 최종 강도 계산 (정규화)
        let final_strength = if count > 0.0 {
            strength / (count * 1.5) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        };

        // 캐시 업데이트
        cache.sell_signal_strength = Some(final_strength);
        cache.last_candle_index = self.ctx.items.len();

        // 결과 반환
        final_strength
    }

    /// 매수(숏 청산) 신호 강도 계산 - 최적화된 버전
    fn calculate_buy_signal_strength_optimized(&self) -> f64 {
        // 아이템이 충분하지 않으면 신호 없음
        if self.ctx.items.len() < 2 {
            return 0.0;
        }

        // 캐시 확인
        let mut cache = self.cache.borrow_mut();
        if cache.buy_signal_strength.is_some() && cache.last_candle_index == self.ctx.items.len() {
            return cache.buy_signal_strength.unwrap();
        }

        let current = self.ctx.items.last().unwrap();
        let previous = &self.ctx.items[self.ctx.items.len() - 2];
        let config = &self.config.base;

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호 (숏 청산이므로 가격이 MA 위로 갈 때)
        if current.candle.close_price() > current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호 (숏 청산)
        if current.macd.histogram > 0.0 && previous.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선을 상향 돌파 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선 위에 있음 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호 (숏 청산)
        let rsi = current.rsi.value();
        if rsi < config.rsi_lower && rsi > previous.rsi.value() {
            // RSI가 과매도 상태에서 반등 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi < 50.0 && rsi > config.rsi_lower {
            // RSI가 중간값 아래이면서 과매도 상태는 아닌 경우 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 4. 추세 확인 (숏 청산이므로 상승 추세가 신호가 됨)
        if self.ctx.items.len() >= 5 {
            let price_5_ago = self.ctx.items[self.ctx.items.len() - 5]
                .candle
                .close_price();
            let current_price = current.candle.close_price();

            if current_price > price_5_ago {
                // 5개 캔들 전보다 가격이 높으면 상승 추세로 판단
                strength += 0.5;
                count += 0.5;
            }
        }

        // 최종 강도 계산 (정규화)
        let final_strength = if count > 0.0 {
            strength / (count * 1.5) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        };

        // 캐시 업데이트
        cache.buy_signal_strength = Some(final_strength);
        cache.last_candle_index = self.ctx.items.len();

        // 결과 반환
        final_strength
    }
}

impl<C: Candle + Clone + 'static> HybridStrategyCommon<C> for HybridShortStrategy<C> {
    fn context(&self) -> &HybridAnalyzer<C> {
        &self.ctx
    }

    fn config_base(&self) -> &HybridStrategyConfigBase {
        &self.config.base
    }

    // 기존 메서드는 하위 호환성을 위해 유지하고 최적화된 버전으로 리디렉션
    fn calculate_buy_signal_strength(&self) -> f64 {
        self.calculate_buy_signal_strength_optimized()
    }

    fn calculate_sell_signal_strength(&self, profit_percentage: f64) -> f64 {
        self.calculate_sell_signal_strength_optimized(profit_percentage)
    }
}

impl<C: Candle + Clone + 'static> Strategy<C> for HybridShortStrategy<C> {
    fn next(&mut self, candle: C) {
        // 새 캔들이 추가되면 캐시를 리셋
        self.reset_cache();

        // 컨텍스트에 데이터 추가
        self.ctx.next_data(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 여러 지표를 종합한 매도(숏 진입) 신호를 기반으로 결정
        let signal_strength = self.calculate_sell_signal_strength_optimized(0.0);

        // 신호 강도 저장
        *self.last_signal_strength.borrow_mut() = signal_strength;

        // 신호 강도가 임계값 이상인 경우에만 숏 진입 (설정에서 임계값 가져옴)
        signal_strength >= self.config.entry_threshold
    }

    fn should_exit(&self, holdings: &TradePosition, candle: &C) -> bool {
        if self.ctx.items.is_empty() {
            return false;
        }

        // 현재 가격
        let current_price = candle.close_price();

        // 현재 수익률 계산 (숏 포지션이므로 방향이 반대)
        let profit_percentage = (1.0 - current_price / holdings.price) * 100.0;

        // 손절/이익실현 경계 확인
        if profit_percentage <= self.config.stop_loss
            || profit_percentage >= self.config.take_profit
        {
            // 손절 또는 이익실현 경계에 도달하면 즉시 청산
            return true;
        }

        // 여러 지표를 종합한 매수 신호를 기반으로 결정
        let signal_strength = self.calculate_buy_signal_strength_optimized();

        // 신호 강도 저장
        *self.last_signal_strength.borrow_mut() = signal_strength;

        // 신호 강도가 임계값 이상인 경우에만 숏 청산 (설정에서 임계값 가져옴)
        signal_strength >= self.config.exit_threshold
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::HybridShort
    }
}
