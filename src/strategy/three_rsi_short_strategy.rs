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

/// 세 개의 RSI를 사용하는 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct ThreeRSIShortStrategyConfig {
    /// 세 가지 RSI 기간
    pub rsi_periods: Vec<usize>,
    /// 이동평균 유형
    pub ma: MAType,
    /// 이동평균 계산 기간
    pub ma_period: usize,
    /// ADX 계산 기간
    pub adx_period: usize,
}

impl Default for ThreeRSIShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        ThreeRSIShortStrategyConfig {
            rsi_periods: vec![6, 14, 26],
            ma: MAType::EMA,
            ma_period: 50,
            adx_period: 14,
        }
    }
}

impl ThreeRSIShortStrategyConfig {
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
    /// * `Result<ThreeRSIShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<ThreeRSIShortStrategyConfig, String> {
        match serde_json::from_str::<ThreeRSIShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ThreeRSIShortStrategyConfig, String> {
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

        let result = ThreeRSIShortStrategyConfig {
            rsi_periods,
            ma,
            ma_period,
            adx_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// ThreeRSI 숏 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    /// 현재 캔들 데이터
    candle: C,
    /// 세 가지 RSI 데이터
    rsis: RSIs,
    /// 이동평균선 데이터
    ma: Box<dyn MA>,
    /// ADX 지표 데이터
    adx: ADX,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
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

/// ThreeRSI 숏 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    /// RSIs 빌더 (세 가지 RSI 계산)
    rsisbuilder: RSIsBuilder<C>,
    /// 이동평균 빌더
    mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    /// ADX 빌더
    adxbuilder: ADXBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, 이동평균: {}, RSIs: {}, ADX: {}",
                first.candle, first.ma, first.rsis, first.adx
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    fn new(config: &ThreeRSIShortStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsisbuilder = RSIsBuilderFactory::build::<C>(&config.rsi_periods);
        let mabuilder = MABuilderFactory::build::<C>(&config.ma, config.ma_period);
        let adxbuilder = ADXBuilder::<C>::new(config.adx_period);

        let mut ctx = StrategyContext {
            rsisbuilder,
            mabuilder,
            adxbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// RSI 값이 모두 50 미만인지 확인 (약세)
    fn is_rsi_all_less_than_50(&self, n: usize) -> bool {
        self.is_all_less_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, n)
    }

    /// RSI가 역순 배열인지 확인 (단기가 더 작고 장기가 더 큼)
    fn is_rsi_reverse_arrangement(&self, n: usize) -> bool {
        self.is_reverse_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, n)
    }

    /// 캔들이 이동평균선보다 낮은지 확인 (약세)
    fn is_candle_low_below_ma(&self, n: usize) -> bool {
        self.is_candle_less_than(|candle| candle.low_price(), |ctx| ctx.ma.get(), n)
    }

    /// ADX가 20 이상인지 확인 (추세 강도)
    fn is_adx_greater_than_20(&self, n: usize) -> bool {
        self.is_greater_than_target(|ctx| ctx.adx.adx, 20.0, n)
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

/// ThreeRSI 기반 숏 트레이딩 전략
///
/// 세 가지 다른 기간의 RSI, 이동평균선, ADX를 함께 분석하여 숏 포지션 진입/청산 신호를 생성합니다.
#[derive(Debug)]
pub struct ThreeRSIShortStrategy<C: Candle> {
    /// 전략 설정
    config: ThreeRSIShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for ThreeRSIShortStrategy<C> {
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
            "[3RSI숏전략] 설정: {{RSI기간: [{}], MA타입: {:?}({}), ADX기간: {}}}, 컨텍스트: {}",
            rsi_periods, self.config.ma, self.config.ma_period, self.config.adx_period, self.ctx
        )
    }
}

impl<C: Candle + 'static> ThreeRSIShortStrategy<C> {
    /// 새 세개 RSI 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<ThreeRSIShortStrategy<C>, String>` - 초기화된 세개 RSI 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<ThreeRSIShortStrategy<C>, String> {
        let config = ThreeRSIShortStrategyConfig::from_json(json_config)?;
        info!("세개 RSI 숏 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

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
        let ctx = StrategyContext::new(&strategy_config, storage);

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
            |data| data.rsis.is_all_less_than(|rsi| rsi.rsi, 50.0),
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
        self.ctx
            .is_all_greater_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, 2)
            && self
                .ctx
                .is_regular_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, 2)
            && self
                .ctx
                .is_candle_greater_than(|candle| candle.high_price(), |ctx| ctx.ma.get(), 2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 신호: RSI가 최근에 모두 50 이상으로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_rsi_above_50(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.rsis.is_all_greater_than(|rsi| rsi.rsi, 50.0),
            2,
            3,
        ) && self
            .ctx
            .is_regular_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, 2)
            && self
                .ctx
                .is_candle_greater_than(|candle| candle.high_price(), |ctx| ctx.ma.get(), 2)
            && self.ctx.is_adx_greater_than_20(2)
    }

    /// 청산 신호: 캔들이 최근에 MA 위로 돌파했고 다른 조건도 충족
    fn should_exit_by_break_through_above_ma(&self) -> bool {
        self.ctx.is_break_through_by_satisfying(
            |data| data.is_candle_greater_than(|candle| candle.close_price(), |ctx| ctx.ma.get()),
            2,
            3,
        ) && self
            .ctx
            .is_all_greater_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, 2)
            && self
                .ctx
                .is_regular_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, 2)
            && self.ctx.is_adx_greater_than_20(2)
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

    fn get_position(&self) -> PositionType {
        PositionType::Short
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::ThreeRSIShort
    }
}
