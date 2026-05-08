use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct StochasticInput {
    high: f64,
    low: f64,
    close: f64,
}

/// Stochastic oscillator indicator.
#[derive(Clone, Debug)]
pub struct Stochastic {
    period: usize,
    pub k: f64,
    pub d: f64,
}

impl Stochastic {
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Display for Stochastic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stochastic({}: {:.2}, {:.2})",
            self.period, self.k, self.d
        )
    }
}

/// Builder for the Stochastic oscillator.
#[derive(Debug)]
pub struct StochasticBuilder<C: Candle> {
    period: usize,
    values: Vec<StochasticInput>,
    k_values: Vec<f64>,
    _phantom: PhantomData<C>,
}

impl<C> StochasticBuilder<C>
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
            return Err("Stochastic 기간은 0보다 커야 합니다".to_string());
        }

        Ok(Self {
            period,
            values: Vec::with_capacity(period * 2),
            k_values: Vec::with_capacity(4),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Stochastic {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> Stochastic {
        self.values.clear();
        self.k_values.clear();

        if data.is_empty() {
            return self.empty_stochastic();
        }

        let mut stochastic = self.empty_stochastic();
        for item in data {
            stochastic = self.next(item);
        }

        stochastic
    }

    pub fn next(&mut self, data: &C) -> Stochastic {
        self.push(data);
        self.calculate()
    }

    fn push(&mut self, data: &C) {
        self.values.push(StochasticInput {
            high: data.high_price(),
            low: data.low_price(),
            close: data.close_price(),
        });

        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }

    fn empty_stochastic(&self) -> Stochastic {
        Stochastic {
            period: self.period,
            k: 50.0,
            d: 50.0,
        }
    }

    fn calculate(&mut self) -> Stochastic {
        let recent = self.recent_values();
        let k = calculate_k(&recent, self.period);

        self.k_values.push(k);
        if self.k_values.len() > 3 {
            let excess = self.k_values.len() - 3;
            self.k_values.drain(0..excess);
        }

        let d = if self.k_values.len() < 3 {
            *self.k_values.last().unwrap_or(&50.0)
        } else {
            self.k_values.iter().sum::<f64>() / 3.0
        };

        Stochastic {
            period: self.period,
            k,
            d,
        }
    }

    fn recent_values(&self) -> Vec<StochasticInput> {
        self.values.iter().rev().cloned().collect()
    }
}

impl<C> TABuilder<Stochastic, C> for StochasticBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Stochastic {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> Stochastic {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> Stochastic {
        self.next(data)
    }
}

fn calculate_k(candles: &[StochasticInput], period: usize) -> f64 {
    if candles.len() < period {
        return 50.0;
    }

    let recent_candles = &candles[..period];
    let highest_high = recent_candles.iter().map(|c| c.high).fold(0.0, f64::max);
    let lowest_low = recent_candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let current_close = match candles.first() {
        Some(c) => c.close,
        None => return 50.0,
    };

    if highest_high == lowest_low {
        return 50.0;
    }

    ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
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
            volume: 1000.0,
        }
    }

    fn uptrend_candles(count: usize) -> Vec<TestCandle> {
        (0..count)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index as i64, close + 1.0, close - 1.0, close)
            })
            .collect()
    }

    #[test]
    fn test_stochastic_build_empty_data() {
        let mut builder = StochasticBuilder::<TestCandle>::new(14);
        let stochastic = builder.build(&[]);

        assert_eq!(stochastic.k, 50.0);
        assert_eq!(stochastic.d, 50.0);
    }

    #[test]
    #[should_panic(expected = "Stochastic 기간은 0보다 커야 합니다")]
    fn test_stochastic_invalid_period() {
        StochasticBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_stochastic_detects_uptrend_values() {
        let candles = uptrend_candles(30);
        let mut builder = StochasticBuilder::<TestCandle>::new(14);
        let stochastic = builder.build(&candles);

        assert!(stochastic.k > 90.0);
        assert!(stochastic.d > 90.0);
    }

    #[test]
    fn test_stochastic_build_and_next_are_consistent() {
        let candles = uptrend_candles(30);
        let mut build_builder = StochasticBuilder::<TestCandle>::new(14);
        let built = build_builder.build(&candles);

        let mut next_builder = StochasticBuilder::<TestCandle>::new(14);
        let mut next = next_builder.build(&[]);
        for candle in &candles {
            next = next_builder.next(candle);
        }

        assert_eq!(built.k, next.k);
        assert_eq!(built.d, next.d);
    }
}
