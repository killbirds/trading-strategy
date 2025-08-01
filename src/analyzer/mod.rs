// 기술적 지표 분석기 모듈
// 다양한 트레이딩 지표를 분석하고 패턴을 식별하는 도구를 제공합니다.
//
// # 주요 기능
// - 다양한 기술적 지표 기반 분석기 제공 (MACD, RSI, 이동평균, 볼린저 밴드 등)
// - 공통 인터페이스를 통한 일관된 API
// - 캔들 데이터 기반 분석
// - 매수/매도 신호 감지 및 강도 계산
// - 거래 패턴 인식
//
// # 주요 컴포넌트
// - base: 모든 분석기의 기본 트레이트 및 기능 정의
// - 개별 분석기들: 각 기술적 지표별 구현체
// - hybrid_analyzer: 여러 지표를 결합한 고급 분석기

pub mod adx_analyzer;
pub mod atr_analyzer;
pub mod base;
pub mod bband_analyzer;
pub mod candle_pattern_analyzer;
pub mod hybrid_analyzer;
pub mod ichimoku_analyzer;
pub mod ma_analyzer;
pub mod macd_analyzer;
pub mod market_structure_analyzer;
pub mod momentum_analyzer;
pub mod price_action_analyzer;
pub mod risk_management_analyzer;
pub mod rsi_analyzer;
pub mod signal_strength_analyzer;
pub mod supertrend_analyzer;
pub mod support_resistance_analyzer;
pub mod three_rsi_analyzer;
pub mod volume_analyzer;
pub mod vwap_analyzer;

pub use adx_analyzer::{ADXAnalyzer, ADXAnalyzerData};
pub use atr_analyzer::{ATRAnalyzer, ATRAnalyzerData};
pub use base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
pub use bband_analyzer::{BBandAnalyzer, BBandAnalyzerData};
pub use candle_pattern_analyzer::{
    CandlePatternAnalyzer, CandlePatternAnalyzerData, MultiCandlePattern, PatternAnalysis,
    PatternReliability, PatternSignal, SingleCandlePattern,
};
pub use hybrid_analyzer::{HybridAnalyzer, HybridAnalyzerData};
pub use ichimoku_analyzer::{IchimokuAnalyzer, IchimokuAnalyzerData};
pub use ma_analyzer::{MAAnalyzer, MAAnalyzerData};
pub use macd_analyzer::{MACDAnalyzer, MACDAnalyzerData};
pub use market_structure_analyzer::{
    FVGType, FairValueGap, LiquidityPool, LiquidityPoolType, MarketStructure,
    MarketStructureAnalyzer, MarketStructureAnalyzerData, OrderBlock, OrderBlockType,
    StructureChange,
};
pub use momentum_analyzer::{
    DivergenceType, MomentumAnalysis, MomentumAnalyzer, MomentumAnalyzerData, MomentumDirection,
    MomentumDivergence, MomentumIndicators, MomentumState, OverBoughtOverSold,
};
pub use price_action_analyzer::{
    CandlePattern, PriceActionAnalyzer, PriceActionAnalyzerData, PriceTrend, SwingPoint, SwingType,
};
pub use risk_management_analyzer::{
    PositionSizingMethod, PositionType, RiskCalculation, RiskLevel, RiskManagementAnalyzer,
    RiskManagementAnalyzerData,
};
pub use rsi_analyzer::{RSIAnalyzer, RSIAnalyzerData};
pub use signal_strength_analyzer::{
    AnalyzerWeights, SignalAnalysis, SignalDirection, SignalQuality, SignalStrengthAnalyzer,
    SignalStrengthAnalyzerData, SignalStrengthLevel,
};
pub use supertrend_analyzer::{SuperTrendAnalyzer, SuperTrendAnalyzerData};
pub use support_resistance_analyzer::{
    LevelType, SupportResistanceAnalyzer, SupportResistanceAnalyzerData, SupportResistanceLevel,
};
pub use three_rsi_analyzer::{ThreeRSIAnalyzer, ThreeRSIAnalyzerData};
pub use volume_analyzer::{VolumeAnalyzer, VolumeAnalyzerData};
pub use vwap_analyzer::{VWAPAnalyzer, VWAPAnalyzerData};
