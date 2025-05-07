use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// RSI 계산 함수
fn calculate_rsi(values: &[f64], period: usize) -> f64 {
    if values.len() < period + 1 {
        return 50.0;
    }

    let mut gains = Vec::with_capacity(values.len());
    let mut losses = Vec::with_capacity(values.len());

    // 가격 변화량 계산
    for i in 1..values.len() {
        let change = values[i] - values[i - 1];
        gains.push(if change > 0.0 { change } else { 0.0 });
        losses.push(if change < 0.0 { -change } else { 0.0 });
    }

    // 첫 번째 평균 게인/로스 계산
    let mut avg_gain = gains.iter().take(period).sum::<f64>() / period as f64;
    let mut avg_loss = losses.iter().take(period).sum::<f64>() / period as f64;

    // 나머지 기간에 대해 지수이동평균으로 업데이트
    for i in period..gains.len() {
        let smoothing_factor = 1.0 / period as f64;
        avg_gain = (avg_gain * (1.0 - smoothing_factor)) + (gains[i] * smoothing_factor);
        avg_loss = (avg_loss * (1.0 - smoothing_factor)) + (losses[i] * smoothing_factor);
    }

    // RSI 계산
    if avg_loss < 0.000001 {
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

/// 상대강도지수(RSI) 기술적 지표 빌더
///
/// RSI 지표를 계산하고 업데이트하는 빌더
#[derive(Debug)]
pub struct RSIBuilder<C: Candle> {
    /// RSI 계산 기간
    period: usize,
    /// RSI 계산을 위한 내부 지표 객체
    values: Vec<f64>,
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
            values: Vec::with_capacity(period * 2),
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
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> RSI {
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

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.close_price());
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return RSI {
                period: self.period,
                value: 50.0,
            };
        }

        // RSI 계산
        let rsi = calculate_rsi(&self.values, self.period);

        RSI {
            period: self.period,
            value: rsi,
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
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return RSI {
                period: self.period,
                value: 50.0,
            };
        }

        // RSI 계산
        let rsi = calculate_rsi(&self.values, self.period);

        RSI {
            period: self.period,
            value: rsi,
        }
    }
}

impl<C> TABuilder<RSI, C> for RSIBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> RSI {
        self.from_storage(storage)
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
}
