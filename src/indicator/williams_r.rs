use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct WilliamsRInput {
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Clone, Debug)]
pub struct WilliamsR {
    period: usize,
    pub value: f64,
}

impl WilliamsR {
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Display for WilliamsR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WilliamsR({}: {:.2})", self.period, self.value)
    }
}

#[derive(Debug)]
pub struct WilliamsRBuilder<C: Candle> {
    period: usize,
    values: Vec<WilliamsRInput>,
    _phantom: PhantomData<C>,
}

impl<C> WilliamsRBuilder<C>
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
            return Err("Williams %R 기간은 0보다 커야 합니다".to_string());
        }

        Ok(Self {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> WilliamsR {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> WilliamsR {
        self.values.clear();

        if data.is_empty() {
            return self.empty_williams_r();
        }

        let mut williams_r = self.empty_williams_r();
        for item in data {
            williams_r = self.next(item);
        }

        williams_r
    }

    pub fn next(&mut self, data: &C) -> WilliamsR {
        self.push(data);
        self.calculate()
    }

    fn push(&mut self, data: &C) {
        self.values.push(WilliamsRInput {
            high: data.high_price(),
            low: data.low_price(),
            close: data.close_price(),
        });

        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }

    fn empty_williams_r(&self) -> WilliamsR {
        WilliamsR {
            period: self.period,
            value: -50.0,
        }
    }

    fn calculate(&self) -> WilliamsR {
        let recent = self.values.iter().rev().cloned().collect::<Vec<_>>();
        WilliamsR {
            period: self.period,
            value: calculate_value(&recent, self.period),
        }
    }
}

impl<C> TABuilder<WilliamsR, C> for WilliamsRBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> WilliamsR {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> WilliamsR {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> WilliamsR {
        self.next(data)
    }
}

fn calculate_value(candles: &[WilliamsRInput], period: usize) -> f64 {
    if candles.len() < period {
        return -50.0;
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
        return -50.0;
    }

    -((highest_high - current_close) / (highest_high - lowest_low)) * 100.0
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

    #[test]
    fn test_williams_r_build_empty_data() {
        let mut builder = WilliamsRBuilder::<TestCandle>::new(14);
        assert_eq!(builder.build(&[]).value, -50.0);
    }

    #[test]
    #[should_panic(expected = "Williams %R 기간은 0보다 커야 합니다")]
    fn test_williams_r_invalid_period() {
        WilliamsRBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_williams_r_detects_uptrend_values() {
        let candles = (0..30)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index, close + 1.0, close - 1.0, close)
            })
            .collect::<Vec<_>>();
        let mut builder = WilliamsRBuilder::<TestCandle>::new(14);
        let williams_r = builder.build(&candles);

        assert!(williams_r.value > -10.0);
    }
}
