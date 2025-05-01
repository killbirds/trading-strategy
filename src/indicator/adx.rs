use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use ta_lib::average_directional_movement_index;
use trading_chart::Candle;

#[derive(Debug)]
struct AverageDirectionalMovementIndex {
    period: usize,
    high: Box<[f64]>,
    low: Box<[f64]>,
    close: Box<[f64]>,
}

impl AverageDirectionalMovementIndex {
    fn new(period: usize) -> AverageDirectionalMovementIndex {
        AverageDirectionalMovementIndex {
            period,
            high: vec![0.0; period * 2].into_boxed_slice(),
            low: vec![0.0; period * 2].into_boxed_slice(),
            close: vec![0.0; period * 2].into_boxed_slice(),
        }
    }
}

fn rotate_left_and_last_mut(items: &mut Box<[f64]>, new_last: f64) {
    items.rotate_left(1);
    if let Some(last) = items.last_mut() {
        *last = new_last;
    }
}

impl AverageDirectionalMovementIndex {
    fn next<T>(&mut self, input: &T) -> f64
    where
        T: Candle,
    {
        rotate_left_and_last_mut(&mut self.high, input.high_price());
        rotate_left_and_last_mut(&mut self.low, input.low_price());
        rotate_left_and_last_mut(&mut self.close, input.close_price());

        let (result, _) = average_directional_movement_index(
            &self.high,
            &self.low,
            &self.close,
            Some(self.period),
        )
        .unwrap();

        *result.last().unwrap_or(&0.0)
    }
}

#[derive(Debug)]
pub struct ADXBuilder<C: Candle> {
    period: usize,
    indicator: AverageDirectionalMovementIndex,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct ADX {
    period: usize,
    pub adx: f64,
}

impl Display for ADX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADX({}: {})", self.period, self.adx)
    }
}

impl<C> ADXBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> Self {
        let indicator = AverageDirectionalMovementIndex::new(period);
        ADXBuilder {
            period,
            indicator,
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> ADX {
        self.build(&storage.get_reversed_items())
    }

    pub fn build(&mut self, data: &[C]) -> ADX {
        let adx: f64 = data.iter().fold(0.0, |_, item| self.indicator.next(item));
        ADX {
            period: self.period,
            adx,
        }
    }

    pub fn next(&mut self, data: &C) -> ADX {
        let adx: f64 = self.indicator.next(data);
        ADX {
            period: self.period,
            adx,
        }
    }
}

impl<C> TABuilder<ADX, C> for ADXBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> ADX {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> ADX {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> ADX {
        self.next(data)
    }
}

pub type ADXs = TAs<usize, ADX>;
pub type ADXsBuilder<C> = TAsBuilder<usize, ADX, C>;

pub struct ADXsBuilderFactory;
impl ADXsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> ADXsBuilder<C> {
        ADXsBuilder::new("adxs".to_owned(), periods, |period| {
            Box::new(ADXBuilder::new(*period))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default, PartialEq)]
    struct TestData {
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    }

    impl TestData {
        fn new(high: f64, low: f64, close: f64) -> TestData {
            TestData {
                high,
                low,
                close,
                volume: 0.0,
            }
        }
    }

    impl Display for TestData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "TestData(H:{}, L:{}, C:{})",
                self.high, self.low, self.close
            )
        }
    }

    impl Candle for TestData {
        fn market(&self) -> &str {
            "TEST"
        }

        fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }

        fn candle_interval(&self) -> &trading_chart::CandleInterval {
            &trading_chart::CandleInterval::Minute1
        }

        fn open_price(&self) -> f64 {
            self.close
        }

        fn high_price(&self) -> f64 {
            self.high
        }

        fn low_price(&self) -> f64 {
            self.low
        }

        fn close_price(&self) -> f64 {
            self.close
        }

        fn acc_trade_price(&self) -> f64 {
            0.0
        }

        fn acc_trade_volume(&self) -> f64 {
            self.volume
        }
    }

    #[test]
    fn test_gen_adx() {
        let mut indicator = AverageDirectionalMovementIndex::new(5);
        let data: Vec<TestData> = vec![
            TestData::new(0.0, 0.0, 0.0),
            TestData::new(1.0, 1.0, 1.0),
            TestData::new(2.0, 1.0, 2.0),
            TestData::new(3.0, 1.0, 2.0),
            TestData::new(4.0, 1.0, 2.0),
            TestData::new(5.0, 1.0, 2.0),
            TestData::new(6.0, 1.0, 2.0),
            TestData::new(7.0, 1.0, 2.0),
            TestData::new(8.0, 1.0, 2.0),
            TestData::new(9.0, 1.0, 2.0),
            TestData::new(10.0, 1.0, 2.0),
        ];
        let adx: f64 = data.iter().fold(0.0, |_, item| indicator.next(item));

        println!("{:?}", adx);
    }
}
