use crate::candle_store::CandleStore;
use crate::indicator::utils::moving_average;
use crate::indicator::{IndicatorResult, TABuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct KeltnerChannelBuilder<C: Candle> {
    period: usize,
    multiplier: f64,
    count: usize,
    previous_close: Option<f64>,
    previous_ema: Option<f64>,
    previous_atr: Option<f64>,
    tr_values: Vec<f64>,
    close_values: Vec<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct KeltnerChannel {
    period: usize,
    multiplier: f64,
    pub middle: f64,
    pub upper: f64,
    pub lower: f64,
    pub atr: f64,
}

impl KeltnerChannel {
    pub fn period(&self) -> usize {
        self.period
    }
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
    pub fn middle(&self) -> f64 {
        self.middle
    }
    pub fn upper(&self) -> f64 {
        self.upper
    }
    pub fn lower(&self) -> f64 {
        self.lower
    }
    pub fn atr(&self) -> f64 {
        self.atr
    }
}

impl Display for KeltnerChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KeltnerChannel({}: {:.2}, {:.2}, {:.2})",
            self.period, self.upper, self.middle, self.lower
        )
    }
}

impl<C> KeltnerChannelBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize, multiplier: f64) -> Self {
        match Self::new_checked(period, multiplier) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn new_checked(period: usize, multiplier: f64) -> IndicatorResult<Self> {
        if period == 0 {
            return Err("Keltner Channel 기간은 0보다 커야 합니다".to_string());
        }
        if multiplier <= 0.0 || !multiplier.is_finite() {
            return Err("Keltner Channel 승수는 유효한 양수여야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("Keltner Channel", period, 2, 0)?;

        Ok(Self {
            period,
            multiplier,
            count: 0,
            previous_close: None,
            previous_ema: None,
            previous_atr: None,
            tr_values: Vec::with_capacity(capacity),
            close_values: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> KeltnerChannel {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> KeltnerChannel {
        self.reset();
        if data.is_empty() {
            return self.current();
        }
        let mut result = self.current();
        for candle in data {
            result = self.next(candle);
        }
        result
    }

    pub fn next(&mut self, data: &C) -> KeltnerChannel {
        let close = data.close_price();
        let tr = if let Some(previous_close) = self.previous_close {
            (data.high_price() - data.low_price())
                .max((data.high_price() - previous_close).abs())
                .max((data.low_price() - previous_close).abs())
        } else {
            data.high_price() - data.low_price()
        };
        self.previous_close = Some(close);
        self.count += 1;
        self.close_values.push(close);
        self.tr_values.push(tr);
        if self.close_values.len() > self.period * 2 {
            let excess = self.close_values.len() - self.period * 2;
            self.close_values.drain(0..excess);
        }
        if self.tr_values.len() > self.period * 2 {
            let excess = self.tr_values.len() - self.period * 2;
            self.tr_values.drain(0..excess);
        }
        let alpha = moving_average::calculate_ema_alpha(self.period);
        let ema = match self.previous_ema {
            Some(previous) => moving_average::calculate_ema_step(close, previous, alpha),
            None if self.close_values.len() >= self.period => {
                moving_average::calculate_sma(&self.close_values, self.period)
            }
            None => close,
        };
        if self.close_values.len() >= self.period {
            self.previous_ema = Some(ema);
        }
        let atr = match self.previous_atr {
            Some(previous) => (previous * (self.period as f64 - 1.0) + tr) / self.period as f64,
            None if self.tr_values.len() >= self.period => {
                moving_average::calculate_sma(&self.tr_values, self.period)
            }
            None => 0.0,
        };
        if self.tr_values.len() >= self.period {
            self.previous_atr = Some(atr);
        }
        self.with_values(ema, atr)
    }

    fn reset(&mut self) {
        self.count = 0;
        self.previous_close = None;
        self.previous_ema = None;
        self.previous_atr = None;
        self.tr_values.clear();
        self.close_values.clear();
    }

    fn current(&self) -> KeltnerChannel {
        self.with_values(
            self.previous_ema.unwrap_or(0.0),
            self.previous_atr.unwrap_or(0.0),
        )
    }

    fn with_values(&self, middle: f64, atr: f64) -> KeltnerChannel {
        KeltnerChannel {
            period: self.period,
            multiplier: self.multiplier,
            middle,
            upper: middle + self.multiplier * atr,
            lower: middle - self.multiplier * atr,
            atr,
        }
    }
}

impl<C> TABuilder<KeltnerChannel, C> for KeltnerChannelBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> KeltnerChannel {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> KeltnerChannel {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> KeltnerChannel {
        self.next(data)
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
    fn rejects_invalid_inputs() {
        assert!(KeltnerChannelBuilder::<TestCandle>::new_checked(0, 2.0).is_err());
        assert!(KeltnerChannelBuilder::<TestCandle>::new_checked(20, 0.0).is_err());
        assert!(KeltnerChannelBuilder::<TestCandle>::new_checked(usize::MAX, 2.0).is_err());
    }

    #[test]
    fn calculates_ema_close_and_wilder_atr_bands() {
        let data = vec![
            candle(1, 11.0, 9.0, 10.0),
            candle(2, 12.0, 10.0, 11.0),
            candle(3, 13.0, 11.0, 12.0),
        ];
        let mut builder = KeltnerChannelBuilder::<TestCandle>::new(3, 2.0);
        let channel = builder.build(&data);
        assert!((channel.middle() - 11.0).abs() < 1e-9);
        assert!((channel.atr() - 2.0).abs() < 1e-9);
        assert!((channel.upper() - 15.0).abs() < 1e-9);
        assert!((channel.lower() - 7.0).abs() < 1e-9);
    }
}
