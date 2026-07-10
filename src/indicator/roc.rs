use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
pub struct ROC {
    period: usize,
    pub value: f64,
}

impl ROC {
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Display for ROC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ROC({}: {:.2})", self.period, self.value)
    }
}

#[derive(Debug)]
pub struct ROCBuilder<C: Candle> {
    period: usize,
    close_values: Vec<f64>,
    _phantom: PhantomData<C>,
}

impl<C> ROCBuilder<C>
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
            return Err("ROC 기간은 0보다 커야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("ROC", period, 2, 0)?;

        Ok(Self {
            period,
            close_values: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ROC {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> ROC {
        self.close_values.clear();

        if data.is_empty() {
            return self.empty_roc();
        }

        let mut roc = self.empty_roc();
        for item in data {
            roc = self.next(item);
        }

        roc
    }

    pub fn next(&mut self, data: &C) -> ROC {
        self.close_values.push(data.close_price());
        if self.close_values.len() > self.period * 2 {
            let excess = self.close_values.len() - self.period * 2;
            self.close_values.drain(0..excess);
        }

        self.calculate()
    }

    fn empty_roc(&self) -> ROC {
        ROC {
            period: self.period,
            value: 0.0,
        }
    }

    fn calculate(&self) -> ROC {
        // 표준 ROC(N) = (close[t] / close[t-N] - 1) * 100 이므로 N+1개 봉이 필요하다.
        if self.close_values.len() <= self.period {
            return self.empty_roc();
        }

        let current_price = match self.close_values.last() {
            Some(value) => *value,
            None => return self.empty_roc(),
        };
        let past_index = self.close_values.len() - self.period - 1;
        let past_price = self.close_values[past_index];
        let value = if past_price == 0.0 {
            0.0
        } else {
            ((current_price - past_price) / past_price) * 100.0
        };

        ROC {
            period: self.period,
            value,
        }
    }
}

impl<C> TABuilder<ROC, C> for ROCBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ROC {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> ROC {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> ROC {
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
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_roc_build_empty_data() {
        let mut builder = ROCBuilder::<TestCandle>::new(10);
        assert_eq!(builder.build(&[]).value, 0.0);
    }

    #[test]
    #[should_panic(expected = "ROC 기간은 0보다 커야 합니다")]
    fn test_roc_invalid_period() {
        ROCBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_roc_calculates_rate_of_change() {
        // close[0..=10] = [100, 101, ..., 110]
        // ROC(10) = (close[10] / close[0] - 1) * 100 = (110/100 - 1)*100 = 10.0
        let candles = (0..=10)
            .map(|index| candle(index, 100.0 + index as f64))
            .collect::<Vec<_>>();
        let mut builder = ROCBuilder::<TestCandle>::new(10);
        let roc = builder.build(&candles);

        assert_eq!(roc.value, 10.0);
    }

    #[test]
    fn test_roc_returns_zero_when_data_eq_period() {
        // 표준 ROC(N)은 N+1개 봉이 필요하다 — 정확히 N개일 때는 계산할 수 없다.
        let candles = (0..10)
            .map(|index| candle(index, 100.0 + index as f64))
            .collect::<Vec<_>>();
        let mut builder = ROCBuilder::<TestCandle>::new(10);
        let roc = builder.build(&candles);
        assert_eq!(roc.value, 0.0);
    }
}
