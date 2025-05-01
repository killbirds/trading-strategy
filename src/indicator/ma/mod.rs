pub mod ema;
pub mod sma;
pub mod wma;

use crate::indicator::{TABuilder, TAs, TAsBuilder};
use ema::EMABuilder;
use serde::Deserialize;
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
#[derive(Deserialize, Debug, Clone, Copy)]
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
