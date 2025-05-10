use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use ta_lib::simple_moving_average;
use trading_chart::Candle;

/// 볼린저 밴드 출력값 구조체
#[derive(Clone, Debug)]
struct BollingerBandsOutput {
    average: f64,
    upper: f64,
    lower: f64,
}

/// 볼린저 밴드 계산기
#[derive(Debug)]
struct BollingerBandsIndicator {
    period: usize,
    multiplier: f64,
    values: Vec<f64>,
}

/// 표준편차 계산 함수
fn calculate_standard_deviation(values: &[f64], period: usize) -> f64 {
    if values.len() < period {
        return 0.0;
    }

    let start_idx = values.len() - period;
    let slice = &values[start_idx..];

    // 평균 계산
    let mean = slice.iter().sum::<f64>() / period as f64;

    // 분산 계산
    let variance = slice
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;

    // 표준편차 계산
    variance.sqrt()
}

impl BollingerBandsIndicator {
    fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            values: Vec::with_capacity(period * 2),
        }
    }

    fn next(&mut self, input: &impl Candle) -> BollingerBandsOutput {
        let price = input.close_price();
        self.values.push(price);

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return BollingerBandsOutput {
                average: price,
                upper: price,
                lower: price,
            };
        }

        // ta-lib으로 SMA 계산
        let (sma_result, _) = simple_moving_average(&self.values, Some(self.period)).unwrap();
        let mean = *sma_result.last().unwrap_or(&price);

        // 표준편차 계산
        let std_dev = calculate_standard_deviation(&self.values, self.period);

        // 볼린저 밴드 계산
        let upper = mean + (std_dev * self.multiplier);
        let lower = mean - (std_dev * self.multiplier);

        BollingerBandsOutput {
            average: mean,
            upper,
            lower,
        }
    }
}

/// 볼린저 밴드 계산 빌더
///
/// 볼린저 밴드는 가격의 변동성을 측정하는 기술적 지표로,
/// 이동평균선과 그 주변의 표준편차 기반 밴드로 구성됩니다.
#[derive(Debug)]
pub struct BollingerBandsBuilder<C: Candle> {
    /// 내부 볼린저 밴드 계산 객체
    indicator: BollingerBandsIndicator,
    /// 계산 기간
    period: usize,
    /// 표준편차 승수
    multiplier: f64,
    _phantom: PhantomData<C>,
}

/// 볼린저 밴드 기술적 지표
///
/// 상단, 중간, 하단 밴드로 구성된 볼린저 밴드 값
#[derive(Clone, Debug)]
pub struct BollingerBands {
    /// 내부 볼린저 밴드 계산 결과
    pub middle: f64,
    pub upper: f64,
    pub lower: f64,
    /// 계산 기간
    period: usize,
    /// 표준편차 승수
    multiplier: f64,
}

impl BollingerBands {
    /// 중간 밴드(이동평균) 값 반환
    ///
    /// # Returns
    /// * `f64` - 중간 밴드 값
    pub fn middle(&self) -> f64 {
        self.middle
    }

    /// 상단 밴드 값 반환
    ///
    /// # Returns
    /// * `f64` - 상단 밴드 값
    pub fn upper(&self) -> f64 {
        self.upper
    }

    /// 하단 밴드 값 반환
    ///
    /// # Returns
    /// * `f64` - 하단 밴드 값
    pub fn lower(&self) -> f64 {
        self.lower
    }

    /// 현재 밴드폭 계산
    ///
    /// # Returns
    /// * `f64` - 밴드폭 (상단 - 하단) / 중간
    pub fn bandwidth(&self) -> f64 {
        (self.upper() - self.lower()) / self.middle()
    }

    /// 가격의 상대적 위치 계산 (%B)
    ///
    /// # Arguments
    /// * `price` - 위치를 계산할 가격
    ///
    /// # Returns
    /// * `f64` - 상대적 위치 (0: 하단 밴드, 0.5: 중간 밴드, 1: 상단 밴드)
    pub fn percent_b(&self, price: f64) -> f64 {
        let range = self.upper() - self.lower();
        if range.abs() < f64::EPSILON {
            return 0.5; // 범위가 없는 경우 중간 위치 반환
        }

        (price - self.lower()) / range
    }
}

impl Display for BollingerBands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BB({},{}: {:.2}, {:.2}, {:.2})",
            self.period, self.multiplier, self.middle, self.upper, self.lower
        )
    }
}

impl<C> BollingerBandsBuilder<C>
where
    C: Candle,
{
    /// 새 볼린저 밴드 빌더 생성
    ///
    /// # Arguments
    /// * `period` - 계산 기간 (일반적으로 20)
    /// * `multiplier` - 표준편차 승수 (일반적으로 2.0)
    ///
    /// # Returns
    /// * `BollingerBandsBuilder` - 새 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 매개변수가 제공되면 패닉 발생
    pub fn new(period: usize, multiplier: f64) -> Self {
        if period == 0 {
            panic!("볼린저 밴드 기간은 0보다 커야 합니다");
        }

        if multiplier <= 0.0 {
            panic!("볼린저 밴드 승수는 0보다 커야 합니다");
        }

        let indicator = BollingerBandsIndicator::new(period, multiplier);

        Self {
            indicator,
            period,
            multiplier,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 볼린저 밴드 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `BollingerBands` - 계산된 볼린저 밴드 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> BollingerBands {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 볼린저 밴드 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `BollingerBands` - 계산된 볼린저 밴드 지표
    pub fn build(&mut self, data: &[C]) -> BollingerBands {
        if data.is_empty() {
            // 빈 데이터의 경우 기본값 반환 (모든 밴드가 0인 볼린저 밴드)
            return BollingerBands {
                middle: 0.0,
                upper: 0.0,
                lower: 0.0,
                period: self.period,
                multiplier: self.multiplier,
            };
        }

        let bband = data.iter().fold(
            BollingerBandsOutput {
                average: 0.0,
                upper: 0.0,
                lower: 0.0,
            },
            |_, item| self.indicator.next(item),
        );

        BollingerBands {
            middle: bband.average,
            upper: bband.upper,
            lower: bband.lower,
            period: self.period,
            multiplier: self.multiplier,
        }
    }

    /// 새 캔들 데이터로 볼린저 밴드 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `BollingerBands` - 업데이트된 볼린저 밴드 지표
    pub fn next(&mut self, data: &C) -> BollingerBands {
        let bband = self.indicator.next(data);
        BollingerBands {
            middle: bband.average,
            upper: bband.upper,
            lower: bband.lower,
            period: self.period,
            multiplier: self.multiplier,
        }
    }
}

impl<C> TABuilder<BollingerBands, C> for BollingerBandsBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> BollingerBands {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> BollingerBands {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> BollingerBands {
        self.next(data)
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
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 120.0,
                low: 80.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 130.0,
                low: 90.0,
                close: 120.0,
                volume: 1000.0,
            },
        ]
    }

    #[test]
    fn test_bband_builder_new() {
        let builder = BollingerBandsBuilder::<TestCandle>::new(20, 2.0);
        assert_eq!(builder.period, 20);
        assert_eq!(builder.multiplier, 2.0);
    }

    #[test]
    #[should_panic(expected = "볼린저 밴드 기간은 0보다 커야 합니다")]
    fn test_bband_builder_new_invalid_period() {
        BollingerBandsBuilder::<TestCandle>::new(0, 2.0);
    }

    #[test]
    #[should_panic(expected = "볼린저 밴드 승수는 0보다 커야 합니다")]
    fn test_bband_builder_new_invalid_multiplier() {
        BollingerBandsBuilder::<TestCandle>::new(20, 0.0);
    }

    #[test]
    fn test_bband_build_empty_data() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(20, 2.0);
        let bband = builder.build(&[]);
        assert_eq!(bband.period, 20);
        assert_eq!(bband.multiplier, 2.0);
        assert_eq!(bband.middle, 0.0);
        assert_eq!(bband.upper, 0.0);
        assert_eq!(bband.lower, 0.0);
    }

    #[test]
    fn test_bband_build_with_data() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(3, 2.0);
        let candles = create_test_candles();
        let bband = builder.build(&candles);

        assert_eq!(bband.period, 3);
        assert_eq!(bband.multiplier, 2.0);
        assert!(bband.middle > 0.0);
        assert!(bband.upper > bband.middle);
        assert!(bband.lower < bband.middle);
    }

    #[test]
    fn test_bband_next() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(3, 2.0);
        let candles = create_test_candles();
        let bband = builder.next(&candles[0]);

        assert_eq!(bband.period, 3);
        assert_eq!(bband.multiplier, 2.0);
        assert!(bband.middle > 0.0);
    }

    #[test]
    fn test_bband_bandwidth() {
        let bband = BollingerBands {
            middle: 100.0,
            upper: 110.0,
            lower: 90.0,
            period: 20,
            multiplier: 2.0,
        };

        assert_eq!(bband.bandwidth(), 0.2); // (110 - 90) / 100 = 0.2
    }

    #[test]
    fn test_bband_calculation() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(2, 2.0);
        let candles = create_test_candles();

        // 첫 번째 볼린저 밴드 계산
        let bband1 = builder.next(&candles[0]);
        assert_eq!(bband1.period, 2);
        assert!(bband1.middle > 0.0);
        assert!(bband1.upper >= bband1.middle); // 상단 밴드는 평균보다 크거나 같음
        assert!(bband1.lower <= bband1.middle); // 하단 밴드는 평균보다 작거나 같음

        // 두 번째 볼린저 밴드 계산
        let bband2 = builder.next(&candles[1]);
        assert!(bband2.middle >= bband1.middle); // 상승 추세에서 평균 증가
        assert!(bband2.upper >= bband2.middle); // 상단 밴드는 평균보다 크거나 같음
        assert!(bband2.lower <= bband2.middle); // 하단 밴드는 평균보다 작거나 같음
    }

    #[test]
    fn test_bband_percent_b() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(2, 2.0);
        let candles = create_test_candles();
        let bband = builder.build(&candles);

        // 중간값 테스트
        let middle_pb = bband.percent_b(bband.middle);
        assert!((middle_pb - 0.5).abs() < 0.01); // 중간값은 약 0.5

        // 상단 밴드 테스트
        let upper_pb = bband.percent_b(bband.upper);
        assert!((upper_pb - 1.0).abs() < 0.01); // 상단 밴드는 1.0

        // 하단 밴드 테스트
        let lower_pb = bband.percent_b(bband.lower);
        assert!((lower_pb - 0.0).abs() < 0.01); // 하단 밴드는 0.0

        // 밴드 밖의 값 테스트
        let above_pb = bband.percent_b(bband.upper + 10.0);
        assert!(above_pb > 1.0); // 상단 밴드 위의 값은 1.0보다 큼

        let below_pb = bband.percent_b(bband.lower - 10.0);
        assert!(below_pb < 0.0); // 하단 밴드 아래의 값은 0.0보다 작음
    }

    #[test]
    fn test_bband_volatility() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(2, 2.0);

        // 낮은 변동성 데이터
        let low_vol_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
        ];

        let low_vol = builder.build(&low_vol_candles);
        let low_bandwidth = low_vol.bandwidth();
        assert!(low_bandwidth < 0.1); // 낮은 변동성에서 밴드폭이 작음

        // 높은 변동성 데이터
        let high_vol_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 120.0,
                low: 80.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 130.0,
                low: 90.0,
                close: 120.0,
                volume: 1000.0,
            },
        ];

        let high_vol = builder.build(&high_vol_candles);
        let high_bandwidth = high_vol.bandwidth();
        assert!(high_bandwidth > low_bandwidth); // 높은 변동성에서 밴드폭이 큼
    }
}
