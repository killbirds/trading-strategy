use crate::candle_store::CandleStore;
use crate::indicator::utils::moving_average;
use crate::indicator::{IndicatorResult, TABuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct PPOBuilder<C: Candle> {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: Option<f64>,
    slow_ema: Option<f64>,
    signal_ema: Option<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct PPO {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    pub ppo: f64,
    pub signal: f64,
    pub histogram: f64,
}

impl PPO {
    pub fn fast_period(&self) -> usize {
        self.fast_period
    }
    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
    pub fn signal_period(&self) -> usize {
        self.signal_period
    }
    pub fn ppo(&self) -> f64 {
        self.ppo
    }
    pub fn signal(&self) -> f64 {
        self.signal
    }
    pub fn histogram(&self) -> f64 {
        self.histogram
    }
}

impl Display for PPO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PPO({},{},{}: {:.2}, {:.2}, {:.2})",
            self.fast_period,
            self.slow_period,
            self.signal_period,
            self.ppo,
            self.signal,
            self.histogram
        )
    }
}

impl<C> PPOBuilder<C>
where
    C: Candle,
{
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        match Self::new_checked(fast_period, slow_period, signal_period) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> IndicatorResult<Self> {
        if fast_period == 0 || slow_period == 0 || signal_period == 0 {
            return Err("PPO 기간은 0보다 커야 합니다".to_string());
        }
        if fast_period >= slow_period {
            return Err("PPO 빠른 기간은 느린 기간보다 작아야 합니다".to_string());
        }
        Ok(Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: None,
            slow_ema: None,
            signal_ema: None,
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> PPO {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> PPO {
        self.fast_ema = None;
        self.slow_ema = None;
        self.signal_ema = None;
        if data.is_empty() {
            return self.current(0.0);
        }
        let mut result = self.current(0.0);
        for candle in data {
            result = self.next(candle);
        }
        result
    }
    pub fn next(&mut self, data: &C) -> PPO {
        let close = data.close_price();
        self.fast_ema = Some(match self.fast_ema {
            Some(previous) => moving_average::calculate_ema_step(
                close,
                previous,
                moving_average::calculate_ema_alpha(self.fast_period),
            ),
            None => close,
        });
        self.slow_ema = Some(match self.slow_ema {
            Some(previous) => moving_average::calculate_ema_step(
                close,
                previous,
                moving_average::calculate_ema_alpha(self.slow_period),
            ),
            None => close,
        });
        let slow = self.slow_ema.unwrap_or(0.0);
        let fast = self.fast_ema.unwrap_or(0.0);
        let ppo = if slow.abs() < f64::EPSILON {
            0.0
        } else {
            ((fast - slow) / slow) * 100.0
        };
        self.signal_ema = Some(match self.signal_ema {
            Some(previous) => moving_average::calculate_ema_step(
                ppo,
                previous,
                moving_average::calculate_ema_alpha(self.signal_period),
            ),
            None => ppo,
        });
        self.current(ppo)
    }
    fn current(&self, ppo: f64) -> PPO {
        let signal = self.signal_ema.unwrap_or(0.0);
        PPO {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            ppo,
            signal,
            histogram: ppo - signal,
        }
    }
}

impl<C> TABuilder<PPO, C> for PPOBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> PPO {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> PPO {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> PPO {
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
        assert!(PPOBuilder::<TestCandle>::new_checked(0, 26, 9).is_err());
        assert!(PPOBuilder::<TestCandle>::new_checked(26, 12, 9).is_err());
    }
    #[test]
    fn calculates_percent_price_oscillator() {
        let data = vec![candle(1, 10.0), candle(2, 12.0), candle(3, 14.0)];
        let mut builder = PPOBuilder::<TestCandle>::new(2, 3, 2);
        let ppo = builder.build(&data);
        assert!(ppo.ppo() > 0.0);
        assert!((ppo.histogram() - (ppo.ppo() - ppo.signal())).abs() < 1e-9);
    }
}
