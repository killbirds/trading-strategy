use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::parabolic_sar::{ParabolicSAR, ParabolicSARBuilder};
use std::fmt::Display;
use trading_chart::Candle;

#[derive(Debug, Clone)]
pub struct ParabolicSARAnalyzerParams {
    pub step: f64,
    pub max_step: f64,
}

impl Default for ParabolicSARAnalyzerParams {
    fn default() -> Self {
        Self {
            step: 0.02,
            max_step: 0.2,
        }
    }
}

#[derive(Debug)]
pub struct ParabolicSARAnalyzerData<C: Candle> {
    pub candle: C,
    pub parabolic_sar: ParabolicSAR,
}

impl<C: Candle> ParabolicSARAnalyzerData<C> {
    pub fn new(candle: C, parabolic_sar: ParabolicSAR) -> Self {
        Self {
            candle,
            parabolic_sar,
        }
    }
}

impl<C: Candle> GetCandle<C> for ParabolicSARAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ParabolicSARAnalyzerData<C> {}

#[derive(Debug)]
pub struct ParabolicSARAnalyzer<C: Candle> {
    pub params: ParabolicSARAnalyzerParams,
    pub builder: ParabolicSARBuilder<C>,
    pub items: Vec<ParabolicSARAnalyzerData<C>>,
}

impl<C: Candle> Display for ParabolicSARAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, ParabolicSAR: {}",
                first.candle, first.parabolic_sar
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> ParabolicSARAnalyzer<C> {
    pub fn new(
        storage: &CandleStore<C>,
        params: ParabolicSARAnalyzerParams,
    ) -> ParabolicSARAnalyzer<C> {
        let builder = ParabolicSARBuilder::new(params.step, params.max_step);
        let mut analyzer = ParabolicSARAnalyzer {
            params,
            builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    pub fn current(&self) -> Option<&ParabolicSARAnalyzerData<C>> {
        self.items.first()
    }
}

impl<C: Candle> AnalyzerOps<ParabolicSARAnalyzerData<C>, C> for ParabolicSARAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ParabolicSARAnalyzerData<C> {
        let parabolic_sar = self.builder.next(&candle);
        ParabolicSARAnalyzerData::new(candle, parabolic_sar)
    }

    fn items(&self) -> &Vec<ParabolicSARAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<ParabolicSARAnalyzerData<C>> {
        &mut self.items
    }
}
