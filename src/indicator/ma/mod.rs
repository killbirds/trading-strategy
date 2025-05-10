pub mod ema;
pub mod sma;
pub mod wma;

use crate::indicator::{TABuilder, TAs, TAsBuilder};
use ema::EMABuilder;
use serde::Deserialize;
use serde::Serialize;
use sma::SMABuilder;
use std::fmt::Debug;
use std::fmt::Display;
use trading_chart::Candle;
use wma::WMABuilder;

/// 이동평균(Moving Average) 인터페이스
///
/// 다양한 이동평균 구현체에 대한 공통 인터페이스
pub trait MA: Display + Send + Debug {
    /// 이동평균 계산 기간
    ///
    /// # Returns
    /// * `usize` - 이동평균 기간
    fn period(&self) -> usize;

    /// 현재 이동평균 값
    ///
    /// # Returns
    /// * `f64` - 계산된 이동평균 값
    fn get(&self) -> f64;
}

/// 이동평균 계산 방식
///
/// 시스템에서 지원하는 이동평균 유형을 정의합니다.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum MAType {
    /// 지수이동평균 (Exponential Moving Average)
    /// 최근 데이터에 더 큰 가중치를 부여합니다.
    EMA,
    /// 단순이동평균 (Simple Moving Average)
    /// 모든 데이터에 동일한 가중치를 부여합니다.
    SMA,
    /// 가중이동평균 (Weighted Moving Average)
    /// 최근 데이터에 선형적으로 증가하는 가중치를 부여합니다.
    WMA,
}

impl Display for MAType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MAType::EMA => write!(f, "EMA"),
            MAType::SMA => write!(f, "SMA"),
            MAType::WMA => write!(f, "WMA"),
        }
    }
}

/// 이동평균 빌더 팩토리
///
/// 지정된 유형의 이동평균 빌더를 생성합니다.
pub struct MABuilderFactory;

impl MABuilderFactory {
    /// 이동평균 유형과 기간에 따른 빌더 생성
    ///
    /// # Arguments
    /// * `ma_type` - 이동평균 유형 (EMA, SMA, WMA)
    /// * `period` - 이동평균 계산 기간
    ///
    /// # Returns
    /// * `Box<dyn TABuilder<Box<dyn MA>>>` - 이동평균 빌더
    pub fn build<C: Candle + 'static>(
        ma_type: &MAType,
        period: usize,
    ) -> Box<dyn TABuilder<Box<dyn MA>, C>> {
        if period == 0 {
            panic!("이동평균 기간은 0보다 커야 합니다");
        }

        match ma_type {
            MAType::EMA => Box::new(EMABuilder::<C>::new(period)),
            MAType::SMA => Box::new(SMABuilder::<C>::new(period)),
            MAType::WMA => Box::new(WMABuilder::<C>::new(period)),
        }
    }
}

/// 여러 기간의 이동평균 컬렉션 타입
pub type MAs = TAs<usize, Box<dyn MA>>;

/// 여러 기간의 이동평균 빌더 타입
pub type MAsBuilder<C> = TAsBuilder<usize, Box<dyn MA>, C>;

/// 이동평균 컬렉션 빌더 팩토리
pub struct MAsBuilderFactory;

impl MAsBuilderFactory {
    /// 여러 기간의 이동평균 빌더 생성
    ///
    /// # Arguments
    /// * `ma_type` - 이동평균 유형 (EMA, SMA, WMA)
    /// * `periods` - 이동평균 계산 기간 목록
    ///
    /// # Returns
    /// * `MAsBuilder` - 여러 기간의 이동평균 빌더
    ///
    /// # Panics
    /// * 빈 기간 목록이 제공되면 패닉 발생
    pub fn build<C: Candle + 'static>(ma_type: &MAType, periods: &[usize]) -> MAsBuilder<C> {
        if periods.is_empty() {
            panic!("이동평균 기간 목록이 비어 있습니다");
        }

        // 기간이 오름차순으로 정렬되어 있는지 확인
        for i in 1..periods.len() {
            if periods[i] <= periods[i - 1] {
                panic!(
                    "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                    periods
                );
            }
        }

        MAsBuilder::new("mas".to_owned(), periods, |period| {
            MABuilderFactory::build::<C>(ma_type, *period)
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
    fn test_ma_trait_implementation() {
        let candles = create_test_candles();

        // EMA 테스트
        let mut ema_builder = EMABuilder::new(2);
        let ema = ema_builder.build(&candles);
        assert_eq!(ema.period(), 2);
        assert!(ema.get() > 0.0);
        assert!(ema.to_string().contains("EMA"));

        // SMA 테스트
        let mut sma_builder = SMABuilder::new(2);
        let sma = sma_builder.build(&candles);
        assert_eq!(sma.period(), 2);
        assert!(sma.get() > 0.0);
        assert!(sma.to_string().contains("SMA"));

        // WMA 테스트
        let mut wma_builder = WMABuilder::new(2);
        let wma = wma_builder.build(&candles);
        assert_eq!(wma.period(), 2);
        assert!(wma.get() > 0.0);
        assert!(wma.to_string().contains("WMA"));
    }

    #[test]
    fn test_ma_type_display() {
        assert_eq!(MAType::EMA.to_string(), "EMA");
        assert_eq!(MAType::SMA.to_string(), "SMA");
        assert_eq!(MAType::WMA.to_string(), "WMA");
    }
}
