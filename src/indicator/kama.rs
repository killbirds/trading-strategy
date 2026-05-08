use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct KAMABuilder<C: Candle> {
    period: usize,
    fast_period: usize,
    slow_period: usize,
    values: Vec<f64>,
    previous_kama: Option<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct KAMA {
    period: usize,
    fast_period: usize,
    slow_period: usize,
    pub value: f64,
    pub efficiency_ratio: f64,
}

impl KAMA {
    pub fn period(&self) -> usize {
        self.period
    }
    pub fn fast_period(&self) -> usize {
        self.fast_period
    }
    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn efficiency_ratio(&self) -> f64 {
        self.efficiency_ratio
    }
}

impl Display for KAMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KAMA({},{},{}: {:.2}, ER={:.2})",
            self.period, self.fast_period, self.slow_period, self.value, self.efficiency_ratio
        )
    }
}

impl<C> KAMABuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize, fast_period: usize, slow_period: usize) -> Self {
        match Self::new_checked(period, fast_period, slow_period) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(
        period: usize,
        fast_period: usize,
        slow_period: usize,
    ) -> IndicatorResult<Self> {
        if period == 0 || fast_period == 0 || slow_period == 0 {
            return Err("KAMA 기간은 0보다 커야 합니다".to_string());
        }
        if fast_period >= slow_period {
            return Err("KAMA 빠른 기간은 느린 기간보다 작아야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("KAMA", period, 2, 1)?;

        Ok(Self {
            period,
            fast_period,
            slow_period,
            values: Vec::with_capacity(capacity),
            previous_kama: None,
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> KAMA {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> KAMA {
        self.values.clear();
        self.previous_kama = None;
        if data.is_empty() {
            return self.current(0.0);
        }
        let mut result = self.current(0.0);
        for candle in data {
            result = self.next(candle);
        }
        result
    }
    pub fn next(&mut self, data: &C) -> KAMA {
        let close = data.close_price();
        self.values.push(close);
        if self.values.len() > self.period * 2 + 1 {
            let excess = self.values.len() - (self.period * 2 + 1);
            self.values.drain(0..excess);
        }
        if self.values.len() <= self.period {
            self.previous_kama = Some(close);
            return self.current(0.0);
        }
        let er = self.efficiency_ratio();
        let fast_sc = 2.0 / (self.fast_period as f64 + 1.0);
        let slow_sc = 2.0 / (self.slow_period as f64 + 1.0);
        let smoothing_constant = (er * (fast_sc - slow_sc) + slow_sc).powi(2);
        let previous = self.previous_kama.unwrap_or(close);
        let kama = previous + smoothing_constant * (close - previous);
        self.previous_kama = Some(kama);
        self.current(er)
    }
    fn efficiency_ratio(&self) -> f64 {
        if self.values.len() <= self.period {
            return 0.0;
        }
        let last_idx = self.values.len() - 1;
        let change = (self.values[last_idx] - self.values[last_idx - self.period]).abs();
        let start = self.values.len() - self.period - 1;
        let volatility = self.values[start..]
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .sum::<f64>();
        if volatility.abs() < f64::EPSILON {
            0.0
        } else {
            change / volatility
        }
    }
    fn current(&self, efficiency_ratio: f64) -> KAMA {
        KAMA {
            period: self.period,
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            value: self.previous_kama.unwrap_or(0.0),
            efficiency_ratio,
        }
    }
}

impl<C> TABuilder<KAMA, C> for KAMABuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> KAMA {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> KAMA {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> KAMA {
        self.next(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    fn candle(timestamp: i64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1.0,
        }
    }
    #[test]
    fn rejects_invalid_periods() {
        assert!(KAMABuilder::<TestCandle>::new_checked(0, 2, 30).is_err());
        assert!(KAMABuilder::<TestCandle>::new_checked(10, 30, 2).is_err());
        assert!(KAMABuilder::<TestCandle>::new_checked(usize::MAX, 2, 30).is_err());
    }
    #[test]
    fn calculates_kama_and_efficiency_ratio() {
        let data = vec![
            candle(1, 10.0),
            candle(2, 11.0),
            candle(3, 12.0),
            candle(4, 13.0),
        ];
        let mut builder = KAMABuilder::<TestCandle>::new(3, 2, 30);
        let kama = builder.build(&data);
        let sc = (1.0_f64 * (2.0 / 3.0 - 2.0 / 31.0) + 2.0 / 31.0).powi(2);
        assert!((kama.efficiency_ratio() - 1.0).abs() < 1e-9);
        assert!((kama.value() - (13.0)).abs() < 1.0);
        assert!(sc > 0.0);
    }
}
