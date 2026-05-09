use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::aroon::{Aroon, AroonBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct AroonAnalyzerParams {
    pub period: usize,
}

impl Default for AroonAnalyzerParams {
    fn default() -> Self {
        Self { period: 25 }
    }
}

#[derive(Debug)]
pub struct AroonAnalyzerData<C: Candle> {
    pub candle: C,
    pub aroon: Aroon,
}

impl<C: Candle> AroonAnalyzerData<C> {
    pub fn new(candle: C, aroon: Aroon) -> Self {
        Self { candle, aroon }
    }
}

impl<C: Candle> GetCandle<C> for AroonAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for AroonAnalyzerData<C> {}

#[derive(Debug)]
pub struct AroonAnalyzer<C: Candle> {
    pub params: AroonAnalyzerParams,
    pub builder: AroonBuilder<C>,
    pub items: Vec<AroonAnalyzerData<C>>,
}

impl<C: Candle> Display for AroonAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Aroon: {}", first.candle, first.aroon),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> AroonAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: AroonAnalyzerParams) -> AroonAnalyzer<C> {
        let builder = AroonBuilder::new(params.period);
        let mut analyzer = AroonAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&AroonAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<AroonAnalyzerData<C>, C> for AroonAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> AroonAnalyzerData<C> {
        let aroon = self.builder.next(&candle);
        AroonAnalyzerData::new(candle, aroon)
    }

    fn items(&self) -> &Vec<AroonAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<AroonAnalyzerData<C>> {
        &mut self.items
    }
}
