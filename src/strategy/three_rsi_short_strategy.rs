use super::Strategy;
use super::StrategyType;
use super::three_rsi_common::{
    ThreeRSIAnalyzer, ThreeRSIStrategyCommon, ThreeRSIStrategyConfigBase,
};
use crate::analyzer::base::AnalyzerOps;
use crate::candle_store::CandleStore;
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 세 개의 RSI를 사용하는 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct ThreeRSIShortStrategyConfig {
    #[serde(flatten)]
    pub base: ThreeRSIStrategyConfigBase,
}

impl Default for ThreeRSIShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        ThreeRSIShortStrategyConfig {
            base: ThreeRSIStrategyConfigBase::default(),
        }
    }
}

impl ThreeRSIShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<ThreeRSIShortStrategyConfig, String> {
        match ThreeRSIStrategyConfigBase::from_json::<ThreeRSIShortStrategyConfig>(json) {
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
    ) -> Result<ThreeRSIShortStrategyConfig, String> {
        let base_config = ThreeRSIStrategyConfigBase::from_hash_map(config)?;
        Ok(ThreeRSIShortStrategyConfig { base: base_config })
    }
}

/// 세개 RSI 기반 숏 전략
#[derive(Debug)]
pub struct ThreeRSIShortStrategy<C: Candle> {
    /// 전략 설정
    config: ThreeRSIShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: ThreeRSIAnalyzer<C>,
}

impl<C: Candle> Display for ThreeRSIShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rsi_periods = self
            .config
            .base
            .rsi_periods
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(
            f,
            "[3RSI숏전략] 설정: {{RSI기간: [{}], MA타입: {:?}({}), ADX기간: {}}}, 컨텍스트: {}",
            rsi_periods,
            self.config.base.ma,
            self.config.base.ma_period,
            self.config.base.adx_period,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> ThreeRSIShortStrategy<C> {
    /// 새 세개 RSI 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<ThreeRSIShortStrategy<C>, String> {
        let config = ThreeRSIShortStrategyConfig::from_json(json_config)?;
        info!("세개 RSI 숏 전략 설정: {:?}", config);
        let ctx = ThreeRSIAnalyzer::new(
            &config.base.rsi_periods,
            &config.base.ma,
            config.base.ma_period,
            config.base.adx_period,
            storage,
        );

        Ok(ThreeRSIShortStrategy { config, ctx })
    }

    /// 새 ThreeRSI 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<ThreeRSIShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => ThreeRSIShortStrategyConfig::from_hash_map(&cfg)?,
            None => ThreeRSIShortStrategyConfig::default(),
        };

        info!("삼중 RSI 숏 전략 설정: {:?}", strategy_config);
        let ctx = ThreeRSIAnalyzer::new(
            &strategy_config.base.rsi_periods,
            &strategy_config.base.ma,
            strategy_config.base.ma_period,
            strategy_config.base.adx_period,
            storage,
        );

        Ok(ThreeRSIShortStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// 엔트리 신호: RSI가 모두 50 미만이고 역순 배열이며 캔들이 MA 아래에 있고 ADX > 20
    fn should_enter_by_rsi_below_50_with_reverse_arrangement(&self) -> bool {
        self.ctx.is_rsi_all_less_than_50(2)
            && self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_candle_low_below_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 엔트리 신호: RSI가 최근에 모두 50 미만으로 돌파했고 다른 조건도 충족
    fn should_enter_by_break_through_rsi_below_50(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.rsis.is_all(|rsi| rsi.value < 50.0),
            2,
            3,
        ) && self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_candle_low_below_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 엔트리 신호: 캔들이 최근에 MA 아래로 돌파했고 다른 조건도 충족
    fn should_enter_by_break_through_below_ma(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.is_candle_less_than(|candle| candle.close_price(), |ctx| ctx.ma.get()),
            2,
            3,
        ) && self.ctx.is_rsi_all_less_than_50(2)
            && self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 신호: RSI가 모두 50 이상이고 정순 배열이며 캔들이 MA 위에 있고 ADX > 20
    fn should_exit_by_rsi_above_50_with_regular_arrangement(&self) -> bool {
        self.ctx.is_rsi_all_greater_than_50(2)
            && self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_candle_high_above_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 신호: RSI가 최근에 모두 50 이상으로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_rsi_above_50(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.rsis.is_all(|rsi| rsi.value > 50.0),
            2,
            3,
        ) && self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_candle_high_above_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 신호: 캔들이 최근에 MA 위로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_above_ma(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.is_candle_greater_than(|candle| candle.close_price(), |ctx| ctx.ma.get()),
            2,
            3,
        ) && self.ctx.is_rsi_all_greater_than_50(2)
            && self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_adx_greater_than_20(2)
    }
}

impl<C: Candle + 'static> ThreeRSIStrategyCommon<C> for ThreeRSIShortStrategy<C> {
    fn context(&self) -> &ThreeRSIAnalyzer<C> {
        &self.ctx
    }
}

impl<C: Candle + 'static> Strategy<C> for ThreeRSIShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        self.should_enter_by_rsi_below_50_with_reverse_arrangement()
            || self.should_enter_by_break_through_rsi_below_50()
            || self.should_enter_by_break_through_below_ma()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        self.should_exit_by_rsi_above_50_with_regular_arrangement()
            || self.should_exit_by_break_through_rsi_above_50()
            || self.should_exit_by_break_through_above_ma()
    }

    fn position(&self) -> PositionType {
        PositionType::Short
    }

    fn name(&self) -> StrategyType {
        StrategyType::ThreeRSIShort
    }
}
