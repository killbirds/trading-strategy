use super::Strategy;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use super::split;
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::adx::{ADX, ADXBuilder};
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::rsi::{RSIs, RSIsBuilder, RSIsBuilderFactory};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 세 개의 RSI를 사용하는 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct ThreeRSIStrategyConfigBase {
    /// 세 가지 RSI 기간
    pub rsi_periods: Vec<usize>,
    /// 이동평균 유형
    pub ma: MAType,
    /// 이동평균 계산 기간
    pub ma_period: usize,
    /// ADX 계산 기간
    pub adx_period: usize,
}

impl Default for ThreeRSIStrategyConfigBase {
    /// 기본 설정값 반환
    fn default() -> Self {
        ThreeRSIStrategyConfigBase {
            rsi_periods: vec![6, 14, 26],
            ma: MAType::EMA,
            ma_period: 50,
            adx_period: 14,
        }
    }
}

impl ThreeRSIStrategyConfigBase {
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
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<ThreeRSIStrategyConfigBase, String> {
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

        let result = ThreeRSIStrategyConfigBase {
            rsi_periods,
            ma,
            ma_period,
            adx_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// ThreeRSI 전략 데이터
#[derive(Debug)]
pub struct ThreeRSIStrategyData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 세 가지 RSI 데이터
    pub rsis: RSIs,
    /// 이동평균선 데이터
    pub ma: Box<dyn MA>,
    /// ADX 지표 데이터
    pub adx: ADX,
}

impl<C: Candle> ThreeRSIStrategyData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, rsis: RSIs, ma: Box<dyn MA>, adx: ADX) -> ThreeRSIStrategyData<C> {
        ThreeRSIStrategyData {
            candle,
            rsis,
            ma,
            adx,
        }
    }

    /// 캔들이 이동평균선보다 높은지 확인 (조건 함수 사용)
    pub fn is_candle_greater_than<F, G>(&self, candle_fn: F, ma_fn: G) -> bool
    where
        F: Fn(&C) -> f64,
        G: Fn(&Self) -> f64,
    {
        candle_fn(&self.candle) > ma_fn(self)
    }

    /// 캔들이 이동평균선보다 낮은지 확인 (조건 함수 사용)
    pub fn is_candle_less_than<F, G>(&self, candle_fn: F, ma_fn: G) -> bool
    where
        F: Fn(&C) -> f64,
        G: Fn(&Self) -> f64,
    {
        candle_fn(&self.candle) < ma_fn(self)
    }
}

impl<C: Candle> GetCandle<C> for ThreeRSIStrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for ThreeRSIStrategyData<C> {}

/// ThreeRSI 전략 컨텍스트
#[derive(Debug)]
pub struct ThreeRSIStrategyContext<C: Candle> {
    /// RSIs 빌더 (세 가지 RSI 계산)
    pub rsisbuilder: RSIsBuilder<C>,
    /// 이동평균 빌더
    pub mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    /// ADX 빌더
    pub adxbuilder: ADXBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<ThreeRSIStrategyData<C>>,
}

impl<C: Candle> Display for ThreeRSIStrategyContext<C> {
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

impl<C: Candle + 'static> ThreeRSIStrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        config: &ThreeRSIStrategyConfigBase,
        storage: &CandleStore<C>,
    ) -> ThreeRSIStrategyContext<C> {
        let rsisbuilder = RSIsBuilderFactory::build::<C>(&config.rsi_periods);
        let mabuilder = MABuilderFactory::build::<C>(&config.ma, config.ma_period);
        let adxbuilder = ADXBuilder::<C>::new(config.adx_period);

        let mut ctx = ThreeRSIStrategyContext {
            rsisbuilder,
            mabuilder,
            adxbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// RSI 값이 모두 50 미만인지 확인 (약세)
    pub fn is_rsi_all_less_than_50(&self, n: usize) -> bool {
        self.is_all_less_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, n)
    }

    /// RSI 값이 모두 50 이상인지 확인 (강세)
    pub fn is_rsi_all_greater_than_50(&self, n: usize) -> bool {
        self.is_all_greater_than_target(|ctx| &ctx.rsis, |rsi| rsi.rsi, 50.0, n)
    }

    /// RSI가 역순 배열인지 확인 (단기가 더 작고 장기가 더 큼)
    pub fn is_rsi_reverse_arrangement(&self, n: usize) -> bool {
        self.is_reverse_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, n)
    }

    /// RSI가 정순 배열인지 확인 (단기가 더 크고 장기가 더 작음)
    pub fn is_rsi_regular_arrangement(&self, n: usize) -> bool {
        self.is_regular_arrangement(|ctx| &ctx.rsis, |rsi| rsi.rsi, n)
    }

    /// 캔들이 이동평균선보다 낮은지 확인 (약세)
    pub fn is_candle_low_below_ma(&self, n: usize) -> bool {
        self.is_candle_less_than(|candle| candle.low_price(), |ctx| ctx.ma.get(), n)
    }

    /// 캔들이 이동평균선보다 높은지 확인 (강세)
    pub fn is_candle_high_above_ma(&self, n: usize) -> bool {
        self.is_candle_greater_than(|candle| candle.high_price(), |ctx| ctx.ma.get(), n)
    }

    /// ADX가 20 이상인지 확인 (추세 강도)
    pub fn is_adx_greater_than_20(&self, n: usize) -> bool {
        self.is_greater_than_target(|ctx| ctx.adx.adx, 20.0, n)
    }
}

impl<C: Candle> StrategyContextOps<ThreeRSIStrategyData<C>, C> for ThreeRSIStrategyContext<C> {
    fn next_data(&mut self, candle: C) -> ThreeRSIStrategyData<C> {
        let rsis = self.rsisbuilder.next(&candle);
        let ma = self.mabuilder.next(&candle);
        let adx = self.adxbuilder.next(&candle);
        ThreeRSIStrategyData::new(candle, rsis, ma, adx)
    }

    fn datum(&self) -> &Vec<ThreeRSIStrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<ThreeRSIStrategyData<C>> {
        &mut self.items
    }
}

/// ThreeRSI 전략의 공통 트레이트
pub trait ThreeRSIStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &ThreeRSIStrategyContext<C>;
}
