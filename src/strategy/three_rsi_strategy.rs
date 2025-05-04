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

/// 세 개의 RSI를 사용하는 전략 설정
#[derive(Debug, Deserialize, Default)]
pub struct ThreeRSIStrategyConfig {
    #[serde(flatten)]
    pub base: ThreeRSIStrategyConfigBase,
}

impl ThreeRSIStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()
    }

    /// JSON 문자열에서 설정 로드
    fn from_json(json: &str) -> Result<ThreeRSIStrategyConfig, String> {
        match ThreeRSIStrategyConfigBase::from_json::<ThreeRSIStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<ThreeRSIStrategyConfig, String> {
        let base_config = ThreeRSIStrategyConfigBase::from_hash_map(config)?;
        Ok(ThreeRSIStrategyConfig { base: base_config })
    }
}

/// 세개 RSI 기반 트레이딩 전략
#[derive(Debug)]
pub struct ThreeRSIStrategy<C: Candle> {
    config: ThreeRSIStrategyConfig,
    ctx: ThreeRSIAnalyzer<C>,
}

impl<C: Candle> Display for ThreeRSIStrategy<C> {
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
            "[3RSI전략] 설정: {{RSI기간: [{}], MA: {:?}({}), ADX기간: {}}}, 컨텍스트: {}",
            rsi_periods,
            self.config.base.ma,
            self.config.base.ma_period,
            self.config.base.adx_period,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> ThreeRSIStrategy<C> {
    /// 새 세개 RSI 전략 인스턴스 생성 (JSON 설정 파일 사용)
    pub fn from_json(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<ThreeRSIStrategy<C>, String> {
        let config = ThreeRSIStrategyConfig::from_json(json_config)?;
        Self::new(storage, config)
    }

    /// 새 세개 RSI 전략 인스턴스 생성
    pub fn new(
        storage: &CandleStore<C>,
        config: ThreeRSIStrategyConfig,
    ) -> Result<ThreeRSIStrategy<C>, String> {
        info!("세개 RSI 전략 설정: {:?}", config);
        let ctx = ThreeRSIAnalyzer::new(
            &config.base.rsi_periods,
            &config.base.ma,
            config.base.ma_period,
            config.base.adx_period,
            storage,
        );

        Ok(ThreeRSIStrategy { config, ctx })
    }

    /// 새 삼중 RSI 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<ThreeRSIStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => ThreeRSIStrategyConfig::from_hash_map(&cfg)?,
            None => ThreeRSIStrategyConfig::default(),
        };

        Self::new(storage, strategy_config)
    }

    /// 진입 조건: RSI가 정규 배열이고 50 이상이며 캔들이 MA 위에 있고 ADX > 20
    fn should_enter_by_rsi_regular_arrangement(&self) -> bool {
        self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_rsi_all_greater_than_50(2)
            && self.ctx.is_candle_high_above_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 진입 조건: RSI가 최근에 모두 50 이상으로 돌파했고 다른 조건도 충족
    fn should_enter_by_break_through_rsi_above_50(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.rsis.is_all_greater_than(|rsi| rsi.rsi, 50.0),
            2,
            3,
        ) && self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_candle_high_above_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 진입 조건: 캔들이 최근에 MA 위로 돌파했고 다른 조건도 충족
    fn should_enter_by_break_through_above_ma(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.is_candle_greater_than(|candle| candle.close_price(), |ctx| ctx.ma.get()),
            2,
            3,
        ) && self.ctx.is_rsi_regular_arrangement(2)
            && self.ctx.is_rsi_all_greater_than_50(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 조건: RSI가 역배열이고 50 미만이며 캔들이 MA 아래에 있고 ADX > 20
    fn should_exit_by_rsi_reverse_arrangement(&self) -> bool {
        self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_rsi_all_less_than_50(2)
            && self.ctx.is_candle_low_below_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 조건: RSI가 최근에 모두 50 미만으로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_rsi_below_50(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.rsis.is_all_less_than(|rsi| rsi.rsi, 50.0),
            2,
            3,
        ) && self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_candle_low_below_ma(2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 조건: 캔들이 최근에 MA 아래로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_below_ma(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.is_candle_less_than(|candle| candle.close_price(), |ctx| ctx.ma.get()),
            2,
            3,
        ) && self.ctx.is_rsi_reverse_arrangement(2)
            && self.ctx.is_rsi_all_less_than_50(2)
            && self.ctx.is_adx_greater_than_20(2)
    }
}

impl<C: Candle + 'static> ThreeRSIStrategyCommon<C> for ThreeRSIStrategy<C> {
    fn context(&self) -> &ThreeRSIAnalyzer<C> {
        &self.ctx
    }
}

impl<C: Candle + 'static> Strategy<C> for ThreeRSIStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        self.should_enter_by_rsi_regular_arrangement()
            || self.should_enter_by_break_through_rsi_above_50()
            || self.should_enter_by_break_through_above_ma()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        self.should_exit_by_rsi_reverse_arrangement()
            || self.should_exit_by_break_through_rsi_below_50()
            || self.should_exit_by_break_through_below_ma()
    }

    fn position(&self) -> PositionType {
        PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::ThreeRSI
    }
}
