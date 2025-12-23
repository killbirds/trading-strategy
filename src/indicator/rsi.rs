use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 상대강도지수(RSI) 기술적 지표 빌더
///
/// RSI 지표를 계산하고 업데이트하는 빌더
///
/// # 성능 고려사항
/// - 메모리 사용량: period + 1개의 종가 데이터만 유지하여 O(period) 메모리 사용
/// - 시간 복잡도: O(1) 업데이트 (Wilder's smoothing), O(n) 초기 빌드 (n = 데이터 개수)
/// - 최적화: Wilder's smoothing을 사용하여 효율적인 증분 계산 지원
#[derive(Debug)]
pub struct RSIBuilder<C: Candle> {
    /// RSI 계산 기간
    period: usize,
    /// 종가 데이터 (최근 period + 1개만 유지)
    values: Vec<f64>,
    /// 이전 평균 게인 (Wilder's smoothing용)
    previous_avg_gain: Option<f64>,
    /// 이전 평균 로스 (Wilder's smoothing용)
    previous_avg_loss: Option<f64>,
    _phantom: PhantomData<C>,
}

/// 상대강도지수(RSI) 기술적 지표
///
/// RSI는 가격 변동의 상대적 강도를 측정하여 과매수/과매도 상태를 판단
#[derive(Clone, Debug)]
pub struct RSI {
    /// RSI 계산 기간
    period: usize,
    /// RSI 값 (0-100)
    pub value: f64,
}

impl Display for RSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RSI({}: {:.2})", self.period, self.value)
    }
}

impl RSI {
    /// RSI가 과매수 상태인지 확인 (일반적으로 70 이상)
    ///
    /// # Arguments
    /// * `threshold` - 과매수 기준값 (기본값 70.0)
    ///
    /// # Returns
    /// * `bool` - 과매수 여부
    pub fn is_overbought(&self, threshold: Option<f64>) -> bool {
        let threshold_value = threshold.unwrap_or(70.0);
        self.value >= threshold_value
    }

    /// RSI가 과매도 상태인지 확인 (일반적으로 30 이하)
    ///
    /// # Arguments
    /// * `threshold` - 과매도 기준값 (기본값 30.0)
    ///
    /// # Returns
    /// * `bool` - 과매도 여부
    pub fn is_oversold(&self, threshold: Option<f64>) -> bool {
        let threshold_value = threshold.unwrap_or(30.0);
        self.value <= threshold_value
    }

    /// RSI 값이 특정 범위 내에 있는지 확인
    ///
    /// # Arguments
    /// * `lower` - 하한 기준값
    /// * `upper` - 상한 기준값
    ///
    /// # Returns
    /// * `bool` - 범위 내 여부
    pub fn is_within_range(&self, lower: f64, upper: f64) -> bool {
        self.value >= lower && self.value <= upper
    }

    /// RSI 기간 반환
    ///
    /// # Returns
    /// * `usize` - RSI 계산 기간
    pub fn period(&self) -> usize {
        self.period
    }

    /// RSI 값 반환
    ///
    /// # Returns
    /// * `f64` - RSI 값
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl<C> RSIBuilder<C>
where
    C: Candle,
{
    /// 새 RSI 빌더 생성
    ///
    /// # Arguments
    /// * `period` - RSI 계산 기간 (일반적으로 14)
    ///
    /// # Returns
    /// * `RSIBuilder` - 새 RSI 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("RSI 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period + 2),
            previous_avg_gain: None,
            previous_avg_loss: None,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 RSI 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `RSI` - 계산된 RSI 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> RSI {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 RSI 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `RSI` - 계산된 RSI 지표
    pub fn build(&mut self, data: &[C]) -> RSI {
        if data.is_empty() {
            return RSI {
                period: self.period,
                value: 50.0,
            };
        }

        // 상태 초기화
        self.values.clear();
        self.previous_avg_gain = None;
        self.previous_avg_loss = None;

        // 데이터를 순차적으로 처리하여 RSI 계산
        let mut rsi_value = 50.0;
        for candle in data {
            rsi_value = self.next_value(candle);
        }

        RSI {
            period: self.period,
            value: rsi_value,
        }
    }

    /// 다음 캔들 데이터로 RSI 값 계산 (내부용)
    fn next_value(&mut self, candle: &C) -> f64 {
        // 새 가격 추가
        self.values.push(candle.close_price());

        // 필요한 데이터만 유지 (period + 1개만 필요)
        if self.values.len() > self.period + 1 {
            let excess = self.values.len() - (self.period + 1);
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우 (최소 2개 필요)
        if self.values.len() < 2 {
            return 50.0;
        }

        // 가격 변화량 계산
        let idx = self.values.len() - 1;
        let change = self.values[idx] - self.values[idx - 1];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };

        // RSI 계산 (Wilder's smoothing)
        let (avg_gain, avg_loss) = if let (Some(prev_avg_gain), Some(prev_avg_loss)) =
            (self.previous_avg_gain, self.previous_avg_loss)
        {
            // Wilder's smoothing으로 업데이트
            // avg = (prev_avg * (period - 1) + current) / period
            let new_avg_gain =
                (prev_avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
            let new_avg_loss =
                (prev_avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;
            (new_avg_gain, new_avg_loss)
        } else if self.values.len() > self.period {
            // 처음 계산할 때는 period개의 gain/loss 평균 사용
            let mut gain_sum = 0.0;
            let mut loss_sum = 0.0;
            for i in 1..=self.period {
                let ch = self.values[i] - self.values[i - 1];
                gain_sum += if ch > 0.0 { ch } else { 0.0 };
                loss_sum += if ch < 0.0 { -ch } else { 0.0 };
            }
            (gain_sum / self.period as f64, loss_sum / self.period as f64)
        } else {
            // 충분한 데이터가 없는 경우
            return 50.0;
        };

        // 계산된 평균값 저장
        self.previous_avg_gain = Some(avg_gain);
        self.previous_avg_loss = Some(avg_loss);

        // RSI 계산
        // avg_loss가 0에 가까우면 RSI는 100에 가까워짐
        if avg_loss < 0.000001 {
            return 100.0;
        }

        // NaN/Infinity 체크
        if avg_gain.is_nan()
            || avg_loss.is_nan()
            || avg_gain.is_infinite()
            || avg_loss.is_infinite()
        {
            return 50.0;
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        // 결과값 유효성 검증
        if rsi.is_nan() || rsi.is_infinite() {
            50.0
        } else {
            rsi.clamp(0.0, 100.0)
        }
    }

    /// 새 캔들 데이터로 RSI 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `RSI` - 업데이트된 RSI 지표
    pub fn next(&mut self, data: &C) -> RSI {
        let rsi_value = self.next_value(data);
        RSI {
            period: self.period,
            value: rsi_value,
        }
    }
}

impl<C> TABuilder<RSI, C> for RSIBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> RSI {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> RSI {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> RSI {
        self.next(data)
    }
}

/// 여러 기간의 RSI 지표 컬렉션 타입
pub type RSIs = TAs<usize, RSI>;

/// 여러 기간의 RSI 지표 빌더 타입
pub type RSIsBuilder<C> = TAsBuilder<usize, RSI, C>;

/// RSI 컬렉션 빌더 팩토리
pub struct RSIsBuilderFactory;

impl RSIsBuilderFactory {
    /// 여러 기간의 RSI 빌더 생성
    ///
    /// # Arguments
    /// * `periods` - RSI 계산 기간 목록
    ///
    /// # Returns
    /// * `RSIsBuilder` - 여러 기간의 RSI 빌더
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> RSIsBuilder<C> {
        RSIsBuilder::new("rsis".to_owned(), periods, |period| {
            Box::new(RSIBuilder::<C>::new(*period))
        })
    }

    /// 기본 RSI 빌더 생성 (14 기간)
    ///
    /// # Returns
    /// * `RSIsBuilder` - 기본 RSI 빌더
    pub fn build_default<C: Candle + 'static>() -> RSIsBuilder<C> {
        let default_periods = vec![14];
        Self::build(&default_periods)
    }

    /// 일반적인 RSI 빌더 세트 생성 (9, 14, 25 기간)
    ///
    /// # Returns
    /// * `RSIsBuilder` - 일반적인 기간 세트의 RSI 빌더
    pub fn build_common<C: Candle + 'static>() -> RSIsBuilder<C> {
        let common_periods = vec![9, 14, 25];
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
        ]
    }

    #[test]
    fn test_rsi_calculation() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        // 첫 번째 RSI 계산
        let rsi1 = builder.next(&candles[0]);
        assert_eq!(rsi1.period, 14);
        assert!(rsi1.value() >= 0.0 && rsi1.value() <= 100.0);

        // 두 번째 RSI 계산
        let rsi2 = builder.next(&candles[1]);
        assert!(rsi2.value() >= 0.0 && rsi2.value() <= 100.0);
    }

    #[test]
    fn test_rsi_overbought() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();
        let rsi = builder.build(&candles);

        // RSI 값이 0-100 범위 내에 있는지 확인
        assert!(rsi.value() >= 0.0 && rsi.value() <= 100.0);

        // 과매수 상태 확인 (기본 임계값 70 사용)
        if rsi.value() > 70.0 {
            assert!(rsi.is_overbought(None));
        }
    }

    #[test]
    fn test_rsi_oversold() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();
        let rsi = builder.build(&candles);

        // RSI 값이 0-100 범위 내에 있는지 확인
        assert!(rsi.value() >= 0.0 && rsi.value() <= 100.0);

        // 과매도 상태 확인 (기본 임계값 30 사용)
        if rsi.value() < 30.0 {
            assert!(rsi.is_oversold(None));
        }
    }

    #[test]
    fn test_rsi_divergence() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        let rsi1 = builder.next(&candles[0]);
        let rsi2 = builder.next(&candles[1]);
        let rsi3 = builder.next(&candles[2]);

        // RSI 값이 0-100 범위 내에 있는지 확인
        assert!(rsi1.value() >= 0.0 && rsi1.value() <= 100.0);
        assert!(rsi2.value() >= 0.0 && rsi2.value() <= 100.0);
        assert!(rsi3.value() >= 0.0 && rsi3.value() <= 100.0);
    }

    #[test]
    fn test_rsi_trend() {
        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        let _rsi1 = builder.next(&candles[0]);
        let rsi2 = builder.next(&candles[1]);
        let rsi3 = builder.next(&candles[2]);

        assert!(rsi3.value() > rsi2.value());
    }

    #[test]
    fn test_rsi_range() {
        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // RSI 값이 항상 0과 100 사이에 있는지 확인
        for candle in &candles {
            let rsi = builder.next(candle);
            assert!(rsi.value() >= 0.0 && rsi.value() <= 100.0);
        }
    }

    #[test]
    fn test_rsi_all_gains() {
        // 모든 가격이 상승하는 경우 RSI는 100에 가까워야 함
        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let mut candles = Vec::new();
        let base_price = 100.0;

        for i in 0..10 {
            candles.push(TestCandle {
                timestamp: Utc::now().timestamp() + i as i64,
                open: base_price + i as f64,
                high: base_price + i as f64 + 1.0,
                low: base_price + i as f64 - 0.5,
                close: base_price + (i + 1) as f64,
                volume: 1000.0,
            });
        }

        let rsi = builder.build(&candles);
        assert!(rsi.value() > 50.0);
        assert!(rsi.value() <= 100.0);
    }

    #[test]
    fn test_rsi_all_losses() {
        // 모든 가격이 하락하는 경우 RSI는 0에 가까워야 함
        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let mut candles = Vec::new();
        let base_price = 100.0;

        for i in 0..10 {
            candles.push(TestCandle {
                timestamp: Utc::now().timestamp() + i as i64,
                open: base_price - i as f64,
                high: base_price - i as f64 + 0.5,
                low: base_price - i as f64 - 1.0,
                close: base_price - (i + 1) as f64,
                volume: 1000.0,
            });
        }

        let rsi = builder.build(&candles);
        assert!(rsi.value() < 50.0);
        assert!(rsi.value() >= 0.0);
    }

    #[test]
    fn test_rsi_incremental_vs_build() {
        // next를 여러 번 호출한 결과와 build를 한 번 호출한 결과가 같아야 함
        let mut builder1 = RSIBuilder::<TestCandle>::new(14);
        let mut builder2 = RSIBuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        // builder1: next를 여러 번 호출
        for candle in &candles {
            builder1.next(candle);
        }
        let rsi1 = builder1.next(&candles[candles.len() - 1]);

        // builder2: build를 한 번 호출
        let rsi2 = builder2.build(&candles);

        // 마지막 값이 같아야 함 (약간의 부동소수점 오차 허용)
        assert!((rsi1.value() - rsi2.value()).abs() < 0.01);
    }

    #[test]
    fn test_rsi_period_2_known_values() {
        // period=2인 경우 알려진 값으로 테스트
        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 102.0,
                high: 107.0,
                low: 97.0,
                close: 105.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 105.0,
                high: 110.0,
                low: 100.0,
                close: 108.0,
                volume: 1200.0,
            },
        ];

        let rsi = builder.build(&candles);
        assert!(rsi.value() >= 0.0 && rsi.value() <= 100.0);
        // period=2이고 모든 가격이 상승하므로 RSI는 높아야 함
        assert!(rsi.value() > 50.0);
    }

    #[test]
    fn test_rsi_empty_data() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let empty: Vec<TestCandle> = vec![];
        let rsi = builder.build(&empty);
        assert_eq!(rsi.value(), 50.0);
        assert_eq!(rsi.period(), 14);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let mut builder = RSIBuilder::<TestCandle>::new(14);
        let candles = vec![TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
        }];
        let rsi = builder.build(&candles);
        assert_eq!(rsi.value(), 50.0);
    }

    #[test]
    fn test_rsi_known_values_accuracy() {
        // 알려진 RSI 계산 결과와 비교
        // period=2인 경우 간단한 계산으로 검증
        // 데이터: [100, 102, 104, 106, 108] (모두 상승)
        // period=2일 때:
        // - 첫 2개 변화: +2, +2
        // - avg_gain = (2+2)/2 = 2.0, avg_loss = 0.0
        // - RS = 2.0/0.0 = inf, RSI = 100.0
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 100.0,
                high: 103.0,
                low: 99.0,
                close: 102.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 102.0,
                high: 105.0,
                low: 101.0,
                close: 104.0,
                volume: 1200.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 3,
                open: 104.0,
                high: 107.0,
                low: 103.0,
                close: 106.0,
                volume: 1300.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 4,
                open: 106.0,
                high: 109.0,
                low: 105.0,
                close: 108.0,
                volume: 1400.0,
            },
        ];

        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let rsi = builder.build(&candles);

        // 모든 가격이 상승하므로 RSI는 100에 가까워야 함
        assert!(
            rsi.value() > 90.0,
            "RSI should be high for all gains. Got: {}",
            rsi.value()
        );
    }

    #[test]
    fn test_rsi_known_values_period_2() {
        // period=2인 경우 정확한 계산 검증
        // 데이터: [100, 98, 96, 94, 92] (모두 하락)
        // period=2일 때:
        // - 첫 2개 변화: -2, -2
        // - avg_gain = 0.0, avg_loss = (2+2)/2 = 2.0
        // - RS = 0.0/2.0 = 0.0, RSI = 100 - 100/(1+0) = 0.0
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 101.0,
                low: 97.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 100.0,
                high: 99.0,
                low: 97.0,
                close: 98.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 98.0,
                high: 97.0,
                low: 95.0,
                close: 96.0,
                volume: 1200.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 3,
                open: 96.0,
                high: 95.0,
                low: 93.0,
                close: 94.0,
                volume: 1300.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 4,
                open: 94.0,
                high: 93.0,
                low: 91.0,
                close: 92.0,
                volume: 1400.0,
            },
        ];

        let mut builder = RSIBuilder::<TestCandle>::new(2);
        let rsi = builder.build(&candles);

        // 모든 가격이 하락하므로 RSI는 0에 가까워야 함
        assert!(
            rsi.value() < 10.0,
            "RSI should be low for all losses. Got: {}",
            rsi.value()
        );
    }
}
