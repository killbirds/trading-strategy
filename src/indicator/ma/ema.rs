use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use std::fmt::Display;
use std::marker::PhantomData;
use ta_lib::exponential_moving_average;
use trading_chart::Candle;

/// 지수이동평균(EMA) 계산 빌더
///
/// 지수이동평균은 최근 데이터에 더 높은 가중치를 부여하는 이동평균입니다.
#[derive(Debug)]
pub struct EMABuilder<C: Candle> {
    /// EMA 계산 기간
    pub period: usize,
    /// 가격 데이터 저장용 배열
    values: Vec<f64>,
    _phantom: PhantomData<C>,
}

/// 지수이동평균(EMA) 기술적 지표
///
/// 계산된 EMA 값을 저장하고 제공합니다.
#[derive(Clone, Debug)]
pub struct EMA {
    /// EMA 계산 기간
    period: usize,
    /// 계산된 EMA 값
    ema: f64,
}

impl Display for EMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EMA({}: {:.2})", self.period, self.ema)
    }
}

impl MA for EMA {
    fn get(&self) -> f64 {
        self.ema
    }

    fn period(&self) -> usize {
        self.period
    }
}

impl<C> EMABuilder<C>
where
    C: Candle,
{
    /// 새 EMA 빌더 생성
    ///
    /// # Arguments
    /// * `period` - EMA 계산 기간
    ///
    /// # Returns
    /// * `EMABuilder` - 새 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("EMA 기간은 0보다 커야 합니다");
        }

        EMABuilder {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 EMA 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `EMA` - 계산된 EMA 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> EMA {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 EMA 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `EMA` - 계산된 EMA 지표
    pub fn build(&mut self, data: &[C]) -> EMA {
        if data.is_empty() {
            return EMA {
                period: self.period,
                ema: 0.0, // 데이터가 없으면 기본값 0 반환
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.close_price());
        }

        // ta-lib으로 EMA 계산
        let (result, _) = exponential_moving_average(&self.values, Some(self.period)).unwrap();
        let ema = *result.last().unwrap_or(&0.0);

        EMA {
            period: self.period,
            ema,
        }
    }

    /// 새 캔들 데이터로 EMA 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `EMA` - 업데이트된 EMA 지표
    pub fn next(&mut self, data: &C) -> EMA {
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return EMA {
                period: self.period,
                ema: data.close_price(),
            };
        }

        // ta-lib으로 EMA 계산
        let (result, _) = exponential_moving_average(&self.values, Some(self.period)).unwrap();
        let ema = *result.last().unwrap_or(&0.0);

        EMA {
            period: self.period,
            ema,
        }
    }
}

impl<C> TABuilder<Box<dyn MA>, C> for EMABuilder<C>
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
    fn test_ema_calculation() {
        let candles = create_test_candles();
        let mut builder = EMABuilder::new(2);

        // 첫 번째 계산
        let ema = builder.build(&candles);
        assert_eq!(ema.period(), 2);
        assert!(ema.get() > 0.0);

        // 새 캔들로 업데이트
        let new_candle = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            volume: 1300.0,
        };

        let updated_ema = builder.next(&new_candle);
        assert_eq!(updated_ema.period(), 2);
        assert!(updated_ema.get() > 0.0);
    }

    #[test]
    #[should_panic(expected = "EMA 기간은 0보다 커야 합니다")]
    fn test_invalid_period() {
        EMABuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_empty_data() {
        let mut builder = EMABuilder::<TestCandle>::new(5);
        let ema = builder.build(&[]);

        assert_eq!(ema.get(), 0.0);
        assert_eq!(ema.period(), 5);
    }

    #[test]
    fn test_ema_display() {
        let ema = EMA {
            period: 5,
            ema: 100.0,
        };

        let display_str = ema.to_string();
        assert!(display_str.contains("EMA"));
        assert!(display_str.contains("5"));
        assert!(display_str.contains("100"));
    }

    #[test]
    fn test_ema_trend() {
        let mut builder = EMABuilder::new(2);

        // 상승 추세 테스트
        let up_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 105.0,
                high: 115.0,
                low: 100.0,
                close: 110.0,
                volume: 1100.0,
            },
        ];

        let ema1 = builder.build(&up_candles);
        let ema2 = builder.next(&up_candles[1]);

        // 상승 추세에서는 EMA 값이 증가해야 함
        assert!(ema2.get() >= ema1.get());
    }
}
