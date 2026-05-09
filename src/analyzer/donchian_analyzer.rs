use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::donchian::{DonchianChannel, DonchianChannelBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct DonchianAnalyzerParams {
    pub period: usize,
}

impl Default for DonchianAnalyzerParams {
    fn default() -> Self {
        Self { period: 20 }
    }
}

#[derive(Debug)]
pub struct DonchianAnalyzerData<C: Candle> {
    pub candle: C,
    pub donchian: DonchianChannel,
}

impl<C: Candle> DonchianAnalyzerData<C> {
    pub fn new(candle: C, donchian: DonchianChannel) -> Self {
        Self { candle, donchian }
    }
}

impl<C: Candle> GetCandle<C> for DonchianAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for DonchianAnalyzerData<C> {}

#[derive(Debug)]
pub struct DonchianAnalyzer<C: Candle> {
    pub params: DonchianAnalyzerParams,
    pub builder: DonchianChannelBuilder<C>,
    pub items: Vec<DonchianAnalyzerData<C>>,
}

impl<C: Candle> Display for DonchianAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Donchian: {}", first.candle, first.donchian),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> DonchianAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: DonchianAnalyzerParams) -> DonchianAnalyzer<C> {
        let builder = DonchianChannelBuilder::new(params.period);
        let mut analyzer = DonchianAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&DonchianAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<DonchianAnalyzerData<C>, C> for DonchianAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> DonchianAnalyzerData<C> {
        let donchian = self.builder.next(&candle);
        DonchianAnalyzerData::new(candle, donchian)
    }

    fn items(&self) -> &Vec<DonchianAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<DonchianAnalyzerData<C>> {
        &mut self.items
    }
}
