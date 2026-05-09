use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::choppiness::{ChoppinessIndex, ChoppinessIndexBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct ChoppinessAnalyzerParams {
    pub period: usize,
}

impl Default for ChoppinessAnalyzerParams {
    fn default() -> Self {
        Self { period: 14 }
    }
}

#[derive(Debug)]
pub struct ChoppinessAnalyzerData<C: Candle> {
    pub candle: C,
    pub choppiness: ChoppinessIndex,
}

impl<C: Candle> ChoppinessAnalyzerData<C> {
    pub fn new(candle: C, choppiness: ChoppinessIndex) -> Self {
        Self { candle, choppiness }
    }
}

impl<C: Candle> GetCandle<C> for ChoppinessAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ChoppinessAnalyzerData<C> {}

#[derive(Debug)]
pub struct ChoppinessAnalyzer<C: Candle> {
    pub params: ChoppinessAnalyzerParams,
    pub builder: ChoppinessIndexBuilder<C>,
    pub items: Vec<ChoppinessAnalyzerData<C>>,
}

impl<C: Candle> Display for ChoppinessAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, Choppiness: {}",
                first.candle, first.choppiness
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> ChoppinessAnalyzer<C> {
    pub fn new(
        storage: &CandleStore<C>,
        params: ChoppinessAnalyzerParams,
    ) -> ChoppinessAnalyzer<C> {
        let builder = ChoppinessIndexBuilder::new(params.period);
        let mut analyzer = ChoppinessAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&ChoppinessAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<ChoppinessAnalyzerData<C>, C> for ChoppinessAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ChoppinessAnalyzerData<C> {
        let choppiness = self.builder.next(&candle);
        ChoppinessAnalyzerData::new(candle, choppiness)
    }

    fn items(&self) -> &Vec<ChoppinessAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<ChoppinessAnalyzerData<C>> {
        &mut self.items
    }
}
