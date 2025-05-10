use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct MAXBuilder<C: Candle> {
    period: usize,
    values: Vec<f64>,
    _phantom: PhantomData<C>,
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
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("MAX 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> MAX {
        self.build(&storage.get_time_ordered_items())
    }

    pub fn build(&mut self, data: &[C]) -> MAX {
        if data.is_empty() {
            return MAX {
                period: self.period,
                max: 0.0,
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.high_price());
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return MAX {
                period: self.period,
                max: *self.values.last().unwrap_or(&0.0),
            };
        }

        // 최대값 계산
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        MAX {
            period: self.period,
            max,
        }
    }

    pub fn next(&mut self, data: &C) -> MAX {
        // 새 가격 추가
        self.values.push(data.high_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return MAX {
                period: self.period,
                max: data.high_price(),
            };
        }

        // 최대값 계산
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    use chrono::Utc;

    fn create_test_candles() -> Vec<TestCandle> {
        vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_max_calculation() {
        let mut builder = MAXBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // 첫 번째 최대값 계산
        let max1 = builder.next(&candles[0]);
        assert_eq!(max1.period, 2);
        assert_eq!(max1.max, 110.0); // 첫 번째 캔들의 고가

        // 두 번째 최대값 계산
        let max2 = builder.next(&candles[1]);
        assert_eq!(max2.max, 115.0); // 두 번째 캔들의 고가

        // 세 번째 최대값 계산
        let max3 = builder.next(&candles[2]);
        assert_eq!(max3.max, 120.0); // 세 번째 캔들의 고가
    }

    #[test]
    fn test_max_period() {
        let mut builder = MAXBuilder::<TestCandle>::new(3);
        let candles = create_test_candles();

        // 첫 번째 최대값 (기간 1)
        let max1 = builder.next(&candles[0]);
        assert_eq!(max1.max, 110.0);

        // 두 번째 최대값 (기간 2)
        let max2 = builder.next(&candles[1]);
        assert_eq!(max2.max, 115.0);

        // 세 번째 최대값 (기간 3)
        let max3 = builder.next(&candles[2]);
        assert_eq!(max3.max, 120.0);

        // 네 번째 최대값 (기간 3, 가장 오래된 값 제외)
        let max4 = builder.next(&candles[0]);
        assert_eq!(max4.max, 120.0); // 여전히 120.0이 최대값
    }

    #[test]
    fn test_max_trend() {
        let mut builder = MAXBuilder::<TestCandle>::new(2);

        // 상승 추세 데이터
        let up_candles = [
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 100.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 110.0,
                close: 120.0,
                volume: 1000.0,
            },
        ];

        let max1 = builder.next(&up_candles[0]);
        let max2 = builder.next(&up_candles[1]);
        assert!(max2.max > max1.max); // 상승 추세에서 최대값이 증가
    }

    #[test]
    fn test_max_consolidation() {
        let mut builder = MAXBuilder::<TestCandle>::new(2);

        // 횡보장 데이터
        let consolidation_candles = [
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 100.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 110.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
        ];

        let max1 = builder.next(&consolidation_candles[0]);
        let max2 = builder.next(&consolidation_candles[1]);
        assert_eq!(max2.max, max1.max); // 횡보장에서 최대값이 유지
    }
}
