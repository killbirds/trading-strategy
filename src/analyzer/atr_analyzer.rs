use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::atr::{ATRs, ATRsBuilder, ATRsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// ATR 분석기 데이터
#[derive(Debug)]
pub struct ATRAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// ATR 지표 집합
    pub atrs: ATRs,
}

impl<C: Candle> ATRAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, atrs: ATRs) -> ATRAnalyzerData<C> {
        ATRAnalyzerData { candle, atrs }
    }

    /// 특정 ATR 값 반환
    pub fn get_atr(&self, period: usize) -> f64 {
        self.atrs.get(&period).value
    }
}

impl<C: Candle> GetCandle<C> for ATRAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ATRAnalyzerData<C> {}

/// ATR 분석기
#[derive(Debug)]
pub struct ATRAnalyzer<C: Candle> {
    /// ATR 빌더
    pub atrsbuilder: ATRsBuilder<C>,
    /// 분석 데이터 히스토리
    pub items: Vec<ATRAnalyzerData<C>>,
}

impl<C: Candle> Display for ATRAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATRAnalyzer {{ items: {} }}", self.items.len())
    }
}

impl<C: Candle> ATRAnalyzer<C> {
    /// 새 ATR 분석기 생성
    pub fn new(periods: &[usize], storage: &CandleStore<C>) -> ATRAnalyzer<C> {
        let atrsbuilder = ATRsBuilderFactory::new(periods);
        let mut analyzer = ATRAnalyzer {
            atrsbuilder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 저장소에서 초기 데이터 계산
    pub fn init_from_storage(&mut self, storage: &CandleStore<C>) {
        for candle in storage.get_time_ordered_items().iter() {
            self.next(candle.clone());
        }
    }

    /// 특정 기간과 배수로 ATR 상단값 계산
    pub fn calculate_upper_band(&self, candle: &C, period: usize, multiplier: f64) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let atr = self.items[0].get_atr(period);
        let price = (candle.high_price() + candle.low_price()) / 2.0;
        price + (atr * multiplier)
    }

    /// 특정 기간과 배수로 ATR 하단값 계산
    pub fn calculate_lower_band(&self, candle: &C, period: usize, multiplier: f64) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let atr = self.items[0].get_atr(period);
        let price = (candle.high_price() + candle.low_price()) / 2.0;
        price - (atr * multiplier)
    }

    /// 현재 ATR이 특정 임계값을 초과하는지 확인
    pub fn is_above_threshold(&self, period: usize, threshold: f64) -> bool {
        if self.items.is_empty() {
            return false;
        }

        self.items[0].get_atr(period) > threshold
    }

    /// 현재 ATR이 이전 n개 캔들의 평균 ATR보다 높은지 확인 (변동성 확대)
    pub fn is_volatility_expanding(&self, period: usize, n: usize) -> bool {
        if self.items.len() <= n {
            return false;
        }

        let current_atr = self.items[0].get_atr(period);
        let avg_atr: f64 = self.items[1..=n]
            .iter()
            .map(|item| item.get_atr(period))
            .sum::<f64>()
            / n as f64;

        current_atr > avg_atr
    }

    /// 현재 ATR이 이전 n개 캔들의 평균 ATR보다 낮은지 확인 (변동성 축소)
    pub fn is_volatility_contracting(&self, period: usize, n: usize) -> bool {
        if self.items.len() <= n {
            return false;
        }

        let current_atr = self.items[0].get_atr(period);
        let avg_atr: f64 = self.items[1..=n]
            .iter()
            .map(|item| item.get_atr(period))
            .sum::<f64>()
            / n as f64;

        current_atr < avg_atr
    }
}

impl<C: Candle> AnalyzerOps<ATRAnalyzerData<C>, C> for ATRAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ATRAnalyzerData<C> {
        let atrs = self.atrsbuilder.next(&candle);
        ATRAnalyzerData::new(candle, atrs)
    }

    fn datum(&self) -> &Vec<ATRAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<ATRAnalyzerData<C>> {
        &mut self.items
    }
}
