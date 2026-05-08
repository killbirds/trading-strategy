use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, TAs, TAsBuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct ChopInput {
    high: f64,
    low: f64,
    tr: f64,
}

#[derive(Debug)]
pub struct ChoppinessIndexBuilder<C: Candle> {
    period: usize,
    values: Vec<ChopInput>,
    previous_close: Option<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct ChoppinessIndex {
    period: usize,
    sample_count: usize,
    pub value: f64,
}

impl ChoppinessIndex {
    pub fn period(&self) -> usize {
        self.period
    }
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Display for ChoppinessIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChoppinessIndex({}: {:.2})", self.period, self.value)
    }
}

impl<C> ChoppinessIndexBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> Self {
        match Self::new_checked(period) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err("Choppiness Index 기간은 0보다 커야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("Choppiness Index", period, 2, 0)?;

        Ok(Self {
            period,
            values: Vec::with_capacity(capacity),
            previous_close: None,
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ChoppinessIndex {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> ChoppinessIndex {
        self.values.clear();
        self.previous_close = None;
        if data.is_empty() {
            return self.calculate();
        }
        for candle in data {
            self.push(candle);
        }
        self.calculate()
    }
    pub fn next(&mut self, data: &C) -> ChoppinessIndex {
        self.push(data);
        self.calculate()
    }
    fn push(&mut self, data: &C) {
        let high = data.high_price();
        let low = data.low_price();
        let close = data.close_price();
        let tr = if let Some(previous_close) = self.previous_close {
            (high - low)
                .max((high - previous_close).abs())
                .max((low - previous_close).abs())
        } else {
            high - low
        };
        self.previous_close = Some(close);
        self.values.push(ChopInput { high, low, tr });
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }
    fn calculate(&self) -> ChoppinessIndex {
        if self.values.is_empty() {
            return ChoppinessIndex {
                period: self.period,
                sample_count: 0,
                value: 0.0,
            };
        }
        let start = self.values.len().saturating_sub(self.period);
        let slice = &self.values[start..];
        let high = slice
            .iter()
            .fold(f64::NEG_INFINITY, |current, item| current.max(item.high));
        let low = slice
            .iter()
            .fold(f64::INFINITY, |current, item| current.min(item.low));
        let range = high - low;
        let sum_tr = slice.iter().map(|item| item.tr).sum::<f64>();
        let value = if slice.len() < 2 {
            0.0
        } else if range <= 0.0 {
            100.0
        } else if sum_tr <= 0.0 {
            0.0
        } else {
            100.0 * (sum_tr / range).log10() / (self.period as f64).log10()
        };
        ChoppinessIndex {
            period: self.period,
            sample_count: slice.len(),
            value,
        }
    }
}

impl<C> TABuilder<ChoppinessIndex, C> for ChoppinessIndexBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ChoppinessIndex {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> ChoppinessIndex {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> ChoppinessIndex {
        self.next(data)
    }
}

pub type ChoppinessIndexes = TAs<usize, ChoppinessIndex>;
pub type ChoppinessIndexesBuilder<C> = TAsBuilder<usize, ChoppinessIndex, C>;
pub struct ChoppinessIndexesBuilderFactory;
impl ChoppinessIndexesBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> ChoppinessIndexesBuilder<C> {
        match Self::build_checked(periods) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn build_checked<C: Candle + 'static>(
        periods: &[usize],
    ) -> IndicatorResult<ChoppinessIndexesBuilder<C>> {
        for period in periods {
            ChoppinessIndexBuilder::<C>::new_checked(*period)?;
        }
        Ok(ChoppinessIndexesBuilder::new(
            "choppiness_indexes".to_owned(),
            periods,
            |period| Box::new(ChoppinessIndexBuilder::<C>::new(*period)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1.0,
        }
    }
    #[test]
    fn rejects_zero_period() {
        assert!(ChoppinessIndexBuilder::<TestCandle>::new_checked(0).is_err());
        assert!(ChoppinessIndexBuilder::<TestCandle>::new_checked(usize::MAX).is_err());
    }
    #[test]
    fn calculates_choppiness_index() {
        let data = vec![
            candle(1, 10.0, 8.0, 9.0),
            candle(2, 11.0, 9.0, 10.0),
            candle(3, 12.0, 10.0, 11.0),
        ];
        let mut builder = ChoppinessIndexBuilder::<TestCandle>::new(3);
        let chop = builder.build(&data);
        let expected = 100.0 * (6.0_f64 / 4.0).log10() / 3.0_f64.log10();
        assert!((chop.value() - expected).abs() < 1e-9);
    }

    #[test]
    fn returns_max_choppiness_for_flat_range() {
        let data = vec![
            candle(1, 10.0, 10.0, 10.0),
            candle(2, 10.0, 10.0, 10.0),
            candle(3, 10.0, 10.0, 10.0),
        ];
        let mut builder = ChoppinessIndexBuilder::<TestCandle>::new(3);
        let chop = builder.build(&data);

        assert_eq!(chop.value(), 100.0);
    }
}
