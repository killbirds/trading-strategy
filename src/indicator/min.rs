use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct MINBuilder<C: Candle> {
    period: usize,
    values: Vec<f64>,
    _phantom: PhantomData<C>,
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
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("MIN 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MIN {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> MIN {
        if data.is_empty() {
            return MIN {
                period: self.period,
                min: 0.0,
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.low_price());
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return MIN {
                period: self.period,
                min: *self.values.last().unwrap_or(&0.0),
            };
        }

        // 최소값 계산 (최근 period 개만 사용)
        let start_idx = self.values.len() - self.period;
        let min = self.values[start_idx..]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        MIN {
            period: self.period,
            min,
        }
    }

    pub fn next(&mut self, data: &C) -> MIN {
        // 새 가격 추가
        self.values.push(data.low_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return MIN {
                period: self.period,
                min: data.low_price(),
            };
        }

        // 최소값 계산 (최근 period 개만 사용)
        let start_idx = self.values.len() - self.period;
        let min = self.values[start_idx..]
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

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
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MIN {
        self.build_from_storage(storage)
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
                low: 85.0,
                close: 110.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 80.0,
                close: 115.0,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_min_calculation() {
        let mut builder = MINBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // 첫 번째 최소값 계산
        let min1 = builder.next(&candles[0]);
        assert_eq!(min1.period, 2);
        assert_eq!(min1.min, 90.0); // 첫 번째 캔들의 저가

        // 두 번째 최소값 계산
        let min2 = builder.next(&candles[1]);
        assert_eq!(min2.min, 85.0); // 두 번째 캔들의 저가

        // 세 번째 최소값 계산
        let min3 = builder.next(&candles[2]);
        assert_eq!(min3.min, 80.0); // 세 번째 캔들의 저가
    }

    #[test]
    fn test_min_period() {
        let mut builder = MINBuilder::<TestCandle>::new(3);
        let candles = create_test_candles();

        // 첫 번째 최소값 (기간 1)
        let min1 = builder.next(&candles[0]);
        assert_eq!(min1.min, 90.0);

        // 두 번째 최소값 (기간 2)
        let min2 = builder.next(&candles[1]);
        assert_eq!(min2.min, 85.0);

        // 세 번째 최소값 (기간 3)
        let min3 = builder.next(&candles[2]);
        assert_eq!(min3.min, 80.0);

        // 네 번째 최소값 (기간 3, 가장 오래된 값 제외)
        let min4 = builder.next(&candles[0]);
        assert_eq!(min4.min, 80.0); // 여전히 80.0이 최소값
    }

    #[test]
    fn test_min_trend() {
        let mut builder = MINBuilder::<TestCandle>::new(2);

        // 하락 추세 데이터
        let down_candles = [
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 110.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 100.0,
                low: 90.0,
                close: 90.0,
                volume: 1000.0,
            },
        ];

        let min1 = builder.next(&down_candles[0]);
        let min2 = builder.next(&down_candles[1]);
        assert!(min2.min < min1.min); // 하락 추세에서 최소값이 감소
    }

    #[test]
    fn test_min_consolidation() {
        let mut builder = MINBuilder::<TestCandle>::new(2);

        // 횡보장 데이터
        let consolidation_candles = [
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 100.0,
                volume: 1000.0,
            },
        ];

        let min1 = builder.next(&consolidation_candles[0]);
        let min2 = builder.next(&consolidation_candles[1]);
        assert_eq!(min2.min, min1.min); // 횡보장에서 최소값이 유지
    }

    #[test]
    fn test_build_method() {
        let mut builder = MINBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // build 메서드로 최소값 계산
        let min = builder.build(&candles);
        assert_eq!(min.period, 2);
        assert_eq!(min.min, 80.0); // 마지막 2개 중 최소값 (85.0, 80.0)
    }

    #[test]
    fn test_build_empty_data() {
        let mut builder = MINBuilder::<TestCandle>::new(2);
        let empty_candles: Vec<TestCandle> = vec![];

        let min = builder.build(&empty_candles);
        assert_eq!(min.period, 2);
        assert_eq!(min.min, 0.0); // 빈 데이터일 때 0.0 반환
    }

    #[test]
    fn test_build_insufficient_data() {
        let mut builder = MINBuilder::<TestCandle>::new(5);
        let candles = create_test_candles(); // 3개만 있음

        let min = builder.build(&candles);
        assert_eq!(min.period, 5);
        assert_eq!(min.min, 80.0); // 마지막 값 반환
    }

    #[test]
    fn test_min_known_values_accuracy() {
        // 알려진 MIN 계산 결과와 비교
        // period=2인 경우 간단한 계산으로 검증
        // 최소값 = min(최근 period개의 low)
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 105.0,
                high: 115.0,
                low: 90.0,
                close: 110.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 110.0,
                high: 120.0,
                low: 85.0,
                close: 115.0,
                volume: 1200.0,
            },
        ];

        let mut builder = MINBuilder::<TestCandle>::new(2);
        let min = builder.build(&candles);

        // 최근 2개의 low: 90.0, 85.0
        // 최소값 = min(90.0, 85.0) = 85.0
        let expected = 85.0;
        assert!(
            (min.min - expected).abs() < 0.01,
            "MIN calculation mismatch. Expected: {}, Got: {}",
            expected,
            min.min
        );
    }

    #[test]
    fn test_min_known_values_period_2() {
        // period=2인 경우 정확한 계산 검증
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 105.0,
                low: 98.0,
                close: 102.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 102.0,
                high: 108.0,
                low: 96.0,
                close: 104.0,
                volume: 1100.0,
            },
        ];

        let mut builder = MINBuilder::<TestCandle>::new(2);
        let min = builder.build(&candles);

        // 최근 2개의 low: 98.0, 96.0
        // 최소값 = min(98.0, 96.0) = 96.0
        let expected = 96.0;
        assert!(
            (min.min - expected).abs() < 0.01,
            "MIN calculation mismatch. Expected: {}, Got: {}",
            expected,
            min.min
        );
    }
}
