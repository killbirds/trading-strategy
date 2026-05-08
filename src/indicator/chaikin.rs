use crate::candle_store::CandleStore;
use crate::indicator::utils::moving_average;
use crate::indicator::{IndicatorResult, TABuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct ChaikinInput {
    money_flow_volume: f64,
    volume: f64,
}

#[derive(Debug)]
pub struct ChaikinBuilder<C: Candle> {
    cmf_period: usize,
    fast_period: usize,
    slow_period: usize,
    values: Vec<ChaikinInput>,
    adl: f64,
    fast_ema: Option<f64>,
    slow_ema: Option<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct Chaikin {
    cmf_period: usize,
    fast_period: usize,
    slow_period: usize,
    pub adl: f64,
    pub cmf: f64,
    pub adosc: f64,
}

impl Chaikin {
    pub fn cmf_period(&self) -> usize {
        self.cmf_period
    }
    pub fn fast_period(&self) -> usize {
        self.fast_period
    }
    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
    pub fn adl(&self) -> f64 {
        self.adl
    }
    pub fn cmf(&self) -> f64 {
        self.cmf
    }
    pub fn adosc(&self) -> f64 {
        self.adosc
    }
}

impl Display for Chaikin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chaikin({},{},{}: ADL={:.2}, CMF={:.2}, ADOSC={:.2})",
            self.cmf_period, self.fast_period, self.slow_period, self.adl, self.cmf, self.adosc
        )
    }
}

impl<C> ChaikinBuilder<C>
where
    C: Candle,
{
    pub fn new(cmf_period: usize, fast_period: usize, slow_period: usize) -> Self {
        match Self::new_checked(cmf_period, fast_period, slow_period) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(
        cmf_period: usize,
        fast_period: usize,
        slow_period: usize,
    ) -> IndicatorResult<Self> {
        if cmf_period == 0 || fast_period == 0 || slow_period == 0 {
            return Err("Chaikin 기간은 0보다 커야 합니다".to_string());
        }
        if fast_period >= slow_period {
            return Err("Chaikin 빠른 기간은 느린 기간보다 작아야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("Chaikin", cmf_period.max(slow_period), 2, 0)?;

        Ok(Self {
            cmf_period,
            fast_period,
            slow_period,
            values: Vec::with_capacity(capacity),
            adl: 0.0,
            fast_ema: None,
            slow_ema: None,
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Chaikin {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> Chaikin {
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
    pub fn next(&mut self, data: &C) -> Chaikin {
        let high = data.high_price();
        let low = data.low_price();
        let close = data.close_price();
        let volume = data.volume();
        let multiplier = if (high - low).abs() < f64::EPSILON {
            0.0
        } else {
            ((close - low) - (high - close)) / (high - low)
        };
        let money_flow_volume = multiplier * volume;
        self.adl += money_flow_volume;
        self.update_emas();
        self.values.push(ChaikinInput {
            money_flow_volume,
            volume,
        });
        if self.values.len() > self.cmf_period.max(self.slow_period) * 2 {
            let excess = self.values.len() - self.cmf_period.max(self.slow_period) * 2;
            self.values.drain(0..excess);
        }
        self.current()
    }
    fn update_emas(&mut self) {
        let fast_alpha = moving_average::calculate_ema_alpha(self.fast_period);
        let slow_alpha = moving_average::calculate_ema_alpha(self.slow_period);
        self.fast_ema = Some(match self.fast_ema {
            Some(previous) => moving_average::calculate_ema_step(self.adl, previous, fast_alpha),
            None => self.adl,
        });
        self.slow_ema = Some(match self.slow_ema {
            Some(previous) => moving_average::calculate_ema_step(self.adl, previous, slow_alpha),
            None => self.adl,
        });
    }
    fn cmf(&self) -> f64 {
        let start = self.values.len().saturating_sub(self.cmf_period);
        let slice = &self.values[start..];
        let volume_sum = slice.iter().map(|item| item.volume).sum::<f64>();
        if volume_sum.abs() < f64::EPSILON {
            0.0
        } else {
            slice.iter().map(|item| item.money_flow_volume).sum::<f64>() / volume_sum
        }
    }
    fn reset(&mut self) {
        self.values.clear();
        self.adl = 0.0;
        self.fast_ema = None;
        self.slow_ema = None;
    }
    fn current(&self) -> Chaikin {
        Chaikin {
            cmf_period: self.cmf_period,
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            adl: self.adl,
            cmf: self.cmf(),
            adosc: self.fast_ema.unwrap_or(0.0) - self.slow_ema.unwrap_or(0.0),
        }
    }
}

impl<C> TABuilder<Chaikin, C> for ChaikinBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Chaikin {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> Chaikin {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> Chaikin {
        self.next(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    fn candle(timestamp: i64, high: f64, low: f64, close: f64, volume: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume,
        }
    }
    #[test]
    fn rejects_invalid_periods() {
        assert!(ChaikinBuilder::<TestCandle>::new_checked(0, 3, 10).is_err());
        assert!(ChaikinBuilder::<TestCandle>::new_checked(20, 10, 3).is_err());
        assert!(ChaikinBuilder::<TestCandle>::new_checked(usize::MAX, 3, 10).is_err());
    }
    #[test]
    fn calculates_adl_cmf_and_adosc() {
        let data = vec![
            candle(1, 10.0, 0.0, 7.5, 100.0),
            candle(2, 10.0, 0.0, 2.5, 100.0),
        ];
        let mut builder = ChaikinBuilder::<TestCandle>::new(2, 3, 10);
        let chaikin = builder.build(&data);
        assert_eq!(chaikin.adl(), 0.0);
        assert_eq!(chaikin.cmf(), 0.0);
        assert!(chaikin.adosc().abs() > 0.0);
    }
}
