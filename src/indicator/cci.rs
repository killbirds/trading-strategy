use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct CCIInput {
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Clone, Debug)]
pub struct CCI {
    period: usize,
    pub value: f64,
}

impl CCI {
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Display for CCI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CCI({}: {:.2})", self.period, self.value)
    }
}

#[derive(Debug)]
pub struct CCIBuilder<C: Candle> {
    period: usize,
    values: Vec<CCIInput>,
    _phantom: PhantomData<C>,
}

impl<C> CCIBuilder<C>
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
            return Err("CCI 기간은 0보다 커야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("CCI", period, 2, 0)?;

        Ok(Self {
            period,
            values: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> CCI {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> CCI {
        self.values.clear();

        if data.is_empty() {
            return self.empty_cci();
        }

        let mut cci = self.empty_cci();
        for item in data {
            cci = self.next(item);
        }

        cci
    }

    pub fn next(&mut self, data: &C) -> CCI {
        self.values.push(CCIInput {
            high: data.high_price(),
            low: data.low_price(),
            close: data.close_price(),
        });

        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        self.calculate()
    }

    fn empty_cci(&self) -> CCI {
        CCI {
            period: self.period,
            value: 0.0,
        }
    }

    fn calculate(&self) -> CCI {
        let recent = self.values.iter().rev().cloned().collect::<Vec<_>>();
        CCI {
            period: self.period,
            value: calculate_value(&recent, self.period),
        }
    }
}

impl<C> TABuilder<CCI, C> for CCIBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> CCI {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> CCI {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> CCI {
        self.next(data)
    }
}

fn calculate_value(candles: &[CCIInput], period: usize) -> f64 {
    if candles.len() < period {
        return 0.0;
    }

    let recent_candles = &candles[..period];
    let typical_prices: Vec<f64> = recent_candles
        .iter()
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();
    let sma = typical_prices.iter().sum::<f64>() / typical_prices.len() as f64;
    let current_typical = match typical_prices.first() {
        Some(&tp) => tp,
        None => return 0.0,
    };
    let mad = typical_prices
        .iter()
        .map(|&tp| (tp - sma).abs())
        .sum::<f64>()
        / typical_prices.len() as f64;

    if mad == 0.0 {
        return 0.0;
    }

    (current_typical - sma) / (0.015 * mad)
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
    fn test_cci_build_empty_data() {
        let mut builder = CCIBuilder::<TestCandle>::new(20);
        assert_eq!(builder.build(&[]).value, 0.0);
    }

    #[test]
    #[should_panic(expected = "CCI 기간은 0보다 커야 합니다")]
    fn test_cci_invalid_period() {
        CCIBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_cci_calculates_uptrend_value() {
        let candles = (0..30)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index, close + 1.0, close - 1.0, close)
            })
            .collect::<Vec<_>>();
        let mut builder = CCIBuilder::<TestCandle>::new(20);
        let cci = builder.build(&candles);

        assert!(cci.value > 0.0);
    }
}
