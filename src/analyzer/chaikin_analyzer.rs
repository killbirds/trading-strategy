use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::chaikin::{Chaikin, ChaikinBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct ChaikinAnalyzerParams {
    pub cmf_period: usize,
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Default for ChaikinAnalyzerParams {
    fn default() -> Self {
        Self {
            cmf_period: 20,
            fast_period: 3,
            slow_period: 10,
        }
    }
}

#[derive(Debug)]
pub struct ChaikinAnalyzerData<C: Candle> {
    pub candle: C,
    pub chaikin: Chaikin,
}

impl<C: Candle> ChaikinAnalyzerData<C> {
    pub fn new(candle: C, chaikin: Chaikin) -> Self {
        Self { candle, chaikin }
    }
}

impl<C: Candle> GetCandle<C> for ChaikinAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ChaikinAnalyzerData<C> {}

#[derive(Debug)]
pub struct ChaikinAnalyzer<C: Candle> {
    pub params: ChaikinAnalyzerParams,
    pub builder: ChaikinBuilder<C>,
    pub items: Vec<ChaikinAnalyzerData<C>>,
}

impl<C: Candle> Display for ChaikinAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Chaikin: {}", first.candle, first.chaikin),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> ChaikinAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: ChaikinAnalyzerParams) -> ChaikinAnalyzer<C> {
        let builder =
            ChaikinBuilder::new(params.cmf_period, params.fast_period, params.slow_period);
        let mut analyzer = ChaikinAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&ChaikinAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<ChaikinAnalyzerData<C>, C> for ChaikinAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ChaikinAnalyzerData<C> {
        let chaikin = self.builder.next(&candle);
        ChaikinAnalyzerData::new(candle, chaikin)
    }

    fn items(&self) -> &Vec<ChaikinAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<ChaikinAnalyzerData<C>> {
        &mut self.items
    }
}
