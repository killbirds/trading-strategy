use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, TAs, TAsBuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct MoneyFlow {
    typical_price: f64,
    raw_flow: f64,
}

#[derive(Debug)]
pub struct MFIBuilder<C: Candle> {
    period: usize,
    flows: Vec<MoneyFlow>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct MFI {
    period: usize,
    sample_count: usize,
    pub value: f64,
}

impl MFI {
    pub fn period(&self) -> usize {
        self.period
    }
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Display for MFI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MFI({}: {:.2})", self.period, self.value)
    }
}

impl<C> MFIBuilder<C>
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
            return Err("MFI 기간은 0보다 커야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("MFI", period, 2, 1)?;

        Ok(Self {
            period,
            flows: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MFI {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> MFI {
        self.flows.clear();
        if data.is_empty() {
            return self.calculate();
        }
        for candle in data {
            self.push(candle);
        }
        self.calculate()
    }

    pub fn next(&mut self, data: &C) -> MFI {
        self.push(data);
        self.calculate()
    }

    fn push(&mut self, data: &C) {
        let typical_price = (data.high_price() + data.low_price() + data.close_price()) / 3.0;
        self.flows.push(MoneyFlow {
            typical_price,
            raw_flow: typical_price * data.volume(),
        });
        if self.flows.len() > self.period * 2 + 1 {
            let excess = self.flows.len() - (self.period * 2 + 1);
            self.flows.drain(0..excess);
        }
    }

    fn calculate(&self) -> MFI {
        if self.flows.len() < 2 {
            return MFI {
                period: self.period,
                sample_count: self.flows.len(),
                value: 50.0,
            };
        }
        let start = self.flows.len().saturating_sub(self.period + 1);
        let slice = &self.flows[start..];
        let mut positive = 0.0;
        let mut negative = 0.0;
        for pair in slice.windows(2) {
            let current = &pair[1];
            let previous = &pair[0];
            if current.typical_price > previous.typical_price {
                positive += current.raw_flow;
            } else if current.typical_price < previous.typical_price {
                negative += current.raw_flow;
            }
        }
        let value = if negative.abs() < f64::EPSILON {
            if positive.abs() < f64::EPSILON {
                50.0
            } else {
                100.0
            }
        } else {
            100.0 - (100.0 / (1.0 + positive / negative))
        };
        MFI {
            period: self.period,
            sample_count: slice.len().saturating_sub(1),
            value,
        }
    }
}

impl<C> TABuilder<MFI, C> for MFIBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MFI {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> MFI {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> MFI {
        self.next(data)
    }
}

pub type MFIs = TAs<usize, MFI>;
pub type MFIsBuilder<C> = TAsBuilder<usize, MFI, C>;

pub struct MFIsBuilderFactory;
impl MFIsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> MFIsBuilder<C> {
        match Self::build_checked(periods) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn build_checked<C: Candle + 'static>(
        periods: &[usize],
    ) -> IndicatorResult<MFIsBuilder<C>> {
        for period in periods {
            MFIBuilder::<C>::new_checked(*period)?;
        }
        Ok(MFIsBuilder::new("mfis".to_owned(), periods, |period| {
            Box::new(MFIBuilder::<C>::new(*period))
        }))
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
    fn rejects_zero_period() {
        assert!(MFIBuilder::<TestCandle>::new_checked(0).is_err());
        assert!(MFIBuilder::<TestCandle>::new_checked(usize::MAX).is_err());
    }

    #[test]
    fn calculates_money_flow_index() {
        let data = vec![
            candle(1, 11.0, 9.0, 10.0, 10.0),
            candle(2, 12.0, 10.0, 11.0, 10.0),
            candle(3, 11.0, 9.0, 10.0, 10.0),
        ];
        let mut builder = MFIBuilder::<TestCandle>::new(2);
        let mfi = builder.build(&data);
        assert!((mfi.value() - 52.3809523809).abs() < 1e-9);
    }
}
