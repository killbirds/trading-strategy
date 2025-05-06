use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
struct AverageDirectionalMovementIndex {
    period: usize,
    values: Vec<f64>,
    high_values: Vec<f64>,
    low_values: Vec<f64>,
    close_values: Vec<f64>,
}

impl AverageDirectionalMovementIndex {
    fn new(period: usize) -> Self {
        if period == 0 {
            panic!("ADX 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period * 2),
            high_values: Vec::with_capacity(period * 2),
            low_values: Vec::with_capacity(period * 2),
            close_values: Vec::with_capacity(period * 2),
        }
    }

    fn next(&mut self, input: &impl Candle) -> (f64, f64, f64) {
        // 가격 데이터 저장
        self.high_values.push(input.high_price());
        self.low_values.push(input.low_price());
        self.close_values.push(input.close_price());

        // 필요한 데이터만 유지
        if self.high_values.len() > self.period * 2 {
            self.high_values.remove(0);
            self.low_values.remove(0);
            self.close_values.remove(0);
        }

        // 충분한 데이터가 없는 경우
        if self.high_values.len() < 2 {
            return (0.0, 0.0, 0.0);
        }

        let mut tr_sum = 0.0;
        let mut plus_dm_sum = 0.0;
        let mut minus_dm_sum = 0.0;

        // TR과 DM 계산
        for i in 1..self.high_values.len() {
            let high = self.high_values[i];
            let low = self.low_values[i];
            let prev_high = self.high_values[i - 1];
            let prev_low = self.low_values[i - 1];
            let prev_close = self.close_values[i - 1];

            // True Range 계산
            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            tr_sum += tr;

            // Directional Movement 계산
            let up_move = high - prev_high;
            let down_move = prev_low - low;

            if up_move > down_move && up_move > 0.0 {
                plus_dm_sum += up_move;
            }
            if down_move > up_move && down_move > 0.0 {
                minus_dm_sum += down_move;
            }
        }

        // 평균값 계산
        let tr_avg = tr_sum / (self.high_values.len() - 1) as f64;
        let plus_dm_avg = plus_dm_sum / (self.high_values.len() - 1) as f64;
        let minus_dm_avg = minus_dm_sum / (self.high_values.len() - 1) as f64;

        // DI 계산
        let plus_di = if tr_avg > 0.0 {
            (plus_dm_avg / tr_avg) * 100.0
        } else {
            0.0
        };
        let minus_di = if tr_avg > 0.0 {
            (minus_dm_avg / tr_avg) * 100.0
        } else {
            0.0
        };

        // DX 계산
        let dx = if (plus_di + minus_di) > 0.0 {
            ((plus_di - minus_di).abs() / (plus_di + minus_di)) * 100.0
        } else {
            0.0
        };

        // ADX는 DX의 지수이동평균
        let adx_multiplier = 2.0 / (self.period as f64 + 1.0);
        self.values.push(dx);
        if self.values.len() > self.period {
            self.values.remove(0);
        }

        let mut adx = if !self.values.is_empty() {
            self.values[0]
        } else {
            0.0
        };

        for &dx_value in self.values.iter().skip(1) {
            adx = (dx_value - adx) * adx_multiplier + adx;
        }

        (adx, plus_di, minus_di)
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
    pub plus_di: f64,
    pub minus_di: f64,
}

impl Display for ADX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ADX({}: {}, +DI: {}, -DI: {})",
            self.period, self.adx, self.plus_di, self.minus_di
        )
    }
}

impl<C> ADXBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> Self {
        ADXBuilder {
            period,
            indicator: AverageDirectionalMovementIndex::new(period),
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> ADX {
        self.build(&storage.get_time_ordered_items())
    }

    pub fn build(&mut self, data: &[C]) -> ADX {
        let mut adx = 0.0;
        let mut plus_di = 0.0;
        let mut minus_di = 0.0;

        // 디버그 로깅
        println!("Building ADX with {} data points", data.len());

        for item in data {
            (adx, plus_di, minus_di) = self.indicator.next(item);
        }

        ADX {
            period: self.period,
            adx,
            plus_di,
            minus_di,
        }
    }

    pub fn next(&mut self, data: &C) -> ADX {
        let (adx, plus_di, minus_di) = self.indicator.next(data);
        ADX {
            period: self.period,
            adx,
            plus_di,
            minus_di,
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
    fn test_adx_calculation() {
        let mut builder = ADXBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // 첫 번째 ADX 계산
        let adx1 = builder.next(&candles[0]);
        assert_eq!(adx1.period, 2);
        assert!(adx1.adx >= 0.0 && adx1.adx <= 100.0);
        assert!(adx1.plus_di >= 0.0 && adx1.plus_di <= 100.0);
        assert!(adx1.minus_di >= 0.0 && adx1.minus_di <= 100.0);

        // 두 번째 ADX 계산
        let adx2 = builder.next(&candles[1]);
        assert!(adx2.adx >= 0.0 && adx2.adx <= 100.0);

        // 세 번째 ADX 계산
        let adx3 = builder.next(&candles[2]);
        assert!(adx3.adx >= 0.0 && adx3.adx <= 100.0);
    }

    #[test]
    fn test_adx_trend_strength() {
        let mut builder = ADXBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // ADX 값 범위 검증
        let adx = builder.build(&candles);
        assert!(adx.adx >= 0.0 && adx.adx <= 100.0); // ADX는 0-100 범위 내
        assert!(adx.plus_di >= 0.0 && adx.plus_di <= 100.0); // +DI는 0-100 범위 내
        assert!(adx.minus_di >= 0.0 && adx.minus_di <= 100.0); // -DI는 0-100 범위 내
    }

    #[test]
    fn test_adx_directional_movement() {
        let mut builder = ADXBuilder::<TestCandle>::new(2);

        // 상승 추세 데이터
        let up_candles = vec![
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

        let up_trend = builder.build(&up_candles);
        assert!(up_trend.adx > 0.0);

        // 하락 추세 데이터
        let down_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 120.0,
                high: 120.0,
                low: 110.0,
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

        let down_trend = builder.build(&down_candles);
        assert!(down_trend.adx > 0.0);

        // ADX는 방향과 관계없이 추세의 강도를 측정
        assert!(up_trend.adx > 0.0 && down_trend.adx > 0.0);
    }
}
