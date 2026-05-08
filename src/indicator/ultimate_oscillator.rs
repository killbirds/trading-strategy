use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct UltimateOscillatorInput {
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Clone, Debug)]
pub struct UltimateOscillator {
    pub value: f64,
}

impl Display for UltimateOscillator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UltimateOscillator({:.2})", self.value)
    }
}

#[derive(Debug)]
pub struct UltimateOscillatorBuilder<C: Candle> {
    values: Vec<UltimateOscillatorInput>,
    _phantom: PhantomData<C>,
}

impl<C> UltimateOscillatorBuilder<C>
where
    C: Candle,
{
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(56),
            _phantom: PhantomData,
        }
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> UltimateOscillator {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> UltimateOscillator {
        self.values.clear();

        if data.is_empty() {
            return self.empty_ultimate_oscillator();
        }

        let mut ultimate_oscillator = self.empty_ultimate_oscillator();
        for item in data {
            ultimate_oscillator = self.next(item);
        }

        ultimate_oscillator
    }

    pub fn next(&mut self, data: &C) -> UltimateOscillator {
        self.values.push(UltimateOscillatorInput {
            high: data.high_price(),
            low: data.low_price(),
            close: data.close_price(),
        });

        if self.values.len() > 56 {
            let excess = self.values.len() - 56;
            self.values.drain(0..excess);
        }

        self.calculate()
    }

    fn empty_ultimate_oscillator(&self) -> UltimateOscillator {
        UltimateOscillator { value: 50.0 }
    }

    fn calculate(&self) -> UltimateOscillator {
        let recent = self.values.iter().rev().cloned().collect::<Vec<_>>();
        UltimateOscillator {
            value: calculate_value(&recent),
        }
    }
}

impl<C> Default for UltimateOscillatorBuilder<C>
where
    C: Candle,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> TABuilder<UltimateOscillator, C> for UltimateOscillatorBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> UltimateOscillator {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> UltimateOscillator {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> UltimateOscillator {
        self.next(data)
    }
}

fn calculate_value(candles: &[UltimateOscillatorInput]) -> f64 {
    if candles.len() < 28 {
        return 50.0;
    }

    let calculate_bp_tr =
        |current: &UltimateOscillatorInput, previous: &UltimateOscillatorInput| -> (f64, f64) {
            let bp = current.close - current.low.min(previous.close);
            let tr = current.high.max(previous.close) - current.low.min(previous.close);
            (bp, tr)
        };

    let mut bp_sum_7 = 0.0;
    let mut tr_sum_7 = 0.0;
    let mut bp_sum_14 = 0.0;
    let mut tr_sum_14 = 0.0;
    let mut bp_sum_28 = 0.0;
    let mut tr_sum_28 = 0.0;

    for i in 0..28.min(candles.len() - 1) {
        let (bp, tr) = calculate_bp_tr(&candles[i], &candles[i + 1]);

        if i < 7 {
            bp_sum_7 += bp;
            tr_sum_7 += tr;
        }
        if i < 14 {
            bp_sum_14 += bp;
            tr_sum_14 += tr;
        }
        bp_sum_28 += bp;
        tr_sum_28 += tr;
    }

    let avg_7 = if tr_sum_7 != 0.0 {
        bp_sum_7 / tr_sum_7
    } else {
        0.0
    };
    let avg_14 = if tr_sum_14 != 0.0 {
        bp_sum_14 / tr_sum_14
    } else {
        0.0
    };
    let avg_28 = if tr_sum_28 != 0.0 {
        bp_sum_28 / tr_sum_28
    } else {
        0.0
    };

    ((4.0 * avg_7) + (2.0 * avg_14) + avg_28) / 7.0 * 100.0
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
    fn test_ultimate_oscillator_build_empty_data() {
        let mut builder = UltimateOscillatorBuilder::<TestCandle>::new();
        assert_eq!(builder.build(&[]).value, 50.0);
    }

    #[test]
    fn test_ultimate_oscillator_waits_for_long_period() {
        let candles = (0..27)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index, close + 1.0, close - 1.0, close)
            })
            .collect::<Vec<_>>();
        let mut builder = UltimateOscillatorBuilder::<TestCandle>::new();

        assert_eq!(builder.build(&candles).value, 50.0);
    }

    #[test]
    fn test_ultimate_oscillator_calculates_uptrend_value() {
        let candles = (0..30)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index, close + 1.0, close - 5.0, close)
            })
            .collect::<Vec<_>>();
        let mut builder = UltimateOscillatorBuilder::<TestCandle>::new();
        let ultimate_oscillator = builder.build(&candles);

        assert!(ultimate_oscillator.value > 50.0);
    }
}
