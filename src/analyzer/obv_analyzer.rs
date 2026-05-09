use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::obv::{OBV, OBVBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug)]
pub struct OBVAnalyzerData<C: Candle> {
    pub candle: C,
    pub obv: OBV,
}

impl<C: Candle> OBVAnalyzerData<C> {
    pub fn new(candle: C, obv: OBV) -> Self {
        Self { candle, obv }
    }
}

impl<C: Candle> GetCandle<C> for OBVAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for OBVAnalyzerData<C> {}

#[derive(Debug)]
pub struct OBVAnalyzer<C: Candle> {
    pub builder: OBVBuilder<C>,
    pub items: Vec<OBVAnalyzerData<C>>,
}

impl<C: Candle> Display for OBVAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, OBV: {}", first.candle, first.obv),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> OBVAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>) -> OBVAnalyzer<C> {
        let mut analyzer = OBVAnalyzer {
            builder: OBVBuilder::new(),
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&OBVAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<OBVAnalyzerData<C>, C> for OBVAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> OBVAnalyzerData<C> {
        let obv = self.builder.next(&candle);
        OBVAnalyzerData::new(candle, obv)
    }

    fn items(&self) -> &Vec<OBVAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<OBVAnalyzerData<C>> {
        &mut self.items
    }
}
