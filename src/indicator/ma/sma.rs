use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use crate::indicator::utils::moving_average;
use std::fmt::Display;
use std::marker::PhantomData;
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
        self.build(&storage.get_time_ordered_items())
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

        // SMA 계산
        let sma = moving_average::calculate_sma(&self.values, self.period);

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

        // SMA 계산
        let sma =
            moving_average::calculate_sma_or_default(&self.values, self.period, data.close_price());

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
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 125.0,
                low: 105.0,
                close: 120.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 120.0,
                high: 125.0,
                low: 110.0,
                close: 115.0,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_sma_calculation() {
        let candles = create_test_candles();
        let mut builder = SMABuilder::new(2);

        // 첫 번째 계산
        let sma = builder.build(&candles);
        assert_eq!(sma.period(), 2);
        assert!(sma.get() > 0.0);

        // 새 캔들로 업데이트
        let new_candle = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            volume: 1300.0,
        };

        let updated_sma = builder.next(&new_candle);
        assert_eq!(updated_sma.period(), 2);
        assert!(updated_sma.get() > 0.0);
    }

    #[test]
    fn test_empty_data() {
        let mut builder = SMABuilder::<TestCandle>::new(5);
        let sma = builder.build(&[]);

        assert_eq!(sma.get(), 0.0);
        assert_eq!(sma.period(), 5);
    }

    #[test]
    fn test_sma_display() {
        let sma = SMA {
            period: 5,
            sma: 100.0,
        };

        let display_str = sma.to_string();
        assert!(display_str.contains("SMA"));
        assert!(display_str.contains("5"));
        assert!(display_str.contains("100"));
    }
}
