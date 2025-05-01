use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct MINBuilder<C: Candle> {
    period: usize,
    indicator: MinimumIndicator,
    _phantom: PhantomData<C>,
}

// Simple implementation for minimum indicator
#[derive(Debug)]
struct MinimumIndicator {
    period: usize,
    values: Vec<f64>,
}

impl MinimumIndicator {
    fn new(period: usize) -> Self {
        Self {
            period,
            values: Vec::with_capacity(period),
        }
    }

    fn next(&mut self, value: &impl Candle) -> f64 {
        let price = value.low_price();
        if self.values.len() >= self.period {
            self.values.remove(0);
        }
        self.values.push(price);

        if self.values.is_empty() {
            return 0.0;
        }

        // Calculate minimum manually since ta-lib doesn't have a direct min function
        self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
}

#[derive(Clone, Debug)]
pub struct MIN {
    period: usize,
    pub min: f64,
}

impl Display for MIN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MIN({}: {})", self.period, self.min)
    }
}

impl<C> MINBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> MINBuilder<C> {
        let indicator = MinimumIndicator::new(period);
        MINBuilder {
            period,
            indicator,
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> MIN {
        self.build(&storage.get_reversed_items())
    }

    pub fn build(&mut self, data: &[C]) -> MIN {
        let min: f64 = data.iter().fold(0.0, |_, item| self.indicator.next(item));
        MIN {
            period: self.period,
            min,
        }
    }

    pub fn next(&mut self, data: &C) -> MIN {
        let min: f64 = self.indicator.next(data);
        MIN {
            period: self.period,
            min,
        }
    }
}

impl<C> TABuilder<MIN, C> for MINBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> MIN {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> MIN {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> MIN {
        self.next(data)
    }
}

pub type MINs = TAs<usize, MIN>;
pub type MINsBuilder<C> = TAsBuilder<usize, MIN, C>;

pub struct MINsBuilderFactory;
impl MINsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> MINsBuilder<C> {
        MINsBuilder::new("mins".to_owned(), periods, |period| {
            Box::new(MINBuilder::<C>::new(*period))
        })
    }
}
