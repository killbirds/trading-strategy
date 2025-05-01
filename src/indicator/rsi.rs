use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

// RSI 지표 계산을 위한 내부 구현
#[derive(Debug)]
struct RSIIndicator {
    period: usize,
    values: Vec<f64>,
    gains: Vec<f64>,
    losses: Vec<f64>,
}

impl RSIIndicator {
    fn new(period: usize) -> Self {
        Self {
            period,
            values: Vec::with_capacity(period * 2),
            gains: Vec::with_capacity(period),
            losses: Vec::with_capacity(period),
        }
    }

    fn next(&mut self, value: &impl Candle) -> f64 {
        let price = value.close_price();

        // 첫 번째 값이면 이전 값과 비교할 수 없으므로 그냥 저장
        if self.values.is_empty() {
            self.values.push(price);
            return 50.0; // 중립값 반환
        }

        // 가격 변화량 계산
        let prev_price = self.values.last().unwrap();
        let change = price - prev_price;

        // 가격 저장
        self.values.push(price);

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 상승/하락 계산
        if change > 0.0 {
            self.gains.push(change);
            self.losses.push(0.0);
        } else {
            self.gains.push(0.0);
            self.losses.push(-change); // 손실은 양수로 저장
        }

        // 필요한 게인/로스 데이터만 유지
        if self.gains.len() > self.period {
            self.gains.remove(0);
            self.losses.remove(0);
        }

        // 충분한 데이터가 쌓일 때까지는 50 (중립값) 반환
        if self.gains.len() < self.period {
            return 50.0;
        }

        // RSI 계산
        let avg_gain: f64 = self.gains.iter().sum::<f64>() / self.period as f64;
        let avg_loss: f64 = self.losses.iter().sum::<f64>() / self.period as f64;

        if avg_loss < 0.000001 {
            // 분모가 0이면 과매수 상태
            return 100.0;
        }

        let rs = avg_gain / avg_loss;

        100.0 - (100.0 / (1.0 + rs))
    }
}

/// 상대강도지수(RSI) 기술적 지표 빌더
///
/// RSI 지표를 계산하고 업데이트하는 빌더
#[derive(Debug)]
pub struct RSIBuilder<C: Candle> {
    /// RSI 계산 기간
    period: usize,
    /// RSI 계산을 위한 내부 지표 객체
    indicator: RSIIndicator,
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
    pub rsi: f64,
}

impl Display for RSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RSI({}: {:.2})", self.period, self.rsi)
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
        self.rsi >= threshold_value
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
        self.rsi <= threshold_value
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
        self.rsi >= lower && self.rsi <= upper
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
        self.rsi
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
    pub fn new(period: usize) -> RSIBuilder<C> {
        if period == 0 {
            panic!("RSI 기간은 0보다 커야 합니다");
        }

        let indicator = RSIIndicator::new(period);

        RSIBuilder {
            period,
            indicator,
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
        self.build(&storage.get_reversed_items())
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
                rsi: 50.0, // 기본값으로 중립값 반환
            };
        }

        let rsi: f64 = data.iter().fold(0.0, |_, item| self.indicator.next(item));
        RSI {
            period: self.period,
            rsi,
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
        let rsi: f64 = self.indicator.next(data);
        RSI {
            period: self.period,
            rsi,
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

    #[test]
    fn test_rsi_is_overbought() {
        let rsi = RSI {
            period: 14,
            rsi: 75.0,
        };

        assert!(rsi.is_overbought(None));
        assert!(rsi.is_overbought(Some(70.0)));
        assert!(!rsi.is_overbought(Some(80.0)));
    }

    #[test]
    fn test_rsi_is_oversold() {
        let rsi = RSI {
            period: 14,
            rsi: 25.0,
        };

        assert!(rsi.is_oversold(None));
        assert!(rsi.is_oversold(Some(30.0)));
        assert!(!rsi.is_oversold(Some(20.0)));
    }

    #[test]
    fn test_rsi_is_within_range() {
        let rsi = RSI {
            period: 14,
            rsi: 45.0,
        };

        assert!(rsi.is_within_range(40.0, 60.0));
        assert!(!rsi.is_within_range(50.0, 70.0));
        assert!(!rsi.is_within_range(20.0, 40.0));
    }
}
