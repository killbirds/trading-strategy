use serde::Serialize;
use std::fmt::Debug;

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
