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
        self.build(&storage.get_reversed_items())
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
    use chrono::Utc;
    use trading_chart::{CandleInterval, OhlcvCandle};

    fn create_test_candles() -> Vec<OhlcvCandle> {
        vec![
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 100.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                quote_volume: 1000.0,
                volume: 1000.0,
                trade_count: None,
            },
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 110.0,
                high: 125.0,
                low: 105.0,
                close: 120.0,
                quote_volume: 1100.0,
                volume: 1100.0,
                trade_count: None,
            },
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 120.0,
                high: 125.0,
                low: 110.0,
                close: 115.0,
                quote_volume: 1200.0,
                volume: 1200.0,
                trade_count: None,
            },
        ]
    }

    #[test]
    fn test_ema_calculation() {
        let candles = create_test_candles();
        let mut builder = EMABuilder::new(2);

        let ema = builder.build(&candles);
        assert!(ema.get() > 0.0); // 실제 값은 구현에 따라 다를 수 있음

        let new_candle = OhlcvCandle {
            symbol: "TEST".to_string(),
            interval: CandleInterval::Minute1,
            open_time: Utc::now(),
            close_time: Utc::now(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            quote_volume: 1300.0,
            volume: 1300.0,
            trade_count: None,
        };

        let updated_ema = builder.next(&new_candle);
        assert!(updated_ema.get() > ema.get()); // 상승 추세이므로 EMA도 증가해야 함
    }

    #[test]
    #[should_panic(expected = "EMA 기간은 0보다 커야 합니다")]
    fn test_invalid_period() {
        EMABuilder::<OhlcvCandle>::new(0); // 0 기간은 유효하지 않음
    }

    #[test]
    fn test_empty_data() {
        let mut builder = EMABuilder::<OhlcvCandle>::new(5);
        let ema = builder.build(&[]);

        assert_eq!(ema.get(), 0.0); // 비어있는 데이터는 0 반환
        assert_eq!(ema.period(), 5); // 기간은 유지
    }
}
