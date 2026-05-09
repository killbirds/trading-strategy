use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::mfi::{MFI, MFIBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct MFIAnalyzerParams {
    pub period: usize,
}

impl Default for MFIAnalyzerParams {
    fn default() -> Self {
        Self { period: 14 }
    }
}

#[derive(Debug)]
pub struct MFIAnalyzerData<C: Candle> {
    pub candle: C,
    pub mfi: MFI,
}

impl<C: Candle> MFIAnalyzerData<C> {
    pub fn new(candle: C, mfi: MFI) -> Self {
        Self { candle, mfi }
    }
}

impl<C: Candle> GetCandle<C> for MFIAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for MFIAnalyzerData<C> {}

#[derive(Debug)]
pub struct MFIAnalyzer<C: Candle> {
    pub params: MFIAnalyzerParams,
    pub builder: MFIBuilder<C>,
    pub items: Vec<MFIAnalyzerData<C>>,
}

impl<C: Candle> Display for MFIAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MFI: {}", first.candle, first.mfi),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> MFIAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: MFIAnalyzerParams) -> MFIAnalyzer<C> {
        let builder = MFIBuilder::new(params.period);
        let mut analyzer = MFIAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&MFIAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<MFIAnalyzerData<C>, C> for MFIAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> MFIAnalyzerData<C> {
        let mfi = self.builder.next(&candle);
        MFIAnalyzerData::new(candle, mfi)
    }

    fn items(&self) -> &Vec<MFIAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<MFIAnalyzerData<C>> {
        &mut self.items
    }
}
