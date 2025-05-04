use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::adx::{ADX, ADXBuilder};
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::rsi::{RSIs, RSIsBuilder, RSIsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// ThreeRSI 분석기 데이터
#[derive(Debug)]
pub struct ThreeRSIAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 세 가지 RSI 데이터
    pub rsis: RSIs,
    /// 이동평균선 데이터
    pub ma: Box<dyn MA>,
    /// ADX 지표 데이터
    pub adx: ADX,
}

impl<C: Candle> ThreeRSIAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, rsis: RSIs, ma: Box<dyn MA>, adx: ADX) -> ThreeRSIAnalyzerData<C> {
        ThreeRSIAnalyzerData {
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

impl<C: Candle> GetCandle<C> for ThreeRSIAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ThreeRSIAnalyzerData<C> {}

/// ThreeRSI 분석기 컨텍스트
#[derive(Debug)]
pub struct ThreeRSIAnalyzer<C: Candle> {
    /// RSIs 빌더 (세 가지 RSI 계산)
    pub rsisbuilder: RSIsBuilder<C>,
    /// 이동평균 빌더
    pub mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    /// ADX 빌더
    pub adxbuilder: ADXBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<ThreeRSIAnalyzerData<C>>,
}

impl<C: Candle> Display for ThreeRSIAnalyzer<C> {
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

impl<C: Candle + 'static> ThreeRSIAnalyzer<C> {
    /// 새 분석기 컨텍스트 생성
    pub fn new(
        rsi_periods: &[usize],
        ma_type: &MAType,
        ma_period: usize,
        adx_period: usize,
        storage: &CandleStore<C>,
    ) -> ThreeRSIAnalyzer<C> {
        let rsisbuilder = RSIsBuilderFactory::build::<C>(rsi_periods);
        let mabuilder = MABuilderFactory::build::<C>(ma_type, ma_period);
        let adxbuilder = ADXBuilder::<C>::new(adx_period);

        let mut ctx = ThreeRSIAnalyzer {
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
        self.is_all(
            |data| data.is_candle_less_than(|candle| candle.low_price(), |ctx| ctx.ma.get()),
            n,
        )
    }

    /// 캔들이 이동평균선보다 높은지 확인 (강세)
    pub fn is_candle_high_above_ma(&self, n: usize) -> bool {
        self.is_all(
            |data| data.is_candle_greater_than(|candle| candle.high_price(), |ctx| ctx.ma.get()),
            n,
        )
    }

    /// ADX가 20 이상인지 확인 (추세 강도)
    pub fn is_adx_greater_than_20(&self, n: usize) -> bool {
        self.is_greater_than_target(|ctx| ctx.adx.adx, 20.0, n)
    }
}

impl<C: Candle> AnalyzerOps<ThreeRSIAnalyzerData<C>, C> for ThreeRSIAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ThreeRSIAnalyzerData<C> {
        let rsis = self.rsisbuilder.next(&candle);
        let ma = self.mabuilder.next(&candle);
        let adx = self.adxbuilder.next(&candle);
        ThreeRSIAnalyzerData::new(candle, rsis, ma, adx)
    }

    fn datum(&self) -> &Vec<ThreeRSIAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<ThreeRSIAnalyzerData<C>> {
        &mut self.items
    }
}
