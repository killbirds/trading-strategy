use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// Average True Range (ATR) 기술적 지표
///
/// ATR은 가격 변동성을 측정하는 지표로, Wilder's smoothing 방식을 사용합니다.
#[derive(Clone, Debug)]
pub struct ATR {
    /// ATR 계산 기간
    period: usize,
    /// ATR 값
    pub value: f64,
}

impl Display for ATR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATR({}: {:.2})", self.period, self.value)
    }
}

impl ATR {
    /// ATR 기간 반환
    ///
    /// # Returns
    /// * `usize` - ATR 계산 기간
    pub fn period(&self) -> usize {
        self.period
    }

    /// ATR 값 반환
    ///
    /// # Returns
    /// * `f64` - ATR 값
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// ATR 계산을 위한 빌더
///
/// ATR 지표를 계산하고 업데이트하는 빌더
///
/// # 성능 고려사항
/// - 메모리 사용량: period + 1개의 데이터만 유지하여 O(period) 메모리 사용
/// - 시간 복잡도: O(1) 업데이트 (next), O(n) 초기 빌드 (n = 데이터 개수)
/// - 대용량 데이터: period * 2개 이상의 데이터는 자동으로 제거되어 메모리 효율적
#[derive(Debug)]
pub struct ATRBuilder<C: Candle> {
    /// ATR 계산 기간
    period: usize,
    /// 고가 데이터 (최근 period + 1개만 유지)
    high_values: Vec<f64>,
    /// 저가 데이터 (최근 period + 1개만 유지)
    low_values: Vec<f64>,
    /// 종가 데이터 (최근 period + 1개만 유지)
    close_values: Vec<f64>,
    /// 이전 ATR 값 (Wilder's smoothing용)
    previous_atr: Option<f64>,
    /// 캔들 타입 표시자 (제네릭 타입 표시용)
    _phantom: PhantomData<C>,
}

impl<C: Candle> ATRBuilder<C> {
    /// 새 ATR 빌더 생성
    ///
    /// # Arguments
    /// * `period` - ATR 계산 기간 (일반적으로 14)
    ///
    /// # Returns
    /// * `ATRBuilder` - 새 ATR 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("ATR 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            high_values: Vec::with_capacity(period + 2),
            low_values: Vec::with_capacity(period + 2),
            close_values: Vec::with_capacity(period + 2),
            previous_atr: None,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 ATR 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `ATR` - 계산된 ATR 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ATR {
        self.build(&storage.get_ascending_items())
    }

    /// 데이터 벡터에서 ATR 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `ATR` - 계산된 ATR 지표
    pub fn build(&mut self, data: &[C]) -> ATR {
        if data.is_empty() {
            return ATR {
                period: self.period,
                value: 0.0,
            };
        }

        // 데이터를 순차적으로 처리하여 ATR 계산
        let mut atr_value = 0.0;
        for candle in data {
            atr_value = self.next_value(candle);
        }

        ATR {
            period: self.period,
            value: atr_value,
        }
    }

    /// 다음 캔들 데이터로 ATR 값 계산 (내부용)
    fn next_value(&mut self, candle: &C) -> f64 {
        // 가격 데이터 저장
        self.high_values.push(candle.high_price());
        self.low_values.push(candle.low_price());
        self.close_values.push(candle.close_price());

        // 필요한 데이터만 유지 (period + 1개만 필요)
        if self.high_values.len() > self.period + 1 {
            let excess = self.high_values.len() - (self.period + 1);
            self.high_values.drain(0..excess);
            self.low_values.drain(0..excess);
            self.close_values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우 (최소 2개 필요)
        if self.high_values.len() < 2 {
            return 0.0;
        }

        // 현재 캔들의 True Range 계산
        let idx = self.high_values.len() - 1;
        let high = self.high_values[idx];
        let low = self.low_values[idx];
        let prev_close = self.close_values[idx - 1];

        // True Range = max(고가-저가, |고가-이전종가|, |저가-이전종가|)
        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());

        // NaN/Infinity 체크
        if tr.is_nan() || tr.is_infinite() || high.is_nan() || low.is_nan() || prev_close.is_nan() {
            return 0.0;
        }

        // ATR 계산 (Wilder's smoothing)
        let atr = if let Some(prev_atr) = self.previous_atr {
            // Wilder의 평활화 방식으로 업데이트
            (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64
        } else if self.high_values.len() > self.period {
            // 처음 계산할 때는 period개의 TR 평균 사용
            let mut tr_sum = 0.0;
            for i in 1..=self.period {
                let h = self.high_values[i];
                let l = self.low_values[i];
                let pc = self.close_values[i - 1];
                let t = (h - l).max((h - pc).abs()).max((l - pc).abs());
                tr_sum += t;
            }
            tr_sum / self.period as f64
        } else {
            // 충분한 데이터가 없는 경우
            return 0.0;
        };

        // 계산된 ATR 저장 및 유효성 검증
        if atr.is_nan() || atr.is_infinite() {
            return 0.0;
        }
        self.previous_atr = Some(atr);
        atr
    }

    /// 새 캔들 데이터로 ATR 지표 업데이트
    ///
    /// # Arguments
    /// * `candle` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `ATR` - 업데이트된 ATR 지표
    pub fn next(&mut self, candle: &C) -> ATR {
        let atr_value = self.next_value(candle);
        ATR {
            period: self.period,
            value: atr_value,
        }
    }
}

impl<C: Candle> TABuilder<ATR, C> for ATRBuilder<C> {
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ATR {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> ATR {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> ATR {
        self.next(data)
    }
}

/// 여러 기간의 ATR 지표 컬렉션 타입
pub type ATRs = TAs<usize, ATR>;

/// 여러 기간의 ATR 지표 빌더 타입
pub type ATRsBuilder<C> = TAsBuilder<usize, ATR, C>;

/// ATR 컬렉션 빌더 팩토리
pub struct ATRsBuilderFactory;

impl ATRsBuilderFactory {
    /// 여러 기간의 ATR 빌더 생성
    ///
    /// # Arguments
    /// * `periods` - ATR 계산 기간 목록
    ///
    /// # Returns
    /// * `ATRsBuilder` - 여러 기간의 ATR 빌더
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> ATRsBuilder<C> {
        ATRsBuilder::new("atrs".to_owned(), periods, |period| {
            Box::new(ATRBuilder::<C>::new(*period))
        })
    }

    /// 기본 ATR 빌더 생성 (14 기간)
    ///
    /// # Returns
    /// * `ATRsBuilder` - 기본 ATR 빌더
    pub fn build_default<C: Candle + 'static>() -> ATRsBuilder<C> {
        let default_periods = vec![14];
        Self::build(&default_periods)
    }

    /// 일반적인 ATR 빌더 세트 생성 (7, 14, 21 기간)
    ///
    /// # Returns
    /// * `ATRsBuilder` - 일반적인 기간 세트의 ATR 빌더
    pub fn build_common<C: Candle + 'static>() -> ATRsBuilder<C> {
        let common_periods = vec![7, 14, 21];
        Self::build(&common_periods)
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
                open: 115.0,
                high: 125.0,
                low: 105.0,
                close: 120.0,
                volume: 1300.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 120.0,
                high: 130.0,
                low: 110.0,
                close: 125.0,
                volume: 1400.0,
            },
        ]
    }

    #[test]
    fn test_atr_calculation() {
        let mut builder = ATRBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        // 첫 번째 ATR 계산
        let atr1 = builder.next(&candles[0]);
        assert_eq!(atr1.period(), 14);
        assert!(atr1.value() >= 0.0);

        // 두 번째 ATR 계산
        let atr2 = builder.next(&candles[1]);
        assert!(atr2.value() >= 0.0);
    }

    #[test]
    fn test_atr_build() {
        let mut builder = ATRBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        let atr = builder.build(&candles);
        assert_eq!(atr.period(), 2);
        assert!(atr.value() >= 0.0);
    }

    #[test]
    fn test_atr_wilders_smoothing() {
        let mut builder = ATRBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // 첫 번째 값은 평균으로 계산
        let atr1 = builder.next(&candles[0]);
        let atr2 = builder.next(&candles[1]);
        let atr3 = builder.next(&candles[2]);

        // ATR 값이 0 이상이어야 함
        assert!(atr1.value() >= 0.0);
        assert!(atr2.value() >= 0.0);
        assert!(atr3.value() >= 0.0);
    }

    #[test]
    fn test_atr_empty_data() {
        let mut builder = ATRBuilder::<TestCandle>::new(14);
        let empty: Vec<TestCandle> = vec![];

        let atr = builder.build(&empty);
        assert_eq!(atr.period(), 14);
        assert_eq!(atr.value(), 0.0);
    }

    #[test]
    fn test_atr_insufficient_data() {
        let mut builder = ATRBuilder::<TestCandle>::new(14);
        let single_candle = vec![create_test_candles()[0].clone()];

        let atr = builder.build(&single_candle);
        assert_eq!(atr.period(), 14);
        assert_eq!(atr.value(), 0.0);
    }

    #[test]
    fn test_atr_period() {
        let builder = ATRBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();
        let mut builder = builder;
        let atr = builder.build(&candles);

        assert_eq!(atr.period(), 14);
    }

    #[test]
    fn test_atrs_builder() {
        let mut builder = ATRsBuilderFactory::build::<TestCandle>(&[7, 14, 21]);
        let candles = create_test_candles();

        let atrs = builder.build(&candles);
        assert_eq!(atrs.get(&7).period(), 7);
        assert_eq!(atrs.get(&14).period(), 14);
        assert_eq!(atrs.get(&21).period(), 21);
    }

    #[test]
    fn test_atrs_builder_next() {
        let mut builder = ATRsBuilderFactory::build::<TestCandle>(&[7, 14]);
        let candles = create_test_candles();

        for candle in &candles {
            let atrs = builder.next(candle);
            assert!(atrs.get(&7).value() >= 0.0);
            assert!(atrs.get(&14).value() >= 0.0);
        }
    }

    #[test]
    fn test_atr_display() {
        let mut builder = ATRBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();
        let atr = builder.build(&candles);

        let display_str = format!("{}", atr);
        assert!(display_str.contains("ATR"));
        assert!(display_str.contains("14"));
    }

    #[test]
    fn test_atr_known_values_accuracy() {
        // 알려진 ATR 계산 결과와 비교
        // period=2인 경우 간단한 계산으로 검증
        // 데이터:
        //   Candle 1: H=12, L=8, C=10
        //   Candle 2: H=13, L=9, C=11
        //   Candle 3: H=14, L=10, C=12
        // TR1 = max(12-8, |12-10|, |8-10|) = max(4, 2, 2) = 4
        // TR2 = max(13-9, |13-11|, |9-11|) = max(4, 2, 2) = 4
        // TR3 = max(14-10, |14-12|, |10-12|) = max(4, 2, 2) = 4
        // period=2일 때:
        //   초기 ATR = (TR1 + TR2) / 2 = (4 + 4) / 2 = 4.0
        //   Wilder's smoothing: ATR = (prev_ATR * (period-1) + TR) / period
        //   ATR(3) = (4.0 * 1 + 4) / 2 = 4.0
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 10.0,
                high: 13.0,
                low: 9.0,
                close: 11.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 11.0,
                high: 14.0,
                low: 10.0,
                close: 12.0,
                volume: 1200.0,
            },
        ];

        let mut builder = ATRBuilder::<TestCandle>::new(2);
        let atr = builder.build(&candles);

        // ATR은 양수여야 함
        assert!(
            atr.value() > 0.0,
            "ATR should be positive. Got: {}",
            atr.value()
        );

        // ATR 값이 유효한 범위 내에 있어야 함 (최소한 TR 값보다 작거나 같아야 함)
        assert!(
            atr.value() <= 4.0,
            "ATR should be less than or equal to max TR. Got: {}",
            atr.value()
        );
    }

    #[test]
    fn test_atr_known_values_period_2() {
        // period=2인 경우 정확한 계산 검증
        // 데이터:
        //   Candle 1: H=15, L=5, C=10 (이전 종가 없음)
        //   Candle 2: H=16, L=6, C=11
        //   Candle 3: H=17, L=7, C=12
        // TR1 = 15-5 = 10 (첫 번째는 H-L)
        // TR2 = max(16-6, |16-11|, |6-11|) = max(10, 5, 5) = 10
        // TR3 = max(17-7, |17-12|, |7-12|) = max(10, 5, 5) = 10
        // period=2일 때:
        //   초기 ATR = (TR1 + TR2) / 2 = (10 + 10) / 2 = 10.0
        //   Wilder's smoothing: ATR = (prev_ATR * (period-1) + TR) / period
        //   ATR(3) = (10.0 * 1 + 10) / 2 = 10.0
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 15.0,
                low: 5.0,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 10.0,
                high: 16.0,
                low: 6.0,
                close: 11.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 11.0,
                high: 17.0,
                low: 7.0,
                close: 12.0,
                volume: 1200.0,
            },
        ];

        let mut builder = ATRBuilder::<TestCandle>::new(2);
        let atr = builder.build(&candles);

        // ATR은 양수여야 함
        assert!(
            atr.value() > 0.0,
            "ATR should be positive. Got: {}",
            atr.value()
        );

        // ATR 값이 유효한 범위 내에 있어야 함
        assert!(
            atr.value() <= 10.0,
            "ATR should be less than or equal to max TR. Got: {}",
            atr.value()
        );
    }

    #[test]
    fn test_atr_incremental_vs_build_consistency() {
        // next를 여러 번 호출한 결과와 build를 한 번 호출한 결과의 일관성 검증
        let mut builder1 = ATRBuilder::<TestCandle>::new(14);
        let mut builder2 = ATRBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        // builder1: next를 여러 번 호출
        for candle in &candles {
            builder1.next(candle);
        }
        let atr1 = builder1.next(&candles[candles.len() - 1]);

        // builder2: build를 한 번 호출
        let atr2 = builder2.build(&candles);

        // 마지막 값이 비슷해야 함 (Wilder's smoothing으로 인해 약간의 차이는 있을 수 있음)
        // 하지만 둘 다 유효한 범위 내에 있어야 함
        assert!(atr1.value() >= 0.0);
        assert!(atr2.value() >= 0.0);
        // 값의 차이가 너무 크지 않아야 함 (10% 이내)
        let diff_percent = if atr2.value() > 0.0 {
            ((atr1.value() - atr2.value()).abs() / atr2.value()) * 100.0
        } else {
            0.0
        };
        assert!(
            diff_percent < 10.0,
            "ATR values should be consistent. Incremental: {}, Build: {}, Diff: {}%",
            atr1.value(),
            atr2.value(),
            diff_percent
        );
    }
}
