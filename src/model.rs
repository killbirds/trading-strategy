use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::Debug;

/// 가격 표현을 위한 뉴타입
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Price {
    value: f64,
}

impl Price {
    /// 새 가격 인스턴스 생성
    pub fn new(value: f64) -> Self {
        Price { value }
    }

    /// 가격 값 반환
    pub fn value(&self) -> f64 {
        self.value
    }
}

// 산술 연산자 구현
impl std::ops::Add<f64> for Price {
    type Output = Price;

    fn add(self, rhs: f64) -> Self::Output {
        Price::new(self.value + rhs)
    }
}

impl std::ops::Sub<f64> for Price {
    type Output = Price;

    fn sub(self, rhs: f64) -> Self::Output {
        Price::new(self.value - rhs)
    }
}

impl std::ops::Mul<f64> for Price {
    type Output = Price;

    fn mul(self, rhs: f64) -> Self::Output {
        Price::new(self.value * rhs)
    }
}

impl std::ops::Div<f64> for Price {
    type Output = Price;

    fn div(self, rhs: f64) -> Self::Output {
        Price::new(self.value / rhs)
    }
}

// f64와의 비교 연산자 구현
impl PartialEq<f64> for Price {
    fn eq(&self, other: &f64) -> bool {
        self.value == *other
    }
}

impl PartialOrd<f64> for Price {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

/// 리스크 관리용 포지션 유형 (어댑터)
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    /// 롱 포지션
    Long,
    /// 숏 포지션
    Short,
}

/// 트레이딩 신호를 나타내는 열거형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Signal {
    /// 매수 신호
    Enter,
    /// 매도 신호
    Exit,
    /// 홀딩 (포지션 유지) 신호
    Hold,
}

/// 트레이딩 포지션의 보유 상태를 나타내는 구조체
///
/// 이 구조체는 포지션 유형, 진입 시간, 가격, 수량 및 관련 전략 정보를 포함합니다.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TradePosition {
    pub datetime: DateTime<Utc>,
    pub price: f64,
    pub quantity: f64,
    pub market: String,
    pub position_type: PositionType,
    pub stop_loss: Option<f64>,
}

impl TradePosition {
    /// 새로운 TradePosition 인스턴스를 생성합니다.
    ///
    /// # 인자
    /// * `position` - 포지션 유형 (Long 또는 Short)
    /// * `datetime` - 진입 시간
    /// * `price` - 진입 가격
    /// * `quantity` - 수량
    /// * `market` - 시장 식별자
    /// * `strategy_type` - 사용된 전략 유형 (선택 사항)
    pub fn new(datetime: DateTime<Utc>, price: f64, quantity: f64, market: String) -> Self {
        // 가격과 수량이 양수인지 확인
        assert!(price > 0.0, "가격은 양수여야 합니다");
        assert!(quantity > 0.0, "수량은 양수여야 합니다");

        Self {
            datetime,
            price,
            quantity,
            market,
            position_type: PositionType::Long, // 기본값은 롱 포지션
            stop_loss: None,
        }
    }

    /// 포지션 유형 설정
    pub fn with_position_type(mut self, position_type: PositionType) -> Self {
        self.position_type = position_type;
        self
    }

    /// 손절가 설정
    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// 총 포지션 가치를 계산합니다 (가격 * 수량).
    pub fn total_price(&self) -> f64 {
        self.price * self.quantity
    }

    /// 현재 거래 가격에 대한 수익률을 계산합니다.
    ///
    /// # 인자
    /// * `trade_price` - 현재 거래 가격
    pub fn rate_of_return(&self, trade_price: f64) -> f64 {
        (trade_price - self.price) / self.price
    }

    /// 현재 거래 가격에 대한 수익 금액을 계산합니다.
    ///
    /// # 인자
    /// * `trade_price` - 현재 거래 가격
    pub fn returns(&self, trade_price: f64) -> f64 {
        (trade_price - self.price) * self.quantity
    }
}

impl Default for TradePosition {
    /// 기본 TradePosition 인스턴스를 생성합니다.
    /// 기본값으로 현재 시간, 1.0의 가격과 수량, "default" 시장, Long 포지션을 사용합니다.
    fn default() -> Self {
        Self {
            datetime: Utc::now(),
            price: 1.0,
            quantity: 1.0,
            market: "default".to_string(),
            position_type: PositionType::Long,
            stop_loss: None,
        }
    }
}
