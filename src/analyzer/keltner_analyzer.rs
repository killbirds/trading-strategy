use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::keltner::{KeltnerChannel, KeltnerChannelBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct KeltnerAnalyzerParams {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for KeltnerAnalyzerParams {
    fn default() -> Self {
        Self {
            period: 20,
            multiplier: 2.0,
        }
    }
}

#[derive(Debug)]
pub struct KeltnerAnalyzerData<C: Candle> {
    pub candle: C,
    pub keltner: KeltnerChannel,
}

impl<C: Candle> KeltnerAnalyzerData<C> {
    pub fn new(candle: C, keltner: KeltnerChannel) -> Self {
        Self { candle, keltner }
    }
}

impl<C: Candle> GetCandle<C> for KeltnerAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for KeltnerAnalyzerData<C> {}

#[derive(Debug)]
pub struct KeltnerAnalyzer<C: Candle> {
    pub params: KeltnerAnalyzerParams,
    pub builder: KeltnerChannelBuilder<C>,
    pub items: Vec<KeltnerAnalyzerData<C>>,
}

impl<C: Candle> Display for KeltnerAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Keltner: {}", first.candle, first.keltner),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> KeltnerAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: KeltnerAnalyzerParams) -> KeltnerAnalyzer<C> {
        let builder = KeltnerChannelBuilder::new(params.period, params.multiplier);
        let mut analyzer = KeltnerAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&KeltnerAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<KeltnerAnalyzerData<C>, C> for KeltnerAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> KeltnerAnalyzerData<C> {
        let keltner = self.builder.next(&candle);
        KeltnerAnalyzerData::new(candle, keltner)
    }

    fn items(&self) -> &Vec<KeltnerAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<KeltnerAnalyzerData<C>> {
        &mut self.items
    }
}
