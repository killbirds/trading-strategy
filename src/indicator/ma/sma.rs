use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use std::fmt::Display;
use std::marker::PhantomData;
use ta_lib::simple_moving_average;
use trading_chart::Candle;

#[derive(Debug)]
pub struct SMABuilder<C: Candle> {
    period: usize,
    values: Vec<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct SMA {
    period: usize,
    sma: f64,
}

impl Display for SMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SMA({}: {})", self.period, self.sma)
    }
}

impl MA for SMA {
    fn get(&self) -> f64 {
        self.sma
    }

    fn period(&self) -> usize {
        self.period
    }
}

impl<C> SMABuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> Self {
        SMABuilder {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> SMA {
        self.build(&storage.get_reversed_items())
    }

    pub fn build(&mut self, data: &[C]) -> SMA {
        if data.is_empty() {
            return SMA {
                period: self.period,
                sma: 0.0,
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.close_price());
        }

        // ta-lib으로 SMA 계산
        let (result, _) = simple_moving_average(&self.values, Some(self.period)).unwrap();
        let sma = *result.last().unwrap_or(&0.0);

        SMA {
            period: self.period,
            sma,
        }
    }

    pub fn next(&mut self, data: &C) -> SMA {
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return SMA {
                period: self.period,
                sma: data.close_price(),
            };
        }

        // ta-lib으로 SMA 계산
        let (result, _) = simple_moving_average(&self.values, Some(self.period)).unwrap();
        let sma = *result.last().unwrap_or(&0.0);

        SMA {
            period: self.period,
            sma,
        }
    }
}

impl<C> TABuilder<Box<dyn MA>, C> for SMABuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> Box<dyn MA> {
        Box::new(self.from_storage(storage))
    }

    fn build(&mut self, data: &[C]) -> Box<dyn MA> {
        Box::new(self.build(data))
    }

    fn next(&mut self, data: &C) -> Box<dyn MA> {
        Box::new(self.next(data))
    }
}
