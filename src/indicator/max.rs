use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct MAXBuilder<C: Candle> {
    period: usize,
    indicator: MaximumIndicator,
    _phantom: PhantomData<C>,
}

// Simple implementation for maximum indicator
#[derive(Debug)]
struct MaximumIndicator {
    period: usize,
    values: Vec<f64>,
}

impl MaximumIndicator {
    fn new(period: usize) -> Self {
        Self {
            period,
            values: Vec::with_capacity(period),
        }
    }

    fn next(&mut self, value: &impl Candle) -> f64 {
        let price = value.high_price();
        if self.values.len() >= self.period {
            self.values.remove(0);
        }
        self.values.push(price);

        if self.values.is_empty() {
            return 0.0;
        }

        // Calculate maximum manually since ta-lib doesn't have a direct max function
        self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }
}

#[derive(Clone, Debug)]
pub struct MAX {
    period: usize,
    pub max: f64,
}

impl Display for MAX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MAX({}: {})", self.period, self.max)
    }
}

impl<C> MAXBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> MAXBuilder<C> {
        let indicator = MaximumIndicator::new(period);
        MAXBuilder {
            period,
            indicator,
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> MAX {
        self.build(&storage.get_reversed_items())
    }

    pub fn build(&mut self, data: &[C]) -> MAX {
        let max: f64 = data.iter().fold(0.0, |_, item| self.indicator.next(item));
        MAX {
            period: self.period,
            max,
        }
    }

    pub fn next(&mut self, data: &C) -> MAX {
        let max: f64 = self.indicator.next(data);
        MAX {
            period: self.period,
            max,
        }
    }
}

impl<C> TABuilder<MAX, C> for MAXBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> MAX {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> MAX {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> MAX {
        self.next(data)
    }
}

pub type MAXs = TAs<usize, MAX>;
pub type MAXsBuilder<C> = TAsBuilder<usize, MAX, C>;

pub struct MAXsBuilderFactory;
impl MAXsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> MAXsBuilder<C> {
        MAXsBuilder::new("maxs".to_owned(), periods, |period| {
            Box::new(MAXBuilder::<C>::new(*period))
        })
    }
}
