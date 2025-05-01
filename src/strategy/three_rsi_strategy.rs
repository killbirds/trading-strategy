use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use super::{Strategy, split};
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::adx::{ADX, ADXBuilder};
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::rsi::{RSIs, RSIsBuilder, RSIsBuilderFactory};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 세 개의 RSI를 사용하는 전략 설정
#[derive(Debug, Deserialize)]
pub struct ThreeRSIStrategyConfig {
    pub rsi_periods: Vec<usize>,
    pub ma: MAType,
    pub ma_period: usize,
    pub adx_period: usize,
}

impl Default for ThreeRSIStrategyConfig {
    fn default() -> Self {
        ThreeRSIStrategyConfig {
            rsi_periods: vec![6, 14, 26],
            ma: MAType::EMA,
            ma_period: 50,
            adx_period: 14,
        }
    }
}

impl ThreeRSIStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_periods.is_empty() {
            return Err("RSI 기간은 최소 하나 이상 지정해야 합니다".to_string());
        }

        for period in &self.rsi_periods {
            if *period == 0 {
                return Err("RSI 기간은 0보다 커야 합니다".to_string());
            }
        }

        if self.ma_period == 0 {
            return Err("이동평균 기간은 0보다 커야 합니다".to_string());
        }

        if self.adx_period == 0 {
            return Err("ADX 기간은 0보다 커야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<ThreeRSIStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<ThreeRSIStrategyConfig, String> {
        match serde_json::from_str::<ThreeRSIStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<ThreeRSIStrategyConfig, String> {
        // RSI 기간 설정
        let rsi_periods = match config.get("rsi_periods") {
            Some(periods_str) => match split(periods_str) {
                Ok(periods) => {
                    if periods.is_empty() {
                        return Err("RSI 기간은 최소 하나 이상 지정해야 합니다".to_string());
                    }
                    for period in &periods {
                        if *period == 0 {
                            return Err("RSI 기간은 0보다 커야 합니다".to_string());
                        }
                    }
                    periods
                }
                Err(e) => return Err(format!("RSI 기간 파싱 오류: {}", e)),
            },
            None => return Err("rsi_periods 설정이 필요합니다".to_string()),
        };

        // 이동평균 유형 설정
        let ma = match config.get("ma").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_period = match config.get("ma_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "이동평균 기간 파싱 오류".to_string())?;

                if period == 0 {
                    return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                }

                period
            }
            None => return Err("ma_period 설정이 필요합니다".to_string()),
        };

        // ADX 기간 설정
        let adx_period = match config.get("adx_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "ADX 기간 파싱 오류".to_string())?;

                if period == 0 {
                    return Err("ADX 기간은 0보다 커야 합니다".to_string());
                }

                period
            }
            None => return Err("adx_period 설정이 필요합니다".to_string()),
        };

        let result = ThreeRSIStrategyConfig {
            rsi_periods,
            ma,
            ma_period,
            adx_period,
        };

        result.validate()?;
        Ok(result)
    }
}

struct StrategyData<C: Candle> {
    candle: C,
    rsis: RSIs,
    ma: Box<dyn MA>,
    adx: ADX,
}

impl<C: Candle> StrategyData<C> {
    fn new(candle: C, rsis: RSIs, ma: Box<dyn MA>, adx: ADX) -> StrategyData<C> {
        StrategyData {
            candle,
            rsis,
            ma,
            adx,
        }
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

struct StrategyContext<C: Candle> {
    rsisbuilder: RSIsBuilder<C>,
    mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    adxbuilder: ADXBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first = self.items.first().unwrap();
        write!(
            f,
            "candle: {}, ma: {}, rsis: {}, adx: {}",
            first.candle, first.ma, first.rsis, first.adx
        )
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    fn new(config: &ThreeRSIStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsisbuilder = RSIsBuilderFactory::build(&config.rsi_periods);
        let mabuilder = MABuilderFactory::build(&config.ma, config.ma_period);
        let adxbuilder = ADXBuilder::new(config.adx_period);

        let mut ctx = StrategyContext {
            rsisbuilder,
            mabuilder,
            adxbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let rsis = self.rsisbuilder.next(&candle);
        let ma = self.mabuilder.next(&candle);
        let adx = self.adxbuilder.next(&candle);
        StrategyData::new(candle, rsis, ma, adx)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

pub struct ThreeRSIStrategy<C: Candle> {
    config: ThreeRSIStrategyConfig,
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for ThreeRSIStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rsi_periods = self
            .config
            .rsi_periods
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(
            f,
            "[3RSI전략] 설정: {{RSI기간: [{}], MA: {:?}({}), ADX기간: {}}}, 컨텍스트: {}",
            rsi_periods, self.config.ma, self.config.ma_period, self.config.adx_period, self.ctx
        )
    }
}

impl<C: Candle + 'static> ThreeRSIStrategy<C> {
    /// 새 세개 RSI 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<ThreeRSIStrategy<C>, String>` - 초기화된 세개 RSI 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<ThreeRSIStrategy<C>, String> {
        let config = ThreeRSIStrategyConfig::from_json(json_config)?;
        info!("세개 RSI 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

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

        info!("삼중 RSI 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(ThreeRSIStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

fn is_rsi_regular_arrangement<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_regular_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, 2)
}

fn is_rsi_greater_than_50<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_all_greater_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, 2)
}

fn is_candle_greater_than_ma<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_candle_greater_than(|candle| candle.low_price(), |ctx| ctx.ma.get(), 2)
}

fn is_adx_greater_than_20<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_greater_than_target(|ctx| ctx.adx.adx, 20.0, 2)
}

fn is_break_through_rsi_regular_arrangement<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.rsis.is_regular_arrangement(|rsi| rsi.rsi),
        2,
        3,
    );

    let result = result && is_rsi_greater_than_50(strategy);
    let result = result && is_candle_greater_than_ma(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_break_through_rsi_regular_arrangement");
    }

    result
}

fn is_break_through_rsi_greater_than_50<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.rsis.is_all_greater_than(|rsi| rsi.rsi, 50.0),
        2,
        3,
    );

    let result = result && is_rsi_regular_arrangement(strategy);
    let result = result && is_candle_greater_than_ma(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_break_through_rsi_greater_than_50");
    }

    result
}

fn is_break_through_candle_greater_than_ma<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.is_candle_greater_than(|candle| candle.high_price(), |ctx| ctx.ma.get()),
        2,
        3,
    );

    let result = result && is_rsi_regular_arrangement(strategy);
    let result = result && is_rsi_greater_than_50(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_break_through_candle_greater_than_ma");
    }

    result
}

fn is_break_through_adx_greater_than_20<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy
        .ctx
        .is_break_through_by_satisfying(|data| data.adx.adx > 20.0, 2, 3);

    let result = result && is_rsi_regular_arrangement(strategy);
    let result = result && is_rsi_greater_than_50(strategy);
    let result = result && is_candle_greater_than_ma(strategy);

    if result {
        log::info!("is_break_through_adx_greater_than_20");
    }

    result
}

fn is_short_rsi_reverse_arrangement<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_reverse_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, 2)
}

fn is_short_rsi_less_than_50<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_all_less_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, 2)
}

fn is_short_candle_less_than_ma<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    strategy
        .ctx
        .is_candle_less_than(|candle| candle.high_price(), |ctx| ctx.ma.get(), 2)
}

fn is_short_break_through_rsi_reverse_arrangement<C: Candle>(
    strategy: &ThreeRSIStrategy<C>,
) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.rsis.is_reverse_arrangement(|rsi| rsi.rsi),
        2,
        3,
    );

    let result = result && is_short_rsi_less_than_50(strategy);
    let result = result && is_short_candle_less_than_ma(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_short_break_through_rsi_reverse_arrangement");
    }

    result
}

fn is_short_break_through_rsi_less_than_50<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.rsis.is_all_less_than(|rsi| rsi.rsi, 50.0),
        2,
        3,
    );

    let result = result && is_short_rsi_reverse_arrangement(strategy);
    let result = result && is_short_candle_less_than_ma(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_short_break_through_rsi_less_than_50");
    }

    result
}

fn is_short_break_through_candle_less_than_ma<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy.ctx.is_break_through_by_satisfying(
        |data| data.is_candle_less_than(|candle| candle.low_price(), |ctx| ctx.ma.get()),
        2,
        3,
    );

    let result = result && is_short_rsi_reverse_arrangement(strategy);
    let result = result && is_short_rsi_less_than_50(strategy);
    let result = result && is_adx_greater_than_20(strategy);

    if result {
        log::info!("is_short_break_through_candle_less_than_ma");
    }

    result
}

fn is_short_break_through_adx_greater_than_20<C: Candle>(strategy: &ThreeRSIStrategy<C>) -> bool {
    let result = strategy
        .ctx
        .is_break_through_by_satisfying(|data| data.adx.adx > 20.0, 2, 3);

    let result = result && is_short_rsi_reverse_arrangement(strategy);
    let result = result && is_short_rsi_less_than_50(strategy);
    let result = result && is_short_candle_less_than_ma(strategy);

    if result {
        log::info!("is_short_break_through_adx_greater_than_20");
    }

    result
}

impl<C: Candle> Strategy<C> for ThreeRSIStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        is_break_through_rsi_regular_arrangement(self)
            || is_break_through_rsi_greater_than_50(self)
            || is_break_through_candle_greater_than_ma(self)
            || is_break_through_adx_greater_than_20(self)
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        is_short_break_through_rsi_reverse_arrangement(self)
            || is_short_break_through_rsi_less_than_50(self)
            || is_short_break_through_candle_less_than_ma(self)
            || is_short_break_through_adx_greater_than_20(self)
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::ThreeRSI
    }
}
