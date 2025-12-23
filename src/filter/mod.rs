use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use trading_chart::Candle;

// 각 필터 모듈 가져오기
mod adx;
mod atr;
mod bollinger_band;
mod candle_pattern;
mod copys;
mod ichimoku;
mod macd;
mod momentum;
mod moving_average;
mod rsi;
mod supertrend;
mod support_resistance;
mod three_rsi;
mod volume;
mod vwap;

/// 필터 공통 유틸리티 함수
pub mod utils {
    use crate::candle_store::CandleStore;
    use trading_chart::Candle;

    /// 캔들 데이터로 CandleStore 생성 (공통 유틸리티)
    pub fn create_candle_store<C: Candle + 'static>(candles: &[C]) -> CandleStore<C> {
        let candles_vec = candles.to_vec();
        CandleStore::new(candles_vec, candles.len() * 2, false)
    }

    /// 기본 파라미터 검증 (period > 0)
    pub fn validate_period(period: usize, param_name: &str) -> anyhow::Result<()> {
        if period == 0 {
            return Err(anyhow::anyhow!(
                "{} 파라미터 오류: period는 0보다 커야 합니다",
                param_name
            ));
        }
        Ok(())
    }

    /// 경계 조건 체크 (캔들 데이터 부족 확인)
    pub fn check_sufficient_candles(
        candles_len: usize,
        required_length: usize,
        coin: &str,
    ) -> bool {
        if candles_len < required_length {
            log::debug!(
                "코인 {} 캔들 데이터 부족: {} < {}",
                coin,
                candles_len,
                required_length
            );
            return false;
        }
        true
    }
}

/// 기술적 필터링 기준
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TechnicalFilterType {
    /// RSI 기반 필터 (과매수/과매도)
    RSI,
    /// MACD 기반 필터 (추세)
    MACD,
    /// 볼린저밴드 기반 필터 (변동성)
    BollingerBand,
    /// ADX 기반 필터 (추세 강도)
    ADX,
    /// 이동평균선 기반 필터 (추세)
    MovingAverage,
    /// 이치모쿠 기반 필터 (추세/지지/저항)
    Ichimoku,
    /// VWAP 기반 필터 (가격/거래량)
    VWAP,
    /// CopyS 기반 필터 (복합 전략)
    Copys,
    /// ATR 기반 필터 (변동성)
    ATR,
    /// SuperTrend 기반 필터 (추세)
    SuperTrend,
    /// Volume 기반 필터 (거래량)
    Volume,
    /// ThreeRSI 기반 필터 (3개 RSI 조합)
    ThreeRSI,
    /// CandlePattern 기반 필터 (캔들 패턴)
    CandlePattern,
    /// SupportResistance 기반 필터 (지지/저항)
    SupportResistance,
    /// Momentum 기반 필터 (모멘텀)
    Momentum,
}

impl fmt::Display for TechnicalFilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TechnicalFilterType::RSI => write!(f, "RSI"),
            TechnicalFilterType::MACD => write!(f, "MACD"),
            TechnicalFilterType::BollingerBand => write!(f, "볼린저밴드"),
            TechnicalFilterType::ADX => write!(f, "ADX"),
            TechnicalFilterType::MovingAverage => write!(f, "이동평균선"),
            TechnicalFilterType::Ichimoku => write!(f, "이치모쿠"),
            TechnicalFilterType::VWAP => write!(f, "VWAP"),
            TechnicalFilterType::Copys => write!(f, "COPYS"),
            TechnicalFilterType::ATR => write!(f, "ATR"),
            TechnicalFilterType::SuperTrend => write!(f, "SuperTrend"),
            TechnicalFilterType::Volume => write!(f, "Volume"),
            TechnicalFilterType::ThreeRSI => write!(f, "ThreeRSI"),
            TechnicalFilterType::CandlePattern => write!(f, "CandlePattern"),
            TechnicalFilterType::SupportResistance => write!(f, "SupportResistance"),
            TechnicalFilterType::Momentum => write!(f, "Momentum"),
        }
    }
}

impl FromStr for TechnicalFilterType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RSI" => Ok(TechnicalFilterType::RSI),
            "MACD" => Ok(TechnicalFilterType::MACD),
            "BOLLINGERBAND" | "BOLLINGER_BAND" | "BB" => Ok(TechnicalFilterType::BollingerBand),
            "ADX" => Ok(TechnicalFilterType::ADX),
            "MOVINGAVERAGE" | "MOVING_AVERAGE" | "MA" => Ok(TechnicalFilterType::MovingAverage),
            "ICHIMOKU" => Ok(TechnicalFilterType::Ichimoku),
            "VWAP" => Ok(TechnicalFilterType::VWAP),
            "COPYS" => Ok(TechnicalFilterType::Copys),
            "ATR" => Ok(TechnicalFilterType::ATR),
            "SUPERTREND" => Ok(TechnicalFilterType::SuperTrend),
            "VOLUME" => Ok(TechnicalFilterType::Volume),
            "THREERSI" => Ok(TechnicalFilterType::ThreeRSI),
            "CANDLEPATTERN" => Ok(TechnicalFilterType::CandlePattern),
            "SUPPORTRESISTANCE" => Ok(TechnicalFilterType::SupportResistance),
            "MOMENTUM" => Ok(TechnicalFilterType::Momentum),
            _ => Err(anyhow::anyhow!("알 수 없는 필터 타입: {}", s)),
        }
    }
}

/// RSI 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum RSIFilterType {
    Overbought = 0,
    Oversold = 1,
    NormalRange = 2,
    CrossAboveThreshold = 3,
    CrossBelowThreshold = 4,
    CrossAbove50 = 5,
    CrossBelow50 = 6,
    RisingTrend = 7,
    FallingTrend = 8,
    CrossAbove40 = 9,
    CrossBelow60 = 10,
    CrossAbove20 = 11,
    CrossBelow80 = 12,
    Sideways = 13,
    StrongRisingMomentum = 14,
    StrongFallingMomentum = 15,
    NeutralRange = 16,
    Above40 = 17,
    Below60 = 18,
    Above50 = 19,
    Below50 = 20,
    Divergence = 21,
    Convergence = 22,
    Stable = 25,
    NeutralTrend = 28,
    Bullish = 29,
    Bearish = 30,
}

impl From<i32> for RSIFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => RSIFilterType::Overbought,
            1 => RSIFilterType::Oversold,
            2 => RSIFilterType::NormalRange,
            3 => RSIFilterType::CrossAboveThreshold,
            4 => RSIFilterType::CrossBelowThreshold,
            5 => RSIFilterType::CrossAbove50,
            6 => RSIFilterType::CrossBelow50,
            7 => RSIFilterType::RisingTrend,
            8 => RSIFilterType::FallingTrend,
            9 => RSIFilterType::CrossAbove40,
            10 => RSIFilterType::CrossBelow60,
            11 => RSIFilterType::CrossAbove20,
            12 => RSIFilterType::CrossBelow80,
            13 => RSIFilterType::Sideways,
            14 => RSIFilterType::StrongRisingMomentum,
            15 => RSIFilterType::StrongFallingMomentum,
            16 => RSIFilterType::NeutralRange,
            17 => RSIFilterType::Above40,
            18 => RSIFilterType::Below60,
            19 => RSIFilterType::Above50,
            20 => RSIFilterType::Below50,
            21 => RSIFilterType::Divergence,
            22 => RSIFilterType::Convergence,
            25 => RSIFilterType::Stable,
            28 => RSIFilterType::NeutralTrend,
            29 => RSIFilterType::Bullish,
            30 => RSIFilterType::Bearish,
            _ => RSIFilterType::Overbought,
        }
    }
}

impl From<RSIFilterType> for i32 {
    fn from(value: RSIFilterType) -> Self {
        value as i32
    }
}

/// MACD 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MACDFilterType {
    MacdAboveSignal = 0,
    MacdBelowSignal = 1,
    SignalCrossAbove = 2,
    SignalCrossBelow = 3,
    HistogramAboveThreshold = 4,
    HistogramBelowThreshold = 5,
    ZeroLineCrossAbove = 6,
    ZeroLineCrossBelow = 7,
    HistogramNegativeTurn = 8,
    HistogramPositiveTurn = 9,
    StrongUptrend = 10,
    StrongDowntrend = 11,
    MacdRising = 12,
    MacdFalling = 13,
    HistogramExpanding = 14,
    HistogramContracting = 15,
    Divergence = 16,
    Convergence = 17,
    Overbought = 18,
    Oversold = 19,
    Sideways = 20,
}

impl From<i32> for MACDFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => MACDFilterType::MacdAboveSignal,
            1 => MACDFilterType::MacdBelowSignal,
            2 => MACDFilterType::SignalCrossAbove,
            3 => MACDFilterType::SignalCrossBelow,
            4 => MACDFilterType::HistogramAboveThreshold,
            5 => MACDFilterType::HistogramBelowThreshold,
            6 => MACDFilterType::ZeroLineCrossAbove,
            7 => MACDFilterType::ZeroLineCrossBelow,
            8 => MACDFilterType::HistogramNegativeTurn,
            9 => MACDFilterType::HistogramPositiveTurn,
            10 => MACDFilterType::StrongUptrend,
            11 => MACDFilterType::StrongDowntrend,
            12 => MACDFilterType::MacdRising,
            13 => MACDFilterType::MacdFalling,
            14 => MACDFilterType::HistogramExpanding,
            15 => MACDFilterType::HistogramContracting,
            16 => MACDFilterType::Divergence,
            17 => MACDFilterType::Convergence,
            18 => MACDFilterType::Overbought,
            19 => MACDFilterType::Oversold,
            20 => MACDFilterType::Sideways,
            _ => MACDFilterType::MacdAboveSignal,
        }
    }
}

impl From<MACDFilterType> for i32 {
    fn from(value: MACDFilterType) -> Self {
        value as i32
    }
}

/// 볼린저 밴드 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum BollingerBandFilterType {
    AboveUpperBand = 0,
    BelowLowerBand = 1,
    InsideBand = 2,
    OutsideBand = 3,
    AboveMiddleBand = 4,
    BelowMiddleBand = 5,
    BandWidthSufficient = 6,
    BreakThroughLowerBand = 7,
    SqueezeBreakout = 8,
    EnhancedSqueezeBreakout = 9,
    SqueezeState = 10,
    BandWidthNarrowing = 11,
    SqueezeExpansionStart = 12,
    BreakThroughUpperBand = 13,
    BreakThroughLowerBandFromBelow = 14,
    BandWidthExpanding = 15,
    MiddleBandSideways = 16,
    UpperBandSideways = 17,
    LowerBandSideways = 18,
    BandWidthSideways = 19,
    UpperBandTouch = 20,
    LowerBandTouch = 21,
    BandWidthThresholdBreakthrough = 22,
    PriceMovingToUpperFromMiddle = 23,
    PriceMovingToLowerFromMiddle = 24,
    BandConvergenceThenDivergence = 25,
    BandDivergenceThenConvergence = 26,
    PriceMovingToUpperWithinBand = 27,
    PriceMovingToLowerWithinBand = 28,
    LowVolatility = 29,
    HighVolatility = 30,
}

impl From<i32> for BollingerBandFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => BollingerBandFilterType::AboveUpperBand,
            1 => BollingerBandFilterType::BelowLowerBand,
            2 => BollingerBandFilterType::InsideBand,
            3 => BollingerBandFilterType::OutsideBand,
            4 => BollingerBandFilterType::AboveMiddleBand,
            5 => BollingerBandFilterType::BelowMiddleBand,
            6 => BollingerBandFilterType::BandWidthSufficient,
            7 => BollingerBandFilterType::BreakThroughLowerBand,
            8 => BollingerBandFilterType::SqueezeBreakout,
            9 => BollingerBandFilterType::EnhancedSqueezeBreakout,
            10 => BollingerBandFilterType::SqueezeState,
            11 => BollingerBandFilterType::BandWidthNarrowing,
            12 => BollingerBandFilterType::SqueezeExpansionStart,
            13 => BollingerBandFilterType::BreakThroughUpperBand,
            14 => BollingerBandFilterType::BreakThroughLowerBandFromBelow,
            15 => BollingerBandFilterType::BandWidthExpanding,
            16 => BollingerBandFilterType::MiddleBandSideways,
            17 => BollingerBandFilterType::UpperBandSideways,
            18 => BollingerBandFilterType::LowerBandSideways,
            19 => BollingerBandFilterType::BandWidthSideways,
            20 => BollingerBandFilterType::UpperBandTouch,
            21 => BollingerBandFilterType::LowerBandTouch,
            22 => BollingerBandFilterType::BandWidthThresholdBreakthrough,
            23 => BollingerBandFilterType::PriceMovingToUpperFromMiddle,
            24 => BollingerBandFilterType::PriceMovingToLowerFromMiddle,
            25 => BollingerBandFilterType::BandConvergenceThenDivergence,
            26 => BollingerBandFilterType::BandDivergenceThenConvergence,
            27 => BollingerBandFilterType::PriceMovingToUpperWithinBand,
            28 => BollingerBandFilterType::PriceMovingToLowerWithinBand,
            29 => BollingerBandFilterType::LowVolatility,
            30 => BollingerBandFilterType::HighVolatility,
            _ => BollingerBandFilterType::AboveUpperBand,
        }
    }
}

impl From<BollingerBandFilterType> for i32 {
    fn from(value: BollingerBandFilterType) -> Self {
        value as i32
    }
}

/// ADX 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum ADXFilterType {
    BelowThreshold = 0,
    AboveThreshold = 1,
    PDIAboveMDI = 2,
    MDIAbovePDI = 3,
    StrongUptrend = 4,
    StrongDowntrend = 5,
    ADXRising = 6,
    ADXFalling = 7,
    DIGapExpanding = 8,
    DIGapContracting = 9,
    ExtremeHigh = 10,
    ExtremeLow = 11,
    MiddleLevel = 12,
    PDICrossAboveMDI = 13,
    MDICrossAbovePDI = 14,
    Sideways = 15,
    Surge = 16,
    Crash = 17,
    StrongDirectionality = 18,
    WeakDirectionality = 19,
    TrendStrengthHigherThanDirection = 20,
    ADXHigherThanMDI = 21,
    PDIHigherThanADX = 22,
    MDIHigherThanADX = 23,
    TrendReversalDown = 24,
    TrendReversalUp = 25,
    DICrossover = 26,
    ExtremePDI = 27,
    ExtremeMDI = 28,
    Stable = 29,
    Unstable = 30,
}

impl From<i32> for ADXFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => ADXFilterType::BelowThreshold,
            1 => ADXFilterType::AboveThreshold,
            2 => ADXFilterType::PDIAboveMDI,
            3 => ADXFilterType::MDIAbovePDI,
            4 => ADXFilterType::StrongUptrend,
            5 => ADXFilterType::StrongDowntrend,
            6 => ADXFilterType::ADXRising,
            7 => ADXFilterType::ADXFalling,
            8 => ADXFilterType::DIGapExpanding,
            9 => ADXFilterType::DIGapContracting,
            10 => ADXFilterType::ExtremeHigh,
            11 => ADXFilterType::ExtremeLow,
            12 => ADXFilterType::MiddleLevel,
            13 => ADXFilterType::PDICrossAboveMDI,
            14 => ADXFilterType::MDICrossAbovePDI,
            15 => ADXFilterType::Sideways,
            16 => ADXFilterType::Surge,
            17 => ADXFilterType::Crash,
            18 => ADXFilterType::StrongDirectionality,
            19 => ADXFilterType::WeakDirectionality,
            20 => ADXFilterType::TrendStrengthHigherThanDirection,
            21 => ADXFilterType::ADXHigherThanMDI,
            22 => ADXFilterType::PDIHigherThanADX,
            23 => ADXFilterType::MDIHigherThanADX,
            24 => ADXFilterType::TrendReversalDown,
            25 => ADXFilterType::TrendReversalUp,
            26 => ADXFilterType::DICrossover,
            27 => ADXFilterType::ExtremePDI,
            28 => ADXFilterType::ExtremeMDI,
            29 => ADXFilterType::Stable,
            30 => ADXFilterType::Unstable,
            _ => ADXFilterType::BelowThreshold,
        }
    }
}

impl From<ADXFilterType> for i32 {
    fn from(value: ADXFilterType) -> Self {
        value as i32
    }
}

/// 이동평균선 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MovingAverageFilterType {
    PriceAboveFirstMA = 0,
    PriceAboveLastMA = 1,
    RegularArrangement = 2,
    FirstMAAboveLastMA = 3,
    FirstMABelowLastMA = 4,
    GoldenCross = 5,
    PriceBetweenMA = 6,
    MAConvergence = 7,
    MADivergence = 8,
    AllMAAbove = 9,
    AllMABelow = 10,
    ReverseArrangement = 11,
    DeadCross = 12,
    MASideways = 13,
    StrongUptrend = 14,
    StrongDowntrend = 15,
    PriceCrossingMA = 16,
    ConvergenceDivergence = 17,
    DivergenceConvergence = 18,
    ParallelMovement = 19,
    NearCrossover = 20,
}

impl From<i32> for MovingAverageFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => MovingAverageFilterType::PriceAboveFirstMA,
            1 => MovingAverageFilterType::PriceAboveLastMA,
            2 => MovingAverageFilterType::RegularArrangement,
            3 => MovingAverageFilterType::FirstMAAboveLastMA,
            4 => MovingAverageFilterType::FirstMABelowLastMA,
            5 => MovingAverageFilterType::GoldenCross,
            6 => MovingAverageFilterType::PriceBetweenMA,
            7 => MovingAverageFilterType::MAConvergence,
            8 => MovingAverageFilterType::MADivergence,
            9 => MovingAverageFilterType::AllMAAbove,
            10 => MovingAverageFilterType::AllMABelow,
            11 => MovingAverageFilterType::ReverseArrangement,
            12 => MovingAverageFilterType::DeadCross,
            13 => MovingAverageFilterType::MASideways,
            14 => MovingAverageFilterType::StrongUptrend,
            15 => MovingAverageFilterType::StrongDowntrend,
            16 => MovingAverageFilterType::PriceCrossingMA,
            17 => MovingAverageFilterType::ConvergenceDivergence,
            18 => MovingAverageFilterType::DivergenceConvergence,
            19 => MovingAverageFilterType::ParallelMovement,
            20 => MovingAverageFilterType::NearCrossover,
            _ => MovingAverageFilterType::PriceAboveFirstMA,
        }
    }
}

impl From<MovingAverageFilterType> for i32 {
    fn from(value: MovingAverageFilterType) -> Self {
        value as i32
    }
}

/// 이치모쿠 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum IchimokuFilterType {
    PriceAboveCloud = 0,
    PriceBelowCloud = 1,
    TenkanAboveKijun = 2,
    GoldenCross = 3,
    DeadCross = 4,
    CloudBreakoutUp = 5,
    CloudBreakdown = 6,
    BuySignal = 7,
    SellSignal = 8,
    CloudThickening = 9,
    PerfectAlignment = 10,
    PerfectReverseAlignment = 11,
    StrongBuySignal = 12,
}

impl From<i32> for IchimokuFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => IchimokuFilterType::PriceAboveCloud,
            1 => IchimokuFilterType::PriceBelowCloud,
            2 => IchimokuFilterType::TenkanAboveKijun,
            3 => IchimokuFilterType::GoldenCross,
            4 => IchimokuFilterType::DeadCross,
            5 => IchimokuFilterType::CloudBreakoutUp,
            6 => IchimokuFilterType::CloudBreakdown,
            7 => IchimokuFilterType::BuySignal,
            8 => IchimokuFilterType::SellSignal,
            9 => IchimokuFilterType::CloudThickening,
            10 => IchimokuFilterType::PerfectAlignment,
            11 => IchimokuFilterType::PerfectReverseAlignment,
            12 => IchimokuFilterType::StrongBuySignal,
            _ => IchimokuFilterType::PriceAboveCloud,
        }
    }
}

impl From<IchimokuFilterType> for i32 {
    fn from(value: IchimokuFilterType) -> Self {
        value as i32
    }
}

/// VWAP 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum VWAPFilterType {
    PriceAboveVWAP = 0,
    PriceBelowVWAP = 1,
    PriceNearVWAP = 2,
    VWAPBreakoutUp = 3,
    VWAPBreakdown = 4,
    VWAPRebound = 5,
    DivergingFromVWAP = 6,
    ConvergingToVWAP = 7,
    StrongUptrend = 8,
    StrongDowntrend = 9,
    TrendStrengthening = 10,
    TrendWeakening = 11,
}

impl From<i32> for VWAPFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => VWAPFilterType::PriceAboveVWAP,
            1 => VWAPFilterType::PriceBelowVWAP,
            2 => VWAPFilterType::PriceNearVWAP,
            3 => VWAPFilterType::VWAPBreakoutUp,
            4 => VWAPFilterType::VWAPBreakdown,
            5 => VWAPFilterType::VWAPRebound,
            6 => VWAPFilterType::DivergingFromVWAP,
            7 => VWAPFilterType::ConvergingToVWAP,
            8 => VWAPFilterType::StrongUptrend,
            9 => VWAPFilterType::StrongDowntrend,
            10 => VWAPFilterType::TrendStrengthening,
            11 => VWAPFilterType::TrendWeakening,
            _ => VWAPFilterType::PriceAboveVWAP,
        }
    }
}

impl From<VWAPFilterType> for i32 {
    fn from(value: VWAPFilterType) -> Self {
        value as i32
    }
}

/// CopyS 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum CopysFilterType {
    BasicBuySignal = 0,
    BasicSellSignal = 1,
    RSIOversold = 2,
    RSIOverbought = 3,
    BBandLowerTouch = 4,
    BBandUpperTouch = 5,
    MASupport = 6,
    MAResistance = 7,
    StrongBuySignal = 8,
    StrongSellSignal = 9,
    WeakBuySignal = 10,
    WeakSellSignal = 11,
    RSINeutral = 12,
    BBandInside = 13,
    MARegularArrangement = 14,
    MAReverseArrangement = 15,
}

impl From<i32> for CopysFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => CopysFilterType::BasicBuySignal,
            1 => CopysFilterType::BasicSellSignal,
            2 => CopysFilterType::RSIOversold,
            3 => CopysFilterType::RSIOverbought,
            4 => CopysFilterType::BBandLowerTouch,
            5 => CopysFilterType::BBandUpperTouch,
            6 => CopysFilterType::MASupport,
            7 => CopysFilterType::MAResistance,
            8 => CopysFilterType::StrongBuySignal,
            9 => CopysFilterType::StrongSellSignal,
            10 => CopysFilterType::WeakBuySignal,
            11 => CopysFilterType::WeakSellSignal,
            12 => CopysFilterType::RSINeutral,
            13 => CopysFilterType::BBandInside,
            14 => CopysFilterType::MARegularArrangement,
            15 => CopysFilterType::MAReverseArrangement,
            _ => CopysFilterType::BasicBuySignal,
        }
    }
}

impl From<CopysFilterType> for i32 {
    fn from(value: CopysFilterType) -> Self {
        value as i32
    }
}

/// ATR 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum ATRFilterType {
    AboveThreshold = 0,
    VolatilityExpanding = 1,
    VolatilityContracting = 2,
    HighVolatility = 3,
    LowVolatility = 4,
    VolatilityIncreasing = 5,
    VolatilityDecreasing = 6,
}

impl From<i32> for ATRFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => ATRFilterType::AboveThreshold,
            1 => ATRFilterType::VolatilityExpanding,
            2 => ATRFilterType::VolatilityContracting,
            3 => ATRFilterType::HighVolatility,
            4 => ATRFilterType::LowVolatility,
            5 => ATRFilterType::VolatilityIncreasing,
            6 => ATRFilterType::VolatilityDecreasing,
            _ => ATRFilterType::AboveThreshold,
        }
    }
}

impl From<ATRFilterType> for i32 {
    fn from(value: ATRFilterType) -> Self {
        value as i32
    }
}

/// SuperTrend 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum SuperTrendFilterType {
    AllUptrend = 0,
    AllDowntrend = 1,
    PriceAboveSupertrend = 2,
    PriceBelowSupertrend = 3,
    PriceCrossingAbove = 4,
    PriceCrossingBelow = 5,
    TrendChanged = 6,
    Uptrend = 7,
    Downtrend = 8,
}

impl From<i32> for SuperTrendFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => SuperTrendFilterType::AllUptrend,
            1 => SuperTrendFilterType::AllDowntrend,
            2 => SuperTrendFilterType::PriceAboveSupertrend,
            3 => SuperTrendFilterType::PriceBelowSupertrend,
            4 => SuperTrendFilterType::PriceCrossingAbove,
            5 => SuperTrendFilterType::PriceCrossingBelow,
            6 => SuperTrendFilterType::TrendChanged,
            7 => SuperTrendFilterType::Uptrend,
            8 => SuperTrendFilterType::Downtrend,
            _ => SuperTrendFilterType::AllUptrend,
        }
    }
}

impl From<SuperTrendFilterType> for i32 {
    fn from(value: SuperTrendFilterType) -> Self {
        value as i32
    }
}

/// Volume 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum VolumeFilterType {
    VolumeAboveAverage = 0,
    VolumeBelowAverage = 1,
    VolumeSurge = 2,
    VolumeDecline = 3,
    VolumeSignificantlyAbove = 4,
    BullishWithIncreasedVolume = 5,
    BearishWithIncreasedVolume = 6,
    IncreasingVolumeInUptrend = 7,
    DecreasingVolumeInDowntrend = 8,
    VolumeSharpDecline = 9,
    VolumeStable = 10,
    VolumeVolatile = 11,
    BullishWithDecreasedVolume = 12,
    BearishWithDecreasedVolume = 13,
    VolumeDoubleAverage = 14,
    VolumeHalfAverage = 15,
    VolumeConsecutiveIncrease = 16,
    VolumeConsecutiveDecrease = 17,
    VolumeSideways = 18,
    VolumeExtremelyHigh = 19,
    VolumeExtremelyLow = 20,
}

impl From<i32> for VolumeFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => VolumeFilterType::VolumeAboveAverage,
            1 => VolumeFilterType::VolumeBelowAverage,
            2 => VolumeFilterType::VolumeSurge,
            3 => VolumeFilterType::VolumeDecline,
            4 => VolumeFilterType::VolumeSignificantlyAbove,
            5 => VolumeFilterType::BullishWithIncreasedVolume,
            6 => VolumeFilterType::BearishWithIncreasedVolume,
            7 => VolumeFilterType::IncreasingVolumeInUptrend,
            8 => VolumeFilterType::DecreasingVolumeInDowntrend,
            9 => VolumeFilterType::VolumeSharpDecline,
            10 => VolumeFilterType::VolumeStable,
            11 => VolumeFilterType::VolumeVolatile,
            12 => VolumeFilterType::BullishWithDecreasedVolume,
            13 => VolumeFilterType::BearishWithDecreasedVolume,
            14 => VolumeFilterType::VolumeDoubleAverage,
            15 => VolumeFilterType::VolumeHalfAverage,
            16 => VolumeFilterType::VolumeConsecutiveIncrease,
            17 => VolumeFilterType::VolumeConsecutiveDecrease,
            18 => VolumeFilterType::VolumeSideways,
            19 => VolumeFilterType::VolumeExtremelyHigh,
            20 => VolumeFilterType::VolumeExtremelyLow,
            _ => VolumeFilterType::VolumeAboveAverage,
        }
    }
}

impl From<VolumeFilterType> for i32 {
    fn from(value: VolumeFilterType) -> Self {
        value as i32
    }
}

/// ThreeRSI 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum ThreeRSIFilterType {
    AllRSILessThan50 = 0,
    AllRSIGreaterThan50 = 1,
    RSIReverseArrangement = 2,
    RSIRegularArrangement = 3,
    CandleLowBelowMA = 4,
    CandleHighAboveMA = 5,
    ADXGreaterThan20 = 6,
    AllRSILessThan30 = 7,
    AllRSIGreaterThan70 = 8,
    RSIStableRange = 9,
    RSIBullishRange = 10,
    RSIBearishRange = 11,
    RSIOverboughtRange = 12,
    RSIOversoldRange = 13,
    RSICrossAbove50 = 14,
    RSICrossBelow50 = 15,
    RSICrossAbove40 = 16,
    RSICrossBelow60 = 17,
    RSICrossAbove20 = 18,
    RSICrossBelow80 = 19,
    RSISideways = 20,
    RSIBullishMomentum = 21,
    RSIBearishMomentum = 22,
    RSIDivergence = 23,
    RSIConvergence = 24,
    RSIDoubleBottom = 25,
    RSIDoubleTop = 26,
    RSIOverboughtReversal = 27,
    RSIOversoldReversal = 28,
    RSINeutralTrend = 29,
    RSIExtremeOverbought = 30,
    RSIExtremeOversold = 31,
}

impl From<i32> for ThreeRSIFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => ThreeRSIFilterType::AllRSILessThan50,
            1 => ThreeRSIFilterType::AllRSIGreaterThan50,
            2 => ThreeRSIFilterType::RSIReverseArrangement,
            3 => ThreeRSIFilterType::RSIRegularArrangement,
            4 => ThreeRSIFilterType::CandleLowBelowMA,
            5 => ThreeRSIFilterType::CandleHighAboveMA,
            6 => ThreeRSIFilterType::ADXGreaterThan20,
            7 => ThreeRSIFilterType::AllRSILessThan30,
            8 => ThreeRSIFilterType::AllRSIGreaterThan70,
            9 => ThreeRSIFilterType::RSIStableRange,
            10 => ThreeRSIFilterType::RSIBullishRange,
            11 => ThreeRSIFilterType::RSIBearishRange,
            12 => ThreeRSIFilterType::RSIOverboughtRange,
            13 => ThreeRSIFilterType::RSIOversoldRange,
            14 => ThreeRSIFilterType::RSICrossAbove50,
            15 => ThreeRSIFilterType::RSICrossBelow50,
            16 => ThreeRSIFilterType::RSICrossAbove40,
            17 => ThreeRSIFilterType::RSICrossBelow60,
            18 => ThreeRSIFilterType::RSICrossAbove20,
            19 => ThreeRSIFilterType::RSICrossBelow80,
            20 => ThreeRSIFilterType::RSISideways,
            21 => ThreeRSIFilterType::RSIBullishMomentum,
            22 => ThreeRSIFilterType::RSIBearishMomentum,
            23 => ThreeRSIFilterType::RSIDivergence,
            24 => ThreeRSIFilterType::RSIConvergence,
            25 => ThreeRSIFilterType::RSIDoubleBottom,
            26 => ThreeRSIFilterType::RSIDoubleTop,
            27 => ThreeRSIFilterType::RSIOverboughtReversal,
            28 => ThreeRSIFilterType::RSIOversoldReversal,
            29 => ThreeRSIFilterType::RSINeutralTrend,
            30 => ThreeRSIFilterType::RSIExtremeOverbought,
            31 => ThreeRSIFilterType::RSIExtremeOversold,
            _ => ThreeRSIFilterType::AllRSILessThan50,
        }
    }
}

impl From<ThreeRSIFilterType> for i32 {
    fn from(value: ThreeRSIFilterType) -> Self {
        value as i32
    }
}

/// CandlePattern 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum CandlePatternFilterType {
    StrongBullishPattern = 0,
    StrongBearishPattern = 1,
    ReversalPattern = 2,
    ContinuationPattern = 3,
    VolumeConfirmedPattern = 4,
    HighReliabilityPattern = 5,
    ContextAlignedPattern = 6,
    StrongReversalSignal = 7,
    HighConfidenceSignal = 8,
    VolumeConfirmedSignal = 9,
    PatternClusteringSignal = 10,
    HammerPattern = 11,
    ShootingStarPattern = 12,
    DojiPattern = 13,
    SpinningTopPattern = 14,
    MarubozuPattern = 15,
    MorningStarPattern = 16,
    EveningStarPattern = 17,
    EngulfingPattern = 18,
    PiercingPattern = 19,
    DarkCloudPattern = 20,
    HaramiPattern = 21,
    TweezerPattern = 22,
    TriStarPattern = 23,
    AdvanceBlockPattern = 24,
    DeliberanceBlockPattern = 25,
    BreakawayPattern = 26,
    ConcealmentPattern = 27,
    CounterattackPattern = 28,
    DarkCloudCoverPattern = 29,
    RisingWindowPattern = 30,
    FallingWindowPattern = 31,
    HighBreakoutPattern = 32,
    LowBreakoutPattern = 33,
    GapPattern = 34,
    GapFillPattern = 35,
    DoubleBottomPattern = 36,
    DoubleTopPattern = 37,
    TrianglePattern = 38,
    FlagPattern = 39,
    PennantPattern = 40,
}

impl From<i32> for CandlePatternFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => CandlePatternFilterType::StrongBullishPattern,
            1 => CandlePatternFilterType::StrongBearishPattern,
            2 => CandlePatternFilterType::ReversalPattern,
            3 => CandlePatternFilterType::ContinuationPattern,
            4 => CandlePatternFilterType::VolumeConfirmedPattern,
            5 => CandlePatternFilterType::HighReliabilityPattern,
            6 => CandlePatternFilterType::ContextAlignedPattern,
            7 => CandlePatternFilterType::StrongReversalSignal,
            8 => CandlePatternFilterType::HighConfidenceSignal,
            9 => CandlePatternFilterType::VolumeConfirmedSignal,
            10 => CandlePatternFilterType::PatternClusteringSignal,
            11 => CandlePatternFilterType::HammerPattern,
            12 => CandlePatternFilterType::ShootingStarPattern,
            13 => CandlePatternFilterType::DojiPattern,
            14 => CandlePatternFilterType::SpinningTopPattern,
            15 => CandlePatternFilterType::MarubozuPattern,
            16 => CandlePatternFilterType::MorningStarPattern,
            17 => CandlePatternFilterType::EveningStarPattern,
            18 => CandlePatternFilterType::EngulfingPattern,
            19 => CandlePatternFilterType::PiercingPattern,
            20 => CandlePatternFilterType::DarkCloudPattern,
            21 => CandlePatternFilterType::HaramiPattern,
            22 => CandlePatternFilterType::TweezerPattern,
            23 => CandlePatternFilterType::TriStarPattern,
            24 => CandlePatternFilterType::AdvanceBlockPattern,
            25 => CandlePatternFilterType::DeliberanceBlockPattern,
            26 => CandlePatternFilterType::BreakawayPattern,
            27 => CandlePatternFilterType::ConcealmentPattern,
            28 => CandlePatternFilterType::CounterattackPattern,
            29 => CandlePatternFilterType::DarkCloudCoverPattern,
            30 => CandlePatternFilterType::RisingWindowPattern,
            31 => CandlePatternFilterType::FallingWindowPattern,
            32 => CandlePatternFilterType::HighBreakoutPattern,
            33 => CandlePatternFilterType::LowBreakoutPattern,
            34 => CandlePatternFilterType::GapPattern,
            35 => CandlePatternFilterType::GapFillPattern,
            36 => CandlePatternFilterType::DoubleBottomPattern,
            37 => CandlePatternFilterType::DoubleTopPattern,
            38 => CandlePatternFilterType::TrianglePattern,
            39 => CandlePatternFilterType::FlagPattern,
            40 => CandlePatternFilterType::PennantPattern,
            _ => CandlePatternFilterType::StrongBullishPattern,
        }
    }
}

impl From<CandlePatternFilterType> for i32 {
    fn from(value: CandlePatternFilterType) -> Self {
        value as i32
    }
}

/// SupportResistance 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum SupportResistanceFilterType {
    SupportBreakdown = 0,
    ResistanceBreakout = 1,
    SupportBounce = 2,
    ResistanceRejection = 3,
    NearStrongSupport = 4,
    NearStrongResistance = 5,
    AboveSupport = 6,
    BelowResistance = 7,
    NearSupport = 8,
    NearResistance = 9,
}

impl From<i32> for SupportResistanceFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => SupportResistanceFilterType::SupportBreakdown,
            1 => SupportResistanceFilterType::ResistanceBreakout,
            2 => SupportResistanceFilterType::SupportBounce,
            3 => SupportResistanceFilterType::ResistanceRejection,
            4 => SupportResistanceFilterType::NearStrongSupport,
            5 => SupportResistanceFilterType::NearStrongResistance,
            6 => SupportResistanceFilterType::AboveSupport,
            7 => SupportResistanceFilterType::BelowResistance,
            8 => SupportResistanceFilterType::NearSupport,
            9 => SupportResistanceFilterType::NearResistance,
            _ => SupportResistanceFilterType::SupportBreakdown,
        }
    }
}

impl From<SupportResistanceFilterType> for i32 {
    fn from(value: SupportResistanceFilterType) -> Self {
        value as i32
    }
}

/// Momentum 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MomentumFilterType {
    StrongPositiveMomentum = 0,
    StrongNegativeMomentum = 1,
    AcceleratingMomentum = 2,
    DeceleratingMomentum = 3,
    Overbought = 4,
    Oversold = 5,
    MomentumDivergence = 6,
    BullishDivergence = 7,
    BearishDivergence = 8,
    PersistentMomentum = 9,
    StableMomentum = 10,
    MomentumReversalSignal = 11,
    MomentumSideways = 12,
    MomentumSurge = 13,
    MomentumCrash = 14,
    MomentumConvergence = 15,
    MomentumDivergencePattern = 16,
    MomentumParallel = 17,
    MomentumCrossover = 18,
    MomentumSupportTest = 19,
    MomentumResistanceTest = 20,
}

impl From<i32> for MomentumFilterType {
    fn from(value: i32) -> Self {
        match value {
            0 => MomentumFilterType::StrongPositiveMomentum,
            1 => MomentumFilterType::StrongNegativeMomentum,
            2 => MomentumFilterType::AcceleratingMomentum,
            3 => MomentumFilterType::DeceleratingMomentum,
            4 => MomentumFilterType::Overbought,
            5 => MomentumFilterType::Oversold,
            6 => MomentumFilterType::MomentumDivergence,
            7 => MomentumFilterType::BullishDivergence,
            8 => MomentumFilterType::BearishDivergence,
            9 => MomentumFilterType::PersistentMomentum,
            10 => MomentumFilterType::StableMomentum,
            11 => MomentumFilterType::MomentumReversalSignal,
            12 => MomentumFilterType::MomentumSideways,
            13 => MomentumFilterType::MomentumSurge,
            14 => MomentumFilterType::MomentumCrash,
            15 => MomentumFilterType::MomentumConvergence,
            16 => MomentumFilterType::MomentumDivergencePattern,
            17 => MomentumFilterType::MomentumParallel,
            18 => MomentumFilterType::MomentumCrossover,
            19 => MomentumFilterType::MomentumSupportTest,
            20 => MomentumFilterType::MomentumResistanceTest,
            _ => MomentumFilterType::StrongPositiveMomentum,
        }
    }
}

impl From<MomentumFilterType> for i32 {
    fn from(value: MomentumFilterType) -> Self {
        value as i32
    }
}

/// RSI 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIParams {
    /// RSI 계산 기간 (기본값: 14)
    pub period: usize,
    /// 과매도 기준점 (기본값: 30)
    pub oversold: f64,
    /// 과매수 기준점 (기본값: 70)
    pub overbought: f64,
    /// 필터 유형
    pub filter_type: RSIFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// 횡보 임계값 (변화율, 기본값: 0.02 = 2%)
    #[serde(default = "default_rsi_sideways_threshold")]
    pub sideways_threshold: f64,
    /// 강한 모멘텀 임계값 (RSI 변화량, 기본값: 3.0)
    #[serde(default = "default_rsi_momentum_threshold")]
    pub momentum_threshold: f64,
}

fn default_rsi_sideways_threshold() -> f64 {
    0.02
}

fn default_rsi_momentum_threshold() -> f64 {
    3.0
}

impl Default for RSIParams {
    fn default() -> Self {
        Self {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: RSIFilterType::Overbought,
            consecutive_n: 1,
            p: 0,
            sideways_threshold: 0.02,
            momentum_threshold: 3.0,
        }
    }
}

/// MACD 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDParams {
    /// 빠른 이동평균 기간 (기본값: 12)
    pub fast_period: usize,
    /// 느린 이동평균 기간 (기본값: 26)
    pub slow_period: usize,
    /// 시그널 기간 (기본값: 9)
    pub signal_period: usize,
    /// 필터 유형
    pub filter_type: MACDFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 히스토그램 임계값 (기본값: 0)
    pub threshold: f64,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// 과매수 임계값 (MACD/가격 비율, 기본값: 0.02 = 2%)
    #[serde(default = "default_macd_overbought_threshold")]
    pub overbought_threshold: f64,
    /// 과매도 임계값 (MACD/가격 비율, 기본값: 0.02 = 2%)
    #[serde(default = "default_macd_oversold_threshold")]
    pub oversold_threshold: f64,
    /// 횡보 임계값 (변화율, 기본값: 0.05 = 5%)
    #[serde(default = "default_macd_sideways_threshold")]
    pub sideways_threshold: f64,
}

fn default_macd_overbought_threshold() -> f64 {
    0.02
}

fn default_macd_oversold_threshold() -> f64 {
    0.02
}

fn default_macd_sideways_threshold() -> f64 {
    0.05
}

impl Default for MACDParams {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            filter_type: MACDFilterType::MacdAboveSignal,
            consecutive_n: 1,
            threshold: 0.0,
            p: 0,
            overbought_threshold: 0.02,
            oversold_threshold: 0.02,
            sideways_threshold: 0.05,
        }
    }
}

/// 볼린저 밴드 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBandParams {
    /// 볼린저 밴드 기간 (기본값: 20)
    pub period: usize,
    /// 표준편차 배수 (기본값: 2.0)
    pub dev_mult: f64,
    /// 필터 유형
    pub filter_type: BollingerBandFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// 스퀴즈/횡보 임계값 (기본값: 0.02 = 2%)
    #[serde(default = "default_bband_squeeze_threshold")]
    pub squeeze_threshold: f64,
    /// 중간 변동성 임계값 (기본값: 0.05 = 5%)
    #[serde(default = "default_bband_medium_threshold")]
    pub medium_threshold: f64,
    /// 큰 변동성/가격 이동 임계값 (기본값: 0.1 = 10%)
    #[serde(default = "default_bband_large_threshold")]
    pub large_threshold: f64,
    /// 스퀴즈 브레이크아웃 확인 기간 (기본값: 5)
    #[serde(default = "default_bband_squeeze_breakout_period")]
    pub squeeze_breakout_period: usize,
    /// 향상된 스퀴즈 브레이크아웃 좁아지는 기간 (기본값: 3)
    #[serde(default = "default_bband_enhanced_narrowing_period")]
    pub enhanced_narrowing_period: usize,
    /// 향상된 스퀴즈 브레이크아웃 스퀴즈 기간 (기본값: 2)
    #[serde(default = "default_bband_enhanced_squeeze_period")]
    pub enhanced_squeeze_period: usize,
    /// 상단 밴드 터치 임계값 (기본값: 0.99 = 99%)
    #[serde(default = "default_bband_upper_touch_threshold")]
    pub upper_touch_threshold: f64,
    /// 하단 밴드 터치 임계값 (기본값: 1.01 = 101%)
    #[serde(default = "default_bband_lower_touch_threshold")]
    pub lower_touch_threshold: f64,
}

fn default_bband_squeeze_breakout_period() -> usize {
    5
}

fn default_bband_enhanced_narrowing_period() -> usize {
    3
}

fn default_bband_enhanced_squeeze_period() -> usize {
    2
}

fn default_bband_upper_touch_threshold() -> f64 {
    0.99
}

fn default_bband_lower_touch_threshold() -> f64 {
    1.01
}

fn default_bband_squeeze_threshold() -> f64 {
    0.02
}

fn default_bband_medium_threshold() -> f64 {
    0.05
}

fn default_bband_large_threshold() -> f64 {
    0.1
}

impl Default for BollingerBandParams {
    fn default() -> Self {
        Self {
            period: 20,
            dev_mult: 2.0,
            filter_type: BollingerBandFilterType::AboveUpperBand,
            consecutive_n: 1,
            p: 0,
            squeeze_threshold: 0.02,
            medium_threshold: 0.05,
            large_threshold: 0.1,
            squeeze_breakout_period: 5,
            enhanced_narrowing_period: 3,
            enhanced_squeeze_period: 2,
            upper_touch_threshold: 0.99,
            lower_touch_threshold: 1.01,
        }
    }
}

/// ADX 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADXParams {
    /// ADX 계산 기간 (기본값: 14)
    pub period: usize,
    /// ADX 임계값 (기본값: 25.0)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: ADXFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for ADXParams {
    fn default() -> Self {
        Self {
            period: 14,
            threshold: 25.0,
            filter_type: ADXFilterType::BelowThreshold,
            consecutive_n: 1,
            p: 0,
        }
    }
}

/// 이동평균선 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovingAverageParams {
    /// 이동평균 기간 목록
    pub periods: Vec<usize>,
    /// 필터 유형
    pub filter_type: MovingAverageFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// 횡보 판단 임계값 (기본값: 0.02 = 2%)
    #[serde(default = "default_ma_sideways_threshold")]
    pub sideways_threshold: f64,
    /// 교차점 근처 판단 임계값 (기본값: 0.005 = 0.5%)
    #[serde(default = "default_ma_crossover_threshold")]
    pub crossover_threshold: f64,
}

fn default_ma_sideways_threshold() -> f64 {
    0.02
}

fn default_ma_crossover_threshold() -> f64 {
    0.005
}

impl Default for MovingAverageParams {
    fn default() -> Self {
        Self {
            periods: vec![5, 20],
            filter_type: MovingAverageFilterType::PriceAboveFirstMA,
            consecutive_n: 1,
            p: 0,
            sideways_threshold: 0.02,
            crossover_threshold: 0.005,
        }
    }
}

/// 이치모쿠 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IchimokuParams {
    /// 전환선 기간 (기본값: 9)
    pub tenkan_period: usize,
    /// 기준선 기간 (기본값: 26)
    pub kijun_period: usize,
    /// 선행스팬B 기간 (기본값: 52)
    pub senkou_span_b_period: usize,
    /// 필터 유형
    pub filter_type: IchimokuFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for IchimokuParams {
    fn default() -> Self {
        Self {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_span_b_period: 52,
            filter_type: IchimokuFilterType::PriceAboveCloud,
            consecutive_n: 1,
            p: 0,
        }
    }
}

/// VWAP 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VWAPParams {
    /// VWAP 계산 기간 (기본값: 20)
    pub period: usize,
    /// 필터 유형
    pub filter_type: VWAPFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 임계값 (기본값: 0.05 - 5%)
    pub threshold: f64,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for VWAPParams {
    fn default() -> Self {
        Self {
            period: 20,
            filter_type: VWAPFilterType::PriceAboveVWAP,
            consecutive_n: 1,
            threshold: 0.05,
            p: 0,
        }
    }
}

/// CopyS 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopysParams {
    /// RSI 계산 기간 (기본값: 14)
    pub rsi_period: usize,
    /// RSI 상한 기준점 (기본값: 70)
    pub rsi_upper: f64,
    /// RSI 하한 기준점 (기본값: 30)
    pub rsi_lower: f64,
    /// 필터 유형
    pub filter_type: CopysFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// 볼린저밴드 기간 (기본값: 20)
    #[serde(default = "default_copys_bband_period")]
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 배수 (기본값: 2.0)
    #[serde(default = "default_copys_bband_multiplier")]
    pub bband_multiplier: f64,
    /// 이동평균 기간 목록 (기본값: [5, 20, 60, 120, 200, 240])
    #[serde(default = "default_copys_ma_periods")]
    pub ma_periods: Vec<usize>,
}

fn default_copys_bband_period() -> usize {
    20
}

fn default_copys_bband_multiplier() -> f64 {
    2.0
}

fn default_copys_ma_periods() -> Vec<usize> {
    vec![5, 20, 60, 120, 200, 240]
}

/// ATR 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATRParams {
    /// ATR 계산 기간 (기본값: 14)
    pub period: usize,
    /// ATR 임계값 (기본값: 0.01)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: ATRFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// SuperTrend 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperTrendParams {
    /// SuperTrend 계산 기간 (기본값: 10)
    pub period: usize,
    /// SuperTrend 승수 (기본값: 3.0)
    pub multiplier: f64,
    /// 필터 유형
    pub filter_type: SuperTrendFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// Volume 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeParams {
    /// Volume 계산 기간 (기본값: 20)
    pub period: usize,
    /// Volume 임계값 (기본값: 1.5)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: VolumeFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
    /// VolumeStable 필터의 최소 임계값 (기본값: 0.1)
    #[serde(default = "default_volume_stable_min_threshold")]
    pub stable_min_threshold: f64,
}

fn default_volume_stable_min_threshold() -> f64 {
    0.1
}

/// ThreeRSI 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeRSIParams {
    /// RSI 계산 기간 목록 (기본값: [7, 14, 21])
    pub rsi_periods: Vec<usize>,
    /// 이동평균 타입 (기본값: SMA)
    pub ma_type: String,
    /// 이동평균 기간 (기본값: 20)
    pub ma_period: usize,
    /// ADX 기간 (기본값: 14)
    pub adx_period: usize,
    /// 필터 유형
    pub filter_type: ThreeRSIFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// CandlePattern 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlePatternParams {
    /// 최소 몸통 크기 비율 (기본값: 0.3)
    pub min_body_ratio: f64,
    /// 최소 꼬리 크기 비율 (기본값: 0.3)
    pub min_shadow_ratio: f64,
    /// 패턴 히스토리 길이 (기본값: 5)
    pub pattern_history_length: usize,
    /// 임계값 (기본값: 0.5)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: CandlePatternFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// SupportResistance 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportResistanceParams {
    /// 되돌아 볼 기간 (기본값: 20)
    pub lookback_period: usize,
    /// 터치 임계값 (기본값: 0.01)
    pub touch_threshold: f64,
    /// 최소 터치 횟수 (기본값: 2)
    pub min_touch_count: usize,
    /// 거리 임계값 (기본값: 0.05)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: SupportResistanceFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// Momentum 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumParams {
    /// RSI 기간 (기본값: 14)
    pub rsi_period: usize,
    /// 스토캐스틱 기간 (기본값: 14)
    pub stoch_period: usize,
    /// 윌리엄스 %R 기간 (기본값: 14)
    pub williams_period: usize,
    /// ROC 기간 (기본값: 10)
    pub roc_period: usize,
    /// CCI 기간 (기본값: 20)
    pub cci_period: usize,
    /// 모멘텀 기간 (기본값: 10)
    pub momentum_period: usize,
    /// 히스토리 길이 (기본값: 50)
    pub history_length: usize,
    /// 임계값 (기본값: 0.5)
    pub threshold: f64,
    /// 필터 유형
    pub filter_type: MomentumFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for CopysParams {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            filter_type: CopysFilterType::BasicBuySignal,
            consecutive_n: 1,
            p: 0,
            bband_period: 20,
            bband_multiplier: 2.0,
            ma_periods: vec![5, 20, 60, 120, 200, 240],
        }
    }
}

impl Default for ATRParams {
    fn default() -> Self {
        Self {
            period: 14,
            threshold: 0.01,
            filter_type: ATRFilterType::AboveThreshold,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for SuperTrendParams {
    fn default() -> Self {
        Self {
            period: 10,
            multiplier: 3.0,
            filter_type: SuperTrendFilterType::AllUptrend,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for VolumeParams {
    fn default() -> Self {
        Self {
            period: 20,
            threshold: 1.5,
            filter_type: VolumeFilterType::VolumeAboveAverage,
            consecutive_n: 1,
            p: 0,
            stable_min_threshold: 0.1,
        }
    }
}

impl Default for ThreeRSIParams {
    fn default() -> Self {
        Self {
            rsi_periods: vec![7, 14, 21],
            ma_type: "SMA".to_string(),
            ma_period: 20,
            adx_period: 14,
            filter_type: ThreeRSIFilterType::AllRSILessThan50,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for CandlePatternParams {
    fn default() -> Self {
        Self {
            min_body_ratio: 0.3,
            min_shadow_ratio: 0.3,
            pattern_history_length: 5,
            threshold: 0.5,
            filter_type: CandlePatternFilterType::StrongBullishPattern,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for SupportResistanceParams {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            touch_threshold: 0.01,
            min_touch_count: 2,
            threshold: 0.05,
            filter_type: SupportResistanceFilterType::SupportBreakdown,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for MomentumParams {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            stoch_period: 14,
            williams_period: 14,
            roc_period: 10,
            cci_period: 20,
            momentum_period: 10,
            history_length: 50,
            threshold: 0.5,
            filter_type: MomentumFilterType::StrongPositiveMomentum,
            consecutive_n: 1,
            p: 0,
        }
    }
}

/// 기술적 필터 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TechnicalFilterConfig {
    /// RSI 필터 설정
    RSI(RSIParams),
    /// MACD 필터 설정
    MACD(MACDParams),
    /// 볼린저 밴드 필터 설정
    #[serde(alias = "BB", rename = "BOLLINGER_BAND")]
    BollingerBand(BollingerBandParams),
    /// ADX 필터 설정
    ADX(ADXParams),
    /// 이동평균선 필터 설정
    #[serde(alias = "MA", rename = "MOVING_AVERAGE")]
    MovingAverage(MovingAverageParams),
    /// 이치모쿠 필터 설정
    #[serde(rename = "ICHIMOKU")]
    Ichimoku(IchimokuParams),
    /// VWAP 필터 설정
    VWAP(VWAPParams),
    /// CopyS 필터 설정
    #[serde(rename = "COPYS")]
    Copys(CopysParams),
    /// ATR 필터 설정
    #[serde(rename = "ATR")]
    ATR(ATRParams),
    /// SuperTrend 필터 설정
    #[serde(rename = "SUPERTREND")]
    SuperTrend(SuperTrendParams),
    /// Volume 필터 설정
    #[serde(rename = "VOLUME")]
    Volume(VolumeParams),
    /// ThreeRSI 필터 설정
    #[serde(rename = "THREERSI")]
    ThreeRSI(ThreeRSIParams),
    /// CandlePattern 필터 설정
    #[serde(rename = "CANDLEPATTERN")]
    CandlePattern(CandlePatternParams),
    /// SupportResistance 필터 설정
    #[serde(rename = "SUPPORTRESISTANCE")]
    SupportResistance(SupportResistanceParams),
    /// Momentum 필터 설정
    #[serde(rename = "MOMENTUM")]
    Momentum(MomentumParams),
}

impl TechnicalFilterConfig {
    /// 필터 타입 가져오기
    pub fn filter_type(&self) -> TechnicalFilterType {
        match self {
            Self::RSI(_) => TechnicalFilterType::RSI,
            Self::MACD(_) => TechnicalFilterType::MACD,
            Self::BollingerBand(_) => TechnicalFilterType::BollingerBand,
            Self::ADX(_) => TechnicalFilterType::ADX,
            Self::MovingAverage(_) => TechnicalFilterType::MovingAverage,
            Self::Ichimoku(_) => TechnicalFilterType::Ichimoku,
            Self::VWAP(_) => TechnicalFilterType::VWAP,
            Self::Copys(_) => TechnicalFilterType::Copys,
            Self::ATR(_) => TechnicalFilterType::ATR,
            Self::SuperTrend(_) => TechnicalFilterType::SuperTrend,
            Self::Volume(_) => TechnicalFilterType::Volume,
            Self::ThreeRSI(_) => TechnicalFilterType::ThreeRSI,
            Self::CandlePattern(_) => TechnicalFilterType::CandlePattern,
            Self::SupportResistance(_) => TechnicalFilterType::SupportResistance,
            Self::Momentum(_) => TechnicalFilterType::Momentum,
        }
    }
}

// 각 필터 함수 재노출(re-export)
pub use adx::filter_adx;
pub use bollinger_band::filter_bollinger_band;
pub use copys::filter_copys;
pub use ichimoku::{IchimokuValues, filter_ichimoku};
pub use macd::filter_macd;
pub use moving_average::filter_moving_average;
pub use rsi::filter_rsi;
pub use vwap::filter_vwap;

/// 기술적 지표 필터링 적용
pub struct TechnicalFilter;

impl TechnicalFilter {
    /// 개별 코인에 대한 기술적 필터 적용
    pub fn check_filter<C: Candle + 'static>(
        symbol: &str,
        filter: &TechnicalFilterConfig,
        candles: &[C],
    ) -> Result<bool> {
        match filter {
            TechnicalFilterConfig::RSI(params) => filter_rsi(symbol, params, candles),
            TechnicalFilterConfig::MACD(params) => filter_macd(symbol, params, candles),
            TechnicalFilterConfig::BollingerBand(params) => {
                filter_bollinger_band(symbol, params, candles)
            }
            TechnicalFilterConfig::ADX(params) => filter_adx(symbol, params, candles),
            TechnicalFilterConfig::MovingAverage(params) => {
                filter_moving_average(symbol, params, candles)
            }
            TechnicalFilterConfig::Ichimoku(params) => filter_ichimoku(symbol, params, candles),
            TechnicalFilterConfig::VWAP(params) => filter_vwap(symbol, params, candles),
            TechnicalFilterConfig::Copys(params) => filter_copys(symbol, params, candles),
            TechnicalFilterConfig::ATR(params) => atr::filter_atr(symbol, params, candles),
            TechnicalFilterConfig::SuperTrend(params) => {
                supertrend::filter_supertrend(symbol, params, candles)
            }
            TechnicalFilterConfig::Volume(params) => volume::filter_volume(symbol, params, candles),
            TechnicalFilterConfig::ThreeRSI(params) => {
                three_rsi::filter_three_rsi(symbol, params, candles)
            }
            TechnicalFilterConfig::CandlePattern(params) => {
                candle_pattern::filter_candle_pattern(symbol, params, candles)
            }
            TechnicalFilterConfig::SupportResistance(params) => {
                support_resistance::filter_support_resistance(symbol, params, candles)
            }
            TechnicalFilterConfig::Momentum(params) => {
                momentum::filter_momentum(symbol, params, candles)
            }
        }
    }

    /// 개별 코인에 여러 기술적 필터 적용
    pub fn check_filters<C: Candle + 'static>(
        symbol: &str,
        filters: &[TechnicalFilterConfig],
        candles: &[C],
    ) -> Result<bool> {
        for filter in filters {
            log::debug!(
                "코인 {} 기술적 필터 적용 중: {:?}",
                symbol,
                filter.filter_type()
            );

            // 각 필터 적용 결과 확인
            match Self::check_filter(symbol, filter, candles) {
                Ok(true) => {
                    // 필터 통과, 다음 필터로 진행
                    log::debug!("코인 {} 필터 {} 통과", symbol, filter.filter_type());
                    continue;
                }
                Ok(false) => {
                    // 필터 실패, 즉시 false 반환
                    log::debug!("코인 {} 필터 {} 실패", symbol, filter.filter_type());
                    return Ok(false);
                }
                Err(e) => {
                    // 에러 발생, 로그 기록 후 false 반환
                    log::warn!(
                        "코인 {} 필터 {} 적용 중 오류: {}",
                        symbol,
                        filter.filter_type(),
                        e
                    );
                    return Ok(false);
                }
            }
        }

        // 모든 필터 통과
        log::debug!("코인 {symbol} 모든 필터 통과");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RSI 필터 생성 유틸리티 함수
    pub fn create_rsi_filter(
        period: usize,
        oversold: f64,
        overbought: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::RSI(RSIParams {
            period,
            oversold,
            overbought,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    /// MACD 필터 생성 유틸리티 함수
    pub fn create_macd_filter(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        filter_type: i32,
        consecutive_n: usize,
        threshold: f64,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::MACD(MACDParams {
            fast_period,
            slow_period,
            signal_period,
            filter_type: filter_type.into(),
            consecutive_n,
            threshold,
            p: 0,
            ..Default::default()
        })
    }

    /// 볼린저 밴드 필터 생성 유틸리티 함수
    pub fn create_bollinger_band_filter(
        period: usize,
        dev_mult: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::BollingerBand(BollingerBandParams {
            period,
            dev_mult,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    /// ADX 필터 생성 유틸리티 함수
    pub fn create_adx_filter(
        period: usize,
        threshold: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::ADX(ADXParams {
            period,
            threshold,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
        })
    }

    /// 이동평균선 필터 생성 유틸리티 함수
    pub fn create_moving_average_filter(
        periods: Vec<usize>,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::MovingAverage(MovingAverageParams {
            periods,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    /// 이치모쿠 필터 생성 유틸리티 함수
    #[allow(dead_code)]
    pub fn create_ichimoku_filter(
        tenkan_period: usize,
        kijun_period: usize,
        senkou_span_b_period: usize,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::Ichimoku(IchimokuParams {
            tenkan_period,
            kijun_period,
            senkou_span_b_period,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
        })
    }

    /// VWAP 필터 생성 유틸리티 함수
    #[allow(dead_code)]
    pub fn create_vwap_filter(
        period: usize,
        filter_type: i32,
        consecutive_n: usize,
        threshold: f64,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::VWAP(VWAPParams {
            period,
            filter_type: filter_type.into(),
            consecutive_n,
            threshold,
            p: 0,
        })
    }

    /// CopyS 필터 생성 유틸리티 함수
    pub fn create_copys_filter(
        rsi_period: usize,
        rsi_upper: f64,
        rsi_lower: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::Copys(CopysParams {
            rsi_period,
            rsi_upper,
            rsi_lower,
            filter_type: filter_type.into(),
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    #[test]
    fn test_technical_filter_config() {
        // RSI 필터 생성 테스트
        let rsi_filter = create_rsi_filter(14, 30.0, 70.0, 0, 1);
        assert_eq!(rsi_filter.filter_type(), TechnicalFilterType::RSI);
        if let TechnicalFilterConfig::RSI(params) = rsi_filter {
            assert_eq!(params.period, 14);
            assert_eq!(params.oversold, 30.0);
            assert_eq!(params.overbought, 70.0);
            assert_eq!(params.filter_type, RSIFilterType::Overbought);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("잘못된 필터 타입");
        }

        // MACD 필터 생성 테스트
        let macd_filter = create_macd_filter(12, 26, 9, 0, 1, 0.0);
        assert_eq!(macd_filter.filter_type(), TechnicalFilterType::MACD);

        // 볼린저 밴드 필터 생성 테스트
        let bb_filter = create_bollinger_band_filter(20, 2.0, 1, 1);
        assert_eq!(bb_filter.filter_type(), TechnicalFilterType::BollingerBand);

        // CopyS 필터 생성 테스트
        let copys_filter = create_copys_filter(14, 70.0, 30.0, 0, 1);
        assert_eq!(copys_filter.filter_type(), TechnicalFilterType::Copys);
        if let TechnicalFilterConfig::Copys(params) = copys_filter {
            assert_eq!(params.rsi_period, 14);
            assert_eq!(params.rsi_upper, 70.0);
            assert_eq!(params.rsi_lower, 30.0);
            assert_eq!(params.filter_type, CopysFilterType::BasicBuySignal);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("잘못된 필터 타입");
        }

        // filter_list 사용 예시 테스트
        test_example_filter_usage();
    }

    // 실제 필터 사용 예시를 보여주는 함수
    fn test_example_filter_usage() {
        // 빌더 함수를 사용한 필터 생성
        let filter_list = [
            // RSI 과매수 필터 (RSI > 70인 코인 제외)
            create_rsi_filter(14, 30.0, 70.0, 0, 1),
            // 이동평균선 필터 (5일선이 20일선 위에 있을 때)
            create_moving_average_filter(vec![5, 20], 3, 3),
            // MACD 필터 (MACD가 시그널선 위에 있는 코인만 포함)
            create_macd_filter(12, 26, 9, 0, 2, 0.0),
            // ADX 필터 (추세가 강한 코인만 포함)
            create_adx_filter(14, 25.0, 1, 1),
        ];

        // filter_list 검증
        assert_eq!(filter_list.len(), 4);
        assert_eq!(filter_list[0].filter_type(), TechnicalFilterType::RSI);
        assert_eq!(
            filter_list[1].filter_type(),
            TechnicalFilterType::MovingAverage
        );
        assert_eq!(filter_list[2].filter_type(), TechnicalFilterType::MACD);
        assert_eq!(filter_list[3].filter_type(), TechnicalFilterType::ADX);
    }

    #[test]
    fn test_filter_parameter_validation() {
        // RSI 필터 파라미터 검증
        let rsi_filter = create_rsi_filter(14, 30.0, 70.0, 0, 1);
        if let TechnicalFilterConfig::RSI(params) = rsi_filter {
            assert_eq!(params.period, 14);
            assert_eq!(params.oversold, 30.0);
            assert_eq!(params.overbought, 70.0);
            assert_eq!(params.filter_type, RSIFilterType::Overbought);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("RSI 필터 파라미터 검증 실패");
        }

        // MACD 필터 파라미터 검증
        let macd_filter = create_macd_filter(12, 26, 9, 0, 1, 0.0);
        if let TechnicalFilterConfig::MACD(params) = macd_filter {
            assert_eq!(params.fast_period, 12);
            assert_eq!(params.slow_period, 26);
            assert_eq!(params.signal_period, 9);
            assert_eq!(params.filter_type, MACDFilterType::MacdAboveSignal);
            assert_eq!(params.consecutive_n, 1);
            assert_eq!(params.threshold, 0.0);
        } else {
            panic!("MACD 필터 파라미터 검증 실패");
        }

        // 이동평균선 필터 파라미터 검증
        let ma_filter = create_moving_average_filter(vec![5, 20], 3, 1);
        if let TechnicalFilterConfig::MovingAverage(params) = ma_filter {
            assert_eq!(params.periods, vec![5, 20]);
            assert_eq!(
                params.filter_type,
                MovingAverageFilterType::FirstMAAboveLastMA
            );
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("이동평균선 필터 파라미터 검증 실패");
        }
    }

    #[test]
    fn test_filter_combination() {
        // 여러 필터 조합 테스트
        let filters = [
            create_rsi_filter(14, 30.0, 70.0, 0, 1),
            create_macd_filter(12, 26, 9, 0, 1, 0.0),
            create_moving_average_filter(vec![5, 20], 3, 1),
            create_copys_filter(14, 70.0, 30.0, 0, 1),
        ];

        assert_eq!(filters.len(), 4);
        assert_eq!(filters[0].filter_type(), TechnicalFilterType::RSI);
        assert_eq!(filters[1].filter_type(), TechnicalFilterType::MACD);
        assert_eq!(filters[2].filter_type(), TechnicalFilterType::MovingAverage);
        assert_eq!(filters[3].filter_type(), TechnicalFilterType::Copys);
    }

    #[test]
    fn test_default_parameters() {
        // 기본값 검증
        let rsi_params = RSIParams::default();
        assert_eq!(rsi_params.period, 14);
        assert_eq!(rsi_params.oversold, 30.0);
        assert_eq!(rsi_params.overbought, 70.0);
        assert_eq!(rsi_params.filter_type, RSIFilterType::Overbought);
        assert_eq!(rsi_params.consecutive_n, 1);

        let macd_params = MACDParams::default();
        assert_eq!(macd_params.fast_period, 12);
        assert_eq!(macd_params.slow_period, 26);
        assert_eq!(macd_params.signal_period, 9);
        assert_eq!(macd_params.filter_type, MACDFilterType::MacdAboveSignal);
        assert_eq!(macd_params.consecutive_n, 1);
        assert_eq!(macd_params.threshold, 0.0);

        let ma_params = MovingAverageParams::default();
        assert_eq!(ma_params.periods, vec![5, 20]);
        assert_eq!(
            ma_params.filter_type,
            MovingAverageFilterType::PriceAboveFirstMA
        );
        assert_eq!(ma_params.consecutive_n, 1);
    }

    #[test]
    fn test_copys_filter_usage() {
        // CopyS 필터 사용 예시
        let copys_filters = [
            // CopyS 매수 신호 필터
            create_copys_filter(14, 70.0, 30.0, 0, 2),
            // CopyS 매도 신호 필터
            create_copys_filter(14, 70.0, 30.0, 1, 1),
            // CopyS MA 정배열 필터
            create_copys_filter(14, 70.0, 30.0, 2, 1),
        ];

        assert_eq!(copys_filters.len(), 3);
        assert_eq!(copys_filters[0].filter_type(), TechnicalFilterType::Copys);

        // 첫 번째 필터 파라미터 검증
        if let TechnicalFilterConfig::Copys(params) = &copys_filters[0] {
            assert_eq!(params.filter_type, CopysFilterType::BasicBuySignal);
            assert_eq!(params.consecutive_n, 2);
        } else {
            panic!("잘못된 필터 타입");
        }

        // 두 번째 필터 파라미터 검증
        if let TechnicalFilterConfig::Copys(params) = &copys_filters[1] {
            assert_eq!(params.filter_type, CopysFilterType::BasicSellSignal);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("잘못된 필터 타입");
        }
    }
}
