use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::kama::{KAMA, KAMABuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct KAMAAnalyzerParams {
    pub period: usize,
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Default for KAMAAnalyzerParams {
    fn default() -> Self {
        Self {
            period: 10,
            fast_period: 2,
            slow_period: 30,
        }
    }
}

#[derive(Debug)]
pub struct KAMAAnalyzerData<C: Candle> {
    pub candle: C,
    pub kama: KAMA,
}

impl<C: Candle> KAMAAnalyzerData<C> {
    pub fn new(candle: C, kama: KAMA) -> Self {
        Self { candle, kama }
    }
}

impl<C: Candle> GetCandle<C> for KAMAAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for KAMAAnalyzerData<C> {}

#[derive(Debug)]
pub struct KAMAAnalyzer<C: Candle> {
    pub params: KAMAAnalyzerParams,
    pub builder: KAMABuilder<C>,
    pub items: Vec<KAMAAnalyzerData<C>>,
}

impl<C: Candle> Display for KAMAAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, KAMA: {}", first.candle, first.kama),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> KAMAAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: KAMAAnalyzerParams) -> KAMAAnalyzer<C> {
        let builder = KAMABuilder::new(params.period, params.fast_period, params.slow_period);
        let mut analyzer = KAMAAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&KAMAAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<KAMAAnalyzerData<C>, C> for KAMAAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> KAMAAnalyzerData<C> {
        let kama = self.builder.next(&candle);
        KAMAAnalyzerData::new(candle, kama)
    }

    fn items(&self) -> &Vec<KAMAAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<KAMAAnalyzerData<C>> {
        &mut self.items
    }
}
