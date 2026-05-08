use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct OBVBuilder<C: Candle> {
    previous_close: Option<f64>,
    value: f64,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct OBV {
    pub value: f64,
}

impl OBV {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Display for OBV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OBV({:.2})", self.value)
    }
}

impl<C> OBVBuilder<C>
where
    C: Candle,
{
    pub fn new() -> Self {
        Self::new_checked()
    }

    pub fn new_checked() -> Self {
        Self {
            previous_close: None,
            value: 0.0,
            _phantom: PhantomData,
        }
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> OBV {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> OBV {
        self.previous_close = None;
        self.value = 0.0;
        if data.is_empty() {
            return self.current();
        }
        for candle in data {
            self.next(candle);
        }
        self.current()
    }

    pub fn next(&mut self, data: &C) -> OBV {
        let close = data.close_price();
        if let Some(previous_close) = self.previous_close {
            if close > previous_close {
                self.value += data.volume();
            } else if close < previous_close {
                self.value -= data.volume();
            }
        }
        self.previous_close = Some(close);
        self.current()
    }

    fn current(&self) -> OBV {
        OBV { value: self.value }
    }
}

impl<C> Default for OBVBuilder<C>
where
    C: Candle,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> TABuilder<OBV, C> for OBVBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> OBV {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> OBV {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> OBV {
        self.next(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn candle(timestamp: i64, close: f64, volume: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high: close,
            low: close,
            close,
            volume,
        }
    }

    #[test]
    fn calculates_cumulative_obv() {
        let data = vec![
            candle(1, 10.0, 100.0),
            candle(2, 11.0, 50.0),
            candle(3, 9.0, 30.0),
            candle(4, 9.0, 20.0),
        ];
        let mut builder = OBVBuilder::<TestCandle>::new();
        let obv = builder.build(&data);
        assert_eq!(obv.value(), 20.0);
    }

    #[test]
    fn build_resets_state() {
        let mut builder = OBVBuilder::<TestCandle>::new();
        builder.next(&candle(1, 10.0, 100.0));
        builder.next(&candle(2, 11.0, 50.0));
        assert_eq!(builder.build(&[]).value(), 0.0);
    }
}
