use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::ppo::{PPO, PPOBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct PPOAnalyzerParams {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for PPOAnalyzerParams {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

#[derive(Debug)]
pub struct PPOAnalyzerData<C: Candle> {
    pub candle: C,
    pub ppo: PPO,
}

impl<C: Candle> PPOAnalyzerData<C> {
    pub fn new(candle: C, ppo: PPO) -> Self {
        Self { candle, ppo }
    }
}

impl<C: Candle> GetCandle<C> for PPOAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for PPOAnalyzerData<C> {}

#[derive(Debug)]
pub struct PPOAnalyzer<C: Candle> {
    pub params: PPOAnalyzerParams,
    pub builder: PPOBuilder<C>,
    pub items: Vec<PPOAnalyzerData<C>>,
}

impl<C: Candle> Display for PPOAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, PPO: {}", first.candle, first.ppo),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> PPOAnalyzer<C> {
    pub fn new(storage: &CandleStore<C>, params: PPOAnalyzerParams) -> PPOAnalyzer<C> {
        let builder = PPOBuilder::new(params.fast_period, params.slow_period, params.signal_period);
        let mut analyzer = PPOAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&PPOAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<PPOAnalyzerData<C>, C> for PPOAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> PPOAnalyzerData<C> {
        let ppo = self.builder.next(&candle);
        PPOAnalyzerData::new(candle, ppo)
    }

    fn items(&self) -> &Vec<PPOAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<PPOAnalyzerData<C>> {
        &mut self.items
    }
}
