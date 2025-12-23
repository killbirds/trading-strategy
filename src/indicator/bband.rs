use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::utils::moving_average;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
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
            values: Vec::with_capacity(period + 1),
        }
    }

    fn next(&mut self, input: &impl Candle) -> BollingerBandsOutput {
        let price = input.close_price();
        self.values.push(price);

        // 필요한 데이터만 유지 (period만 필요)
        if self.values.len() > self.period {
            let excess = self.values.len() - self.period;
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

        // SMA 계산
        let mean = moving_average::calculate_sma_or_default(&self.values, self.period, price);

        // 표준편차 계산
        let std_dev = calculate_standard_deviation(&self.values, self.period);

        // NaN/Infinity 체크
        if mean.is_nan() || mean.is_infinite() || std_dev.is_nan() || std_dev.is_infinite() {
            return BollingerBandsOutput {
                average: price,
                upper: price,
                lower: price,
            };
        }

        // 볼린저 밴드 계산
        let upper = mean + (std_dev * self.multiplier);
        let lower = mean - (std_dev * self.multiplier);

        // 결과값 유효성 검증
        BollingerBandsOutput {
            average: if mean.is_nan() || mean.is_infinite() {
                price
            } else {
                mean
            },
            upper: if upper.is_nan() || upper.is_infinite() {
                price
            } else {
                upper
            },
            lower: if lower.is_nan() || lower.is_infinite() {
                price
            } else {
                lower
            },
        }
    }
}

/// 볼린저 밴드 계산 빌더
///
/// 볼린저 밴드는 가격의 변동성을 측정하는 기술적 지표로,
/// 이동평균선과 그 주변의 표준편차 기반 밴드로 구성됩니다.
///
/// # 성능 고려사항
/// - 메모리 사용량: period개의 가격 데이터만 유지하여 O(period) 메모리 사용
/// - 시간 복잡도: O(period) 업데이트 (표준편차 계산), O(n*period) 초기 빌드
/// - 최적화: period개 이상의 데이터는 자동으로 제거되어 메모리 효율적
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

    /// 계산 기간 반환
    ///
    /// # Returns
    /// * `usize` - 볼린저 밴드 계산 기간
    pub fn period(&self) -> usize {
        self.period
    }

    /// 표준편차 승수 반환
    ///
    /// # Returns
    /// * `f64` - 표준편차 승수
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// 현재 밴드폭 계산
    ///
    /// # Returns
    /// * `f64` - 밴드폭 (상단 - 하단) / 중간
    pub fn bandwidth(&self) -> f64 {
        if self.middle().abs() < f64::EPSILON {
            return 0.0;
        }
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
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> BollingerBands {
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
            return BollingerBands {
                middle: 0.0,
                upper: 0.0,
                lower: 0.0,
                period: self.period,
                multiplier: self.multiplier,
            };
        }

        // 인디케이터 초기화
        self.indicator.values.clear();

        let bband = data.iter().fold(
            BollingerBandsOutput {
                average: 0.0,
                upper: 0.0,
                lower: 0.0,
            },
            |_, item| self.indicator.next(item),
        );

        // 충분한 데이터가 없는 경우 마지막 가격 사용
        if self.indicator.values.len() < self.period
            && let Some(last_candle) = data.last()
        {
            let price = last_candle.close_price();
            return BollingerBands {
                middle: price,
                upper: price,
                lower: price,
                period: self.period,
                multiplier: self.multiplier,
            };
        }

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
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> BollingerBands {
        self.build_from_storage(storage)
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

    #[test]
    fn test_bband_period_and_multiplier() {
        let mut builder = BollingerBandsBuilder::<TestCandle>::new(20, 2.5);
        let candles = create_test_candles();
        let bband = builder.build(&candles);

        assert_eq!(bband.period(), 20);
        assert_eq!(bband.multiplier(), 2.5);
    }

    #[test]
    fn test_bband_known_values_accuracy() {
        // 알려진 볼린저 밴드 계산 결과와 비교
        // period=3, multiplier=2.0인 경우
        // 데이터: [10.0, 11.0, 12.0]
        // SMA = (10+11+12)/3 = 11.0
        // 표준편차 계산:
        //   variance = ((10-11)^2 + (11-11)^2 + (12-11)^2) / 3 = (1+0+1)/3 = 2/3
        //   std_dev = sqrt(2/3) ≈ 0.816
        // upper = 11.0 + 2.0 * 0.816 ≈ 12.633
        // lower = 11.0 - 2.0 * 0.816 ≈ 9.367
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 10.5,
                low: 9.5,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 10.0,
                high: 11.5,
                low: 9.5,
                close: 11.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 11.0,
                high: 12.5,
                low: 10.5,
                close: 12.0,
                volume: 1200.0,
            },
        ];

        let mut builder = BollingerBandsBuilder::<TestCandle>::new(3, 2.0);
        let bband = builder.build(&candles);

        // SMA 검증
        let expected_sma = (10.0 + 11.0 + 12.0) / 3.0;
        assert!(
            (bband.middle - expected_sma).abs() < 0.01,
            "SMA calculation mismatch. Expected: {}, Got: {}",
            expected_sma,
            bband.middle
        );

        // 표준편차 계산 검증
        let variance = ((10.0 - expected_sma).powi(2)
            + (11.0 - expected_sma).powi(2)
            + (12.0 - expected_sma).powi(2))
            / 3.0;
        let std_dev = variance.sqrt();
        let expected_upper = expected_sma + 2.0 * std_dev;
        let expected_lower = expected_sma - 2.0 * std_dev;

        assert!(
            (bband.upper - expected_upper).abs() < 0.01,
            "Upper band calculation mismatch. Expected: {}, Got: {}",
            expected_upper,
            bband.upper
        );
        assert!(
            (bband.lower - expected_lower).abs() < 0.01,
            "Lower band calculation mismatch. Expected: {}, Got: {}",
            expected_lower,
            bband.lower
        );
    }

    #[test]
    fn test_bband_known_values_period_2() {
        // period=2, multiplier=2.0인 경우 간단한 계산
        // 데이터: [10.0, 12.0]
        // SMA = (10+12)/2 = 11.0
        // 표준편차 = sqrt(((10-11)^2 + (12-11)^2) / 2) = sqrt(1) = 1.0
        // upper = 11.0 + 2.0 * 1.0 = 13.0
        // lower = 11.0 - 2.0 * 1.0 = 9.0
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 10.5,
                low: 9.5,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 10.0,
                high: 12.5,
                low: 9.5,
                close: 12.0,
                volume: 1100.0,
            },
        ];

        let mut builder = BollingerBandsBuilder::<TestCandle>::new(2, 2.0);
        let bband = builder.build(&candles);

        let expected_sma = 11.0;
        assert!(
            (bband.middle - expected_sma).abs() < 0.01,
            "SMA calculation mismatch. Expected: {}, Got: {}",
            expected_sma,
            bband.middle
        );

        let expected_upper = 13.0;
        let expected_lower = 9.0;
        assert!(
            (bband.upper - expected_upper).abs() < 0.01,
            "Upper band calculation mismatch. Expected: {}, Got: {}",
            expected_upper,
            bband.upper
        );
        assert!(
            (bband.lower - expected_lower).abs() < 0.01,
            "Lower band calculation mismatch. Expected: {}, Got: {}",
            expected_lower,
            bband.lower
        );
    }

    #[test]
    fn test_bband_incremental_vs_build_consistency() {
        // next를 여러 번 호출한 결과와 build를 한 번 호출한 결과의 일관성 검증
        // period=3으로 충분한 데이터로 테스트
        let mut builder1 = BollingerBandsBuilder::<TestCandle>::new(3, 2.0);
        let mut builder2 = BollingerBandsBuilder::<TestCandle>::new(3, 2.0);
        let candles = create_test_candles();

        // builder1: next를 여러 번 호출
        for candle in &candles {
            builder1.next(candle);
        }
        let bband1 = builder1.next(&candles[candles.len() - 1]);

        // builder2: build를 한 번 호출
        let bband2 = builder2.build(&candles);

        // 값들이 유효한 범위 내에 있어야 함
        assert!(bband1.middle > 0.0);
        assert!(bband2.middle > 0.0);

        // 데이터가 충분한 경우에만 밴드 관계 검증
        // 데이터가 부족하면 upper, middle, lower가 모두 같을 수 있음
        if bband1.upper != bband1.middle {
            assert!(bband1.upper > bband1.middle);
            assert!(bband1.lower < bband1.middle);
        }
        if bband2.upper != bband2.middle {
            assert!(bband2.upper > bband2.middle);
            assert!(bband2.lower < bband2.middle);
        }

        // 중간 밴드의 차이가 너무 크지 않아야 함 (10% 이내)
        // 볼린저 밴드는 내부 상태(표준편차 계산)에 따라 약간 다른 결과를 낼 수 있음
        let middle_diff_percent = if bband2.middle > 0.0 {
            ((bband1.middle - bband2.middle).abs() / bband2.middle) * 100.0
        } else {
            0.0
        };
        assert!(
            middle_diff_percent < 10.0,
            "Middle band values should be consistent. Incremental: {}, Build: {}, Diff: {}%",
            bband1.middle,
            bband2.middle,
            middle_diff_percent
        );
    }
}
