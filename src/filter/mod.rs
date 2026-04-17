use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use trading_chart::Candle;

pub type Result<T> = std::result::Result<T, FilterError>;

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("{param_name} 파라미터 오류: period는 0보다 커야 합니다")]
    InvalidPeriod { param_name: String },
    #[error("ADX 파라미터 오류: threshold는 0에서 100 사이여야 합니다")]
    InvalidAdxThreshold,
    #[error("{param_name} 파라미터 오류: threshold는 0에서 100 사이여야 합니다")]
    InvalidPercentageThreshold { param_name: String },
    #[error("{param_name} 파라미터 오류: threshold는 0에서 1 사이여야 합니다")]
    InvalidRatioThreshold { param_name: String },
    #[error("{param_name} 파라미터 오류: consecutive_n은 0보다 커야 합니다")]
    InvalidConsecutiveN { param_name: String },
    #[error("SupportResistance 파라미터 오류: min_touch_count는 0보다 커야 합니다")]
    InvalidSupportResistanceMinTouchCount,
    #[error("CandlePattern 파라미터 오류: pattern_history_length는 0보다 커야 합니다")]
    InvalidCandlePatternHistoryLength,
    #[error("PriceReferenceGap 파라미터 오류: 지원하지 않는 이동평균 타입입니다: {ma_type}")]
    UnsupportedPriceReferenceGapMaType { ma_type: String },
    #[error("알 수 없는 필터 타입: {input}")]
    UnknownTechnicalFilterType { input: String },
    #[error("알 수 없는 RSI 필터 타입: {input}")]
    UnknownRsiFilterType { input: String },
    #[error("알 수 없는 MACD 필터 타입: {input}")]
    UnknownMacdFilterType { input: String },
    #[error("알 수 없는 BollingerBand 필터 타입: {input}")]
    UnknownBollingerBandFilterType { input: String },
    #[error("알 수 없는 ADX 필터 타입: {input}")]
    UnknownAdxFilterType { input: String },
    #[error("알 수 없는 MovingAverage 필터 타입: {input}")]
    UnknownMovingAverageFilterType { input: String },
    #[error("알 수 없는 Ichimoku 필터 타입: {input}")]
    UnknownIchimokuFilterType { input: String },
    #[error("알 수 없는 VWAP 필터 타입: {input}")]
    UnknownVwapFilterType { input: String },
    #[error("알 수 없는 PriceReferenceGap 필터 타입: {input}")]
    UnknownPriceReferenceGapFilterType { input: String },
    #[error("알 수 없는 Copys 필터 타입: {input}")]
    UnknownCopysFilterType { input: String },
    #[error("알 수 없는 ATR 필터 타입: {input}")]
    UnknownAtrFilterType { input: String },
    #[error("알 수 없는 SuperTrend 필터 타입: {input}")]
    UnknownSuperTrendFilterType { input: String },
    #[error("알 수 없는 Volume 필터 타입: {input}")]
    UnknownVolumeFilterType { input: String },
    #[error("알 수 없는 ThreeRSI 필터 타입: {input}")]
    UnknownThreeRsiFilterType { input: String },
    #[error("알 수 없는 CandlePattern 필터 타입: {input}")]
    UnknownCandlePatternFilterType { input: String },
    #[error("알 수 없는 SupportResistance 필터 타입: {input}")]
    UnknownSupportResistanceFilterType { input: String },
    #[error("알 수 없는 Momentum 필터 타입: {input}")]
    UnknownMomentumFilterType { input: String },
    #[error("알 수 없는 Slope 필터 타입: {input}")]
    UnknownSlopeFilterType { input: String },
}

// 공통 deserializer 매크로
macro_rules! impl_filter_type_deserialize {
    ($type:ident, $visitor:ident, $name:literal) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct $visitor;

                impl<'de> Visitor<'de> for $visitor {
                    type Value = $type;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("정수 또는 문자열")
                    }

                    fn visit_str<E>(self, value: &str) -> std::result::Result<$type, E>
                    where
                        E: de::Error,
                    {
                        $type::from_str(value)
                            .map_err(|e| E::custom(format!("{} 필터 타입 파싱 오류: {}", $name, e)))
                    }

                    fn visit_i64<E>(self, value: i64) -> std::result::Result<$type, E>
                    where
                        E: de::Error,
                    {
                        let index = usize::try_from(value).map_err(|_| {
                            E::custom(format!("{} 필터 타입 파싱 오류: {}", $name, value))
                        })?;

                        $type::from_index(index)
                            .map_err(|e| E::custom(format!("{} 필터 타입 파싱 오류: {}", $name, e)))
                    }

                    fn visit_u64<E>(self, value: u64) -> std::result::Result<$type, E>
                    where
                        E: de::Error,
                    {
                        let index = usize::try_from(value).map_err(|_| {
                            E::custom(format!("{} 필터 타입 파싱 오류: {}", $name, value))
                        })?;

                        $type::from_index(index)
                            .map_err(|e| E::custom(format!("{} 필터 타입 파싱 오류: {}", $name, e)))
                    }
                }

                deserializer.deserialize_any($visitor)
            }
        }
    };
}

macro_rules! impl_filter_type_display {
    ($($type:ty),+ $(,)?) => {
        $(
            impl fmt::Display for $type {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )+
    };
}

macro_rules! impl_filter_type_fromstr {
    ($type:ident, $error_variant:ident, parse_i32, [$($variant:ident),+ $(,)?]) => {
        impl $type {
            fn from_index(index: usize) -> Result<Self> {
                const VARIANTS: &[$type] = &[$($type::$variant),+];

                VARIANTS
                    .get(index)
                    .copied()
                    .ok_or(FilterError::$error_variant {
                        input: index.to_string(),
                    })
            }
        }

        impl FromStr for $type {
            type Err = FilterError;

            fn from_str(s: &str) -> Result<Self> {
                if let Ok(index) = s.parse::<usize>() {
                    return Self::from_index(index);
                }

                match s {
                    $(stringify!($variant) => Ok($type::$variant),)+
                    _ => Err(FilterError::$error_variant {
                        input: s.to_string(),
                    }),
                }
            }
        }
    };
    ($type:ident, $error_variant:ident, no_parse_i32, [$($variant:ident),+ $(,)?]) => {
        impl $type {
            fn from_index(index: usize) -> Result<Self> {
                const VARIANTS: &[$type] = &[$($type::$variant),+];

                VARIANTS
                    .get(index)
                    .copied()
                    .ok_or(FilterError::$error_variant {
                        input: index.to_string(),
                    })
            }
        }

        impl FromStr for $type {
            type Err = FilterError;

            fn from_str(s: &str) -> Result<Self> {
                if let Ok(index) = s.parse::<usize>() {
                    return Self::from_index(index);
                }

                match s {
                    $(stringify!($variant) => Ok($type::$variant),)+
                    _ => Err(FilterError::$error_variant {
                        input: s.to_string(),
                    }),
                }
            }
        }
    };
}

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
mod price_reference_gap;
mod rsi;
mod slope;
mod supertrend;
mod support_resistance;
mod three_rsi;
mod volume;
mod vwap;

/// 필터 공통 유틸리티 함수
pub mod utils {
    use super::{FilterError, Result};
    use crate::candle_store::CandleStore;
    use trading_chart::Candle;

    /// 캔들 데이터로 CandleStore 생성 (공통 유틸리티)
    pub fn create_candle_store<C: Candle + 'static>(candles: &[C]) -> CandleStore<C> {
        let candles_vec = candles.to_vec();
        CandleStore::new(candles_vec, candles.len() * 2, false)
    }

    /// 기본 파라미터 검증 (period > 0)
    pub fn validate_period(period: usize, param_name: &str) -> Result<()> {
        if period == 0 {
            return Err(FilterError::InvalidPeriod {
                param_name: param_name.to_string(),
            });
        }
        Ok(())
    }

    /// 퍼센트 기준 임계값 검증 (0-100 범위)
    pub fn validate_percentage_threshold(threshold: f64, param_name: &str) -> Result<()> {
        if !(0.0..=100.0).contains(&threshold) {
            return Err(FilterError::InvalidPercentageThreshold {
                param_name: param_name.to_string(),
            });
        }
        Ok(())
    }

    pub fn validate_ratio_threshold(threshold: f64, param_name: &str) -> Result<()> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(FilterError::InvalidRatioThreshold {
                param_name: param_name.to_string(),
            });
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
    PriceReferenceGap,
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
    /// Slope 기반 필터 (기울기)
    Slope,
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
            TechnicalFilterType::PriceReferenceGap => write!(f, "PriceReferenceGap"),
            TechnicalFilterType::Copys => write!(f, "COPYS"),
            TechnicalFilterType::ATR => write!(f, "ATR"),
            TechnicalFilterType::SuperTrend => write!(f, "SuperTrend"),
            TechnicalFilterType::Volume => write!(f, "Volume"),
            TechnicalFilterType::ThreeRSI => write!(f, "ThreeRSI"),
            TechnicalFilterType::CandlePattern => write!(f, "CandlePattern"),
            TechnicalFilterType::SupportResistance => write!(f, "SupportResistance"),
            TechnicalFilterType::Momentum => write!(f, "Momentum"),
            TechnicalFilterType::Slope => write!(f, "Slope"),
        }
    }
}

impl FromStr for TechnicalFilterType {
    type Err = FilterError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "RSI" => Ok(TechnicalFilterType::RSI),
            "MACD" => Ok(TechnicalFilterType::MACD),
            "BOLLINGERBAND" | "BOLLINGER_BAND" => Ok(TechnicalFilterType::BollingerBand),
            "ADX" => Ok(TechnicalFilterType::ADX),
            "MOVINGAVERAGE" | "MOVING_AVERAGE" => Ok(TechnicalFilterType::MovingAverage),
            "ICHIMOKU" => Ok(TechnicalFilterType::Ichimoku),
            "VWAP" => Ok(TechnicalFilterType::VWAP),
            "PRICEREFERENCEGAP" | "PRICE_REFERENCE_GAP" => {
                Ok(TechnicalFilterType::PriceReferenceGap)
            }
            "COPYS" => Ok(TechnicalFilterType::Copys),
            "ATR" => Ok(TechnicalFilterType::ATR),
            "SUPERTREND" => Ok(TechnicalFilterType::SuperTrend),
            "VOLUME" => Ok(TechnicalFilterType::Volume),
            "THREERSI" => Ok(TechnicalFilterType::ThreeRSI),
            "CANDLEPATTERN" => Ok(TechnicalFilterType::CandlePattern),
            "SUPPORTRESISTANCE" => Ok(TechnicalFilterType::SupportResistance),
            "MOMENTUM" => Ok(TechnicalFilterType::Momentum),
            "SLOPE" => Ok(TechnicalFilterType::Slope),
            _ => Err(FilterError::UnknownTechnicalFilterType {
                input: s.to_string(),
            }),
        }
    }
}

/// RSI 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum RSIFilterType {
    Overbought,
    Oversold,
    NormalRange,
    CrossAboveThreshold,
    CrossBelowThreshold,
    CrossAbove,
    CrossBelow,
    RisingTrend,
    FallingTrend,
    Sideways,
    StrongRisingMomentum,
    StrongFallingMomentum,
    NeutralRange,
    Above40,
    Below60,
    Above50,
    Below50,
    Divergence,
    Convergence,
    Stable,
    NeutralTrend,
    Bullish,
    Bearish,
}

impl_filter_type_fromstr!(
    RSIFilterType,
    UnknownRsiFilterType,
    parse_i32,
    [
        Overbought,
        Oversold,
        NormalRange,
        CrossAboveThreshold,
        CrossBelowThreshold,
        CrossAbove,
        CrossBelow,
        RisingTrend,
        FallingTrend,
        Sideways,
        StrongRisingMomentum,
        StrongFallingMomentum,
        NeutralRange,
        Above40,
        Below60,
        Above50,
        Below50,
        Divergence,
        Convergence,
        Stable,
        NeutralTrend,
        Bullish,
        Bearish,
    ]
);

impl_filter_type_deserialize!(RSIFilterType, RSIFilterTypeVisitor, "RSI");

/// MACD 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MACDFilterType {
    MacdAboveSignal,
    MacdBelowSignal,
    SignalCrossAbove,
    SignalCrossBelow,
    HistogramAboveThreshold,
    HistogramBelowThreshold,
    ZeroLineCrossAbove,
    ZeroLineCrossBelow,
    HistogramNegativeTurn,
    HistogramPositiveTurn,
    StrongUptrend,
    StrongDowntrend,
    MacdRising,
    MacdFalling,
    HistogramExpanding,
    HistogramContracting,
    Divergence,
    Convergence,
    Overbought,
    Oversold,
    Sideways,
}

impl_filter_type_fromstr!(
    MACDFilterType,
    UnknownMacdFilterType,
    parse_i32,
    [
        MacdAboveSignal,
        MacdBelowSignal,
        SignalCrossAbove,
        SignalCrossBelow,
        HistogramAboveThreshold,
        HistogramBelowThreshold,
        ZeroLineCrossAbove,
        ZeroLineCrossBelow,
        HistogramNegativeTurn,
        HistogramPositiveTurn,
        StrongUptrend,
        StrongDowntrend,
        MacdRising,
        MacdFalling,
        HistogramExpanding,
        HistogramContracting,
        Divergence,
        Convergence,
        Overbought,
        Oversold,
        Sideways,
    ]
);

impl_filter_type_deserialize!(MACDFilterType, MACDFilterTypeVisitor, "MACD");

/// 볼린저 밴드 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum BollingerBandFilterType {
    AboveUpperBand,
    BelowLowerBand,
    InsideBand,
    OutsideBand,
    AboveMiddleBand,
    BelowMiddleBand,
    BandWidthSufficient,
    BreakThroughLowerBand,
    SqueezeBreakout,
    EnhancedSqueezeBreakout,
    SqueezeState,
    BandWidthNarrowing,
    SqueezeExpansionStart,
    BreakThroughUpperBand,
    BreakThroughLowerBandFromBelow,
    BandWidthExpanding,
    MiddleBandSideways,
    UpperBandSideways,
    LowerBandSideways,
    BandWidthSideways,
    UpperBandTouch,
    LowerBandTouch,
    BandWidthThresholdBreakthrough,
    PriceMovingToUpperFromMiddle,
    PriceMovingToLowerFromMiddle,
    BandConvergenceThenDivergence,
    BandDivergenceThenConvergence,
    PriceMovingToUpperWithinBand,
    PriceMovingToLowerWithinBand,
    LowVolatility,
    HighVolatility,
}

impl_filter_type_fromstr!(
    BollingerBandFilterType,
    UnknownBollingerBandFilterType,
    parse_i32,
    [
        AboveUpperBand,
        BelowLowerBand,
        InsideBand,
        OutsideBand,
        AboveMiddleBand,
        BelowMiddleBand,
        BandWidthSufficient,
        BreakThroughLowerBand,
        SqueezeBreakout,
        EnhancedSqueezeBreakout,
        SqueezeState,
        BandWidthNarrowing,
        SqueezeExpansionStart,
        BreakThroughUpperBand,
        BreakThroughLowerBandFromBelow,
        BandWidthExpanding,
        MiddleBandSideways,
        UpperBandSideways,
        LowerBandSideways,
        BandWidthSideways,
        UpperBandTouch,
        LowerBandTouch,
        BandWidthThresholdBreakthrough,
        PriceMovingToUpperFromMiddle,
        PriceMovingToLowerFromMiddle,
        BandConvergenceThenDivergence,
        BandDivergenceThenConvergence,
        PriceMovingToUpperWithinBand,
        PriceMovingToLowerWithinBand,
        LowVolatility,
        HighVolatility,
    ]
);

impl_filter_type_deserialize!(
    BollingerBandFilterType,
    BollingerBandFilterTypeVisitor,
    "BollingerBand"
);

/// ADX 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ADXFilterType {
    BelowThreshold,
    AboveThreshold,
    PDIAboveMDI,
    MDIAbovePDI,
    StrongUptrend,
    StrongDowntrend,
    ADXRising,
    ADXFalling,
    DIGapExpanding,
    DIGapContracting,
    ExtremeHigh,
    ExtremeLow,
    MiddleLevel,
    PDICrossAboveMDI,
    MDICrossAbovePDI,
    Sideways,
    Surge,
    Crash,
    StrongDirectionality,
    WeakDirectionality,
    TrendStrengthHigherThanDirection,
    ADXHigherThanMDI,
    PDIHigherThanADX,
    MDIHigherThanADX,
    TrendReversalDown,
    TrendReversalUp,
    DICrossover,
    ExtremePDI,
    ExtremeMDI,
    Stable,
    Unstable,
}

impl_filter_type_fromstr!(
    ADXFilterType,
    UnknownAdxFilterType,
    parse_i32,
    [
        BelowThreshold,
        AboveThreshold,
        PDIAboveMDI,
        MDIAbovePDI,
        StrongUptrend,
        StrongDowntrend,
        ADXRising,
        ADXFalling,
        DIGapExpanding,
        DIGapContracting,
        ExtremeHigh,
        ExtremeLow,
        MiddleLevel,
        PDICrossAboveMDI,
        MDICrossAbovePDI,
        Sideways,
        Surge,
        Crash,
        StrongDirectionality,
        WeakDirectionality,
        TrendStrengthHigherThanDirection,
        ADXHigherThanMDI,
        PDIHigherThanADX,
        MDIHigherThanADX,
        TrendReversalDown,
        TrendReversalUp,
        DICrossover,
        ExtremePDI,
        ExtremeMDI,
        Stable,
        Unstable,
    ]
);

impl_filter_type_deserialize!(ADXFilterType, ADXFilterTypeVisitor, "ADX");

/// 이동평균선 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MovingAverageFilterType {
    PriceAboveFirstMA,
    PriceAboveLastMA,
    RegularArrangement,
    FirstMAAboveLastMA,
    FirstMABelowLastMA,
    GoldenCross,
    PriceBetweenMA,
    MAConvergence,
    MADivergence,
    AllMAAbove,
    AllMABelow,
    ReverseArrangement,
    DeadCross,
    MASideways,
    StrongUptrend,
    StrongDowntrend,
    PriceCrossingMA,
    ConvergenceDivergence,
    DivergenceConvergence,
    ParallelMovement,
    NearCrossover,
    PriceBelowFirstMA,
    PriceBelowLastMA,
}

impl_filter_type_fromstr!(
    MovingAverageFilterType,
    UnknownMovingAverageFilterType,
    parse_i32,
    [
        PriceAboveFirstMA,
        PriceAboveLastMA,
        RegularArrangement,
        FirstMAAboveLastMA,
        FirstMABelowLastMA,
        GoldenCross,
        PriceBetweenMA,
        MAConvergence,
        MADivergence,
        AllMAAbove,
        AllMABelow,
        ReverseArrangement,
        DeadCross,
        MASideways,
        StrongUptrend,
        StrongDowntrend,
        PriceCrossingMA,
        ConvergenceDivergence,
        DivergenceConvergence,
        ParallelMovement,
        NearCrossover,
        PriceBelowFirstMA,
        PriceBelowLastMA,
    ]
);

impl_filter_type_deserialize!(
    MovingAverageFilterType,
    MovingAverageFilterTypeVisitor,
    "MovingAverage"
);

/// 이치모쿠 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum IchimokuFilterType {
    PriceAboveCloud,
    PriceBelowCloud,
    TenkanAboveKijun,
    GoldenCross,
    DeadCross,
    CloudBreakoutUp,
    CloudBreakdown,
    BuySignal,
    SellSignal,
    CloudThickening,
    PerfectAlignment,
    PerfectReverseAlignment,
    StrongBuySignal,
}

impl_filter_type_fromstr!(
    IchimokuFilterType,
    UnknownIchimokuFilterType,
    parse_i32,
    [
        PriceAboveCloud,
        PriceBelowCloud,
        TenkanAboveKijun,
        GoldenCross,
        DeadCross,
        CloudBreakoutUp,
        CloudBreakdown,
        BuySignal,
        SellSignal,
        CloudThickening,
        PerfectAlignment,
        PerfectReverseAlignment,
        StrongBuySignal,
    ]
);

impl_filter_type_deserialize!(IchimokuFilterType, IchimokuFilterTypeVisitor, "Ichimoku");

/// VWAP 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum VWAPFilterType {
    PriceAboveVWAP,
    PriceBelowVWAP,
    PriceNearVWAP,
    VWAPBreakoutUp,
    VWAPBreakdown,
    VWAPRebound,
    DivergingFromVWAP,
    ConvergingToVWAP,
    StrongUptrend,
    StrongDowntrend,
    TrendStrengthening,
    TrendWeakening,
}

impl_filter_type_fromstr!(
    VWAPFilterType,
    UnknownVwapFilterType,
    parse_i32,
    [
        PriceAboveVWAP,
        PriceBelowVWAP,
        PriceNearVWAP,
        VWAPBreakoutUp,
        VWAPBreakdown,
        VWAPRebound,
        DivergingFromVWAP,
        ConvergingToVWAP,
        StrongUptrend,
        StrongDowntrend,
        TrendStrengthening,
        TrendWeakening,
    ]
);

impl_filter_type_deserialize!(VWAPFilterType, VWAPFilterTypeVisitor, "VWAP");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PriceReferenceGapFilterType {
    GapAboveThreshold,
    GapBelowThreshold,
    GapAboveReferenceThreshold,
    GapBelowReferenceThreshold,
}

impl_filter_type_fromstr!(
    PriceReferenceGapFilterType,
    UnknownPriceReferenceGapFilterType,
    parse_i32,
    [
        GapAboveThreshold,
        GapBelowThreshold,
        GapAboveReferenceThreshold,
        GapBelowReferenceThreshold,
    ]
);

impl_filter_type_deserialize!(
    PriceReferenceGapFilterType,
    PriceReferenceGapFilterTypeVisitor,
    "PriceReferenceGap"
);

/// CopyS 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum CopysFilterType {
    BasicBuySignal,
    BasicSellSignal,
    RSIOversold,
    RSIOverbought,
    BBandLowerTouch,
    BBandUpperTouch,
    MASupport,
    MAResistance,
    StrongBuySignal,
    StrongSellSignal,
    WeakBuySignal,
    WeakSellSignal,
    RSINeutral,
    BBandInside,
    MARegularArrangement,
    MAReverseArrangement,
}

impl_filter_type_fromstr!(
    CopysFilterType,
    UnknownCopysFilterType,
    parse_i32,
    [
        BasicBuySignal,
        BasicSellSignal,
        RSIOversold,
        RSIOverbought,
        BBandLowerTouch,
        BBandUpperTouch,
        MASupport,
        MAResistance,
        StrongBuySignal,
        StrongSellSignal,
        WeakBuySignal,
        WeakSellSignal,
        RSINeutral,
        BBandInside,
        MARegularArrangement,
        MAReverseArrangement,
    ]
);

impl_filter_type_deserialize!(CopysFilterType, CopysFilterTypeVisitor, "Copys");

/// ATR 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ATRFilterType {
    AboveThreshold,
    VolatilityExpanding,
    VolatilityContracting,
    HighVolatility,
    LowVolatility,
    VolatilityIncreasing,
    VolatilityDecreasing,
}

impl_filter_type_fromstr!(
    ATRFilterType,
    UnknownAtrFilterType,
    parse_i32,
    [
        AboveThreshold,
        VolatilityExpanding,
        VolatilityContracting,
        HighVolatility,
        LowVolatility,
        VolatilityIncreasing,
        VolatilityDecreasing,
    ]
);

impl_filter_type_deserialize!(ATRFilterType, ATRFilterTypeVisitor, "ATR");

/// SuperTrend 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SuperTrendFilterType {
    AllUptrend,
    AllDowntrend,
    PriceAboveSupertrend,
    PriceBelowSupertrend,
    PriceCrossingAbove,
    PriceCrossingBelow,
    TrendChanged,
    Uptrend,
    Downtrend,
}

impl_filter_type_fromstr!(
    SuperTrendFilterType,
    UnknownSuperTrendFilterType,
    parse_i32,
    [
        AllUptrend,
        AllDowntrend,
        PriceAboveSupertrend,
        PriceBelowSupertrend,
        PriceCrossingAbove,
        PriceCrossingBelow,
        TrendChanged,
        Uptrend,
        Downtrend,
    ]
);

impl_filter_type_deserialize!(
    SuperTrendFilterType,
    SuperTrendFilterTypeVisitor,
    "SuperTrend"
);

/// Volume 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum VolumeFilterType {
    VolumeAboveAverage,
    VolumeBelowAverage,
    VolumeSurge,
    VolumeDecline,
    VolumeSignificantlyAbove,
    BullishWithIncreasedVolume,
    BearishWithIncreasedVolume,
    IncreasingVolumeInUptrend,
    DecreasingVolumeInDowntrend,
    VolumeSharpDecline,
    VolumeStable,
    VolumeVolatile,
    BullishWithDecreasedVolume,
    BearishWithDecreasedVolume,
    VolumeDoubleAverage,
    VolumeHalfAverage,
    VolumeConsecutiveIncrease,
    VolumeConsecutiveDecrease,
    VolumeSideways,
    VolumeExtremelyHigh,
    VolumeExtremelyLow,
}

impl_filter_type_fromstr!(
    VolumeFilterType,
    UnknownVolumeFilterType,
    parse_i32,
    [
        VolumeAboveAverage,
        VolumeBelowAverage,
        VolumeSurge,
        VolumeDecline,
        VolumeSignificantlyAbove,
        BullishWithIncreasedVolume,
        BearishWithIncreasedVolume,
        IncreasingVolumeInUptrend,
        DecreasingVolumeInDowntrend,
        VolumeSharpDecline,
        VolumeStable,
        VolumeVolatile,
        BullishWithDecreasedVolume,
        BearishWithDecreasedVolume,
        VolumeDoubleAverage,
        VolumeHalfAverage,
        VolumeConsecutiveIncrease,
        VolumeConsecutiveDecrease,
        VolumeSideways,
        VolumeExtremelyHigh,
        VolumeExtremelyLow,
    ]
);

impl_filter_type_deserialize!(VolumeFilterType, VolumeFilterTypeVisitor, "Volume");

/// ThreeRSI 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ThreeRSIFilterType {
    AllRSILessThan50,
    AllRSIGreaterThan50,
    RSIReverseArrangement,
    RSIRegularArrangement,
    CandleLowBelowMA,
    CandleHighAboveMA,
    ADXGreaterThan20,
    AllRSILessThan30,
    AllRSIGreaterThan70,
    RSIStableRange,
    RSIBullishRange,
    RSIBearishRange,
    RSIOverboughtRange,
    RSIOversoldRange,
    RSICrossAbove,
    RSICrossBelow,
    RSISideways,
    RSIBullishMomentum,
    RSIBearishMomentum,
    RSIDivergence,
    RSIConvergence,
    RSIDoubleBottom,
    RSIDoubleTop,
    RSIOverboughtReversal,
    RSIOversoldReversal,
    RSINeutralTrend,
    RSIExtremeOverbought,
    RSIExtremeOversold,
}

impl_filter_type_fromstr!(
    ThreeRSIFilterType,
    UnknownThreeRsiFilterType,
    parse_i32,
    [
        AllRSILessThan50,
        AllRSIGreaterThan50,
        RSIReverseArrangement,
        RSIRegularArrangement,
        CandleLowBelowMA,
        CandleHighAboveMA,
        ADXGreaterThan20,
        AllRSILessThan30,
        AllRSIGreaterThan70,
        RSIStableRange,
        RSIBullishRange,
        RSIBearishRange,
        RSIOverboughtRange,
        RSIOversoldRange,
        RSICrossAbove,
        RSICrossBelow,
        RSISideways,
        RSIBullishMomentum,
        RSIBearishMomentum,
        RSIDivergence,
        RSIConvergence,
        RSIDoubleBottom,
        RSIDoubleTop,
        RSIOverboughtReversal,
        RSIOversoldReversal,
        RSINeutralTrend,
        RSIExtremeOverbought,
        RSIExtremeOversold,
    ]
);

impl_filter_type_deserialize!(ThreeRSIFilterType, ThreeRSIFilterTypeVisitor, "ThreeRSI");

/// CandlePattern 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum CandlePatternFilterType {
    StrongBullishPattern,
    StrongBearishPattern,
    ReversalPattern,
    ContinuationPattern,
    VolumeConfirmedPattern,
    HighReliabilityPattern,
    ContextAlignedPattern,
    StrongReversalSignal,
    HighConfidenceSignal,
    VolumeConfirmedSignal,
    PatternClusteringSignal,
    HammerPattern,
    ShootingStarPattern,
    DojiPattern,
    SpinningTopPattern,
    MarubozuPattern,
    MorningStarPattern,
    EveningStarPattern,
    EngulfingPattern,
    PiercingPattern,
    DarkCloudPattern,
    HaramiPattern,
    TweezerPattern,
    TriStarPattern,
    AdvanceBlockPattern,
    DeliberanceBlockPattern,
    BreakawayPattern,
    ConcealmentPattern,
    CounterattackPattern,
    DarkCloudCoverPattern,
    RisingWindowPattern,
    FallingWindowPattern,
    HighBreakoutPattern,
    LowBreakoutPattern,
    GapPattern,
    GapFillPattern,
    DoubleBottomPattern,
    DoubleTopPattern,
    TrianglePattern,
    FlagPattern,
    PennantPattern,
}

impl_filter_type_fromstr!(
    CandlePatternFilterType,
    UnknownCandlePatternFilterType,
    parse_i32,
    [
        StrongBullishPattern,
        StrongBearishPattern,
        ReversalPattern,
        ContinuationPattern,
        VolumeConfirmedPattern,
        HighReliabilityPattern,
        ContextAlignedPattern,
        StrongReversalSignal,
        HighConfidenceSignal,
        VolumeConfirmedSignal,
        PatternClusteringSignal,
        HammerPattern,
        ShootingStarPattern,
        DojiPattern,
        SpinningTopPattern,
        MarubozuPattern,
        MorningStarPattern,
        EveningStarPattern,
        EngulfingPattern,
        PiercingPattern,
        DarkCloudPattern,
        HaramiPattern,
        TweezerPattern,
        TriStarPattern,
        AdvanceBlockPattern,
        DeliberanceBlockPattern,
        BreakawayPattern,
        ConcealmentPattern,
        CounterattackPattern,
        DarkCloudCoverPattern,
        RisingWindowPattern,
        FallingWindowPattern,
        HighBreakoutPattern,
        LowBreakoutPattern,
        GapPattern,
        GapFillPattern,
        DoubleBottomPattern,
        DoubleTopPattern,
        TrianglePattern,
        FlagPattern,
        PennantPattern,
    ]
);

impl_filter_type_deserialize!(
    CandlePatternFilterType,
    CandlePatternFilterTypeVisitor,
    "CandlePattern"
);

/// SupportResistance 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SupportResistanceFilterType {
    SupportBreakdown,
    ResistanceBreakout,
    SupportBounce,
    ResistanceRejection,
    NearStrongSupport,
    NearStrongResistance,
    AboveSupport,
    BelowResistance,
    NearSupport,
    NearResistance,
}

impl_filter_type_fromstr!(
    SupportResistanceFilterType,
    UnknownSupportResistanceFilterType,
    parse_i32,
    [
        SupportBreakdown,
        ResistanceBreakout,
        SupportBounce,
        ResistanceRejection,
        NearStrongSupport,
        NearStrongResistance,
        AboveSupport,
        BelowResistance,
        NearSupport,
        NearResistance,
    ]
);

impl_filter_type_deserialize!(
    SupportResistanceFilterType,
    SupportResistanceFilterTypeVisitor,
    "SupportResistance"
);

/// Momentum 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MomentumFilterType {
    StrongPositiveMomentum,
    StrongNegativeMomentum,
    AcceleratingMomentum,
    DeceleratingMomentum,
    Overbought,
    Oversold,
    MomentumDivergence,
    BullishDivergence,
    BearishDivergence,
    PersistentMomentum,
    StableMomentum,
    MomentumReversalSignal,
    MomentumSideways,
    MomentumSurge,
    MomentumCrash,
    MomentumConvergence,
    MomentumDivergencePattern,
    MomentumParallel,
    MomentumCrossover,
    MomentumSupportTest,
    MomentumResistanceTest,
}

impl_filter_type_fromstr!(
    MomentumFilterType,
    UnknownMomentumFilterType,
    parse_i32,
    [
        StrongPositiveMomentum,
        StrongNegativeMomentum,
        AcceleratingMomentum,
        DeceleratingMomentum,
        Overbought,
        Oversold,
        MomentumDivergence,
        BullishDivergence,
        BearishDivergence,
        PersistentMomentum,
        StableMomentum,
        MomentumReversalSignal,
        MomentumSideways,
        MomentumSurge,
        MomentumCrash,
        MomentumConvergence,
        MomentumDivergencePattern,
        MomentumParallel,
        MomentumCrossover,
        MomentumSupportTest,
        MomentumResistanceTest,
    ]
);

impl_filter_type_deserialize!(MomentumFilterType, MomentumFilterTypeVisitor, "Momentum");

/// RSI 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// 횡보 임계값 (변화율, 기본값: 0.02 = 2%)
    pub sideways_threshold: f64,
    /// 강한 모멘텀 임계값 (RSI 변화량, 기본값: 3.0)
    pub momentum_threshold: f64,
    /// 교차 판단 임계값 (기본값: 50.0)
    pub cross_threshold: f64,
}

fn default_rsi_sideways_threshold() -> f64 {
    0.02
}

fn default_rsi_momentum_threshold() -> f64 {
    3.0
}

fn default_rsi_cross_threshold() -> f64 {
    50.0
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
            sideways_threshold: default_rsi_sideways_threshold(),
            momentum_threshold: default_rsi_momentum_threshold(),
            cross_threshold: default_rsi_cross_threshold(),
        }
    }
}

/// MACD 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// 과매수 임계값 (MACD/가격 비율, 기본값: 0.02 = 2%)
    pub overbought_threshold: f64,
    /// 과매도 임계값 (MACD/가격 비율, 기본값: 0.02 = 2%)
    pub oversold_threshold: f64,
    /// 횡보 임계값 (변화율, 기본값: 0.05 = 5%)
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
            overbought_threshold: default_macd_overbought_threshold(),
            oversold_threshold: default_macd_oversold_threshold(),
            sideways_threshold: default_macd_sideways_threshold(),
        }
    }
}

/// 볼린저 밴드 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// 스퀴즈/횡보 임계값 (기본값: 0.02 = 2%)
    pub squeeze_threshold: f64,
    /// 중간 변동성 임계값 (기본값: 0.05 = 5%)
    pub medium_threshold: f64,
    /// 큰 변동성/가격 이동 임계값 (기본값: 0.1 = 10%)
    pub large_threshold: f64,
    /// 스퀴즈 브레이크아웃 확인 기간 (기본값: 5)
    pub squeeze_breakout_period: usize,
    /// 향상된 스퀴즈 브레이크아웃 좁아지는 기간 (기본값: 3)
    pub enhanced_narrowing_period: usize,
    /// 향상된 스퀴즈 브레이크아웃 스퀴즈 기간 (기본값: 2)
    pub enhanced_squeeze_period: usize,
    /// 상단 밴드 터치 임계값 (기본값: 0.99 = 99%)
    pub upper_touch_threshold: f64,
    /// 하단 밴드 터치 임계값 (기본값: 1.01 = 101%)
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
            squeeze_threshold: default_bband_squeeze_threshold(),
            medium_threshold: default_bband_medium_threshold(),
            large_threshold: default_bband_large_threshold(),
            squeeze_breakout_period: default_bband_squeeze_breakout_period(),
            enhanced_narrowing_period: default_bband_enhanced_narrowing_period(),
            enhanced_squeeze_period: default_bband_enhanced_squeeze_period(),
            upper_touch_threshold: default_bband_upper_touch_threshold(),
            lower_touch_threshold: default_bband_lower_touch_threshold(),
        }
    }
}

/// ADX 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
pub struct MovingAverageParams {
    /// 이동평균 기간 목록
    pub periods: Vec<usize>,
    /// 필터 유형
    pub filter_type: MovingAverageFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    pub p: usize,
    /// 횡보 판단 임계값 (기본값: 0.02 = 2%)
    pub sideways_threshold: f64,
    /// 교차점 근처 판단 임계값 (기본값: 0.005 = 0.5%)
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
            sideways_threshold: default_ma_sideways_threshold(),
            crossover_threshold: default_ma_crossover_threshold(),
        }
    }
}

/// 이치모쿠 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum PriceReferenceSource {
    #[serde(rename = "MOVING_AVERAGE")]
    MovingAverage {
        ma_type: crate::indicator::ma::MAType,
        period: usize,
    },
    #[serde(rename = "VWAP")]
    VWAP { period: usize },
    #[serde(rename = "HIGHEST_HIGH")]
    HighestHigh {
        lookback_period: usize,
        #[serde(default = "default_price_reference_include_current_candle")]
        include_current_candle: bool,
    },
    #[serde(rename = "LOWEST_LOW")]
    LowestLow {
        lookback_period: usize,
        #[serde(default = "default_price_reference_include_current_candle")]
        include_current_candle: bool,
    },
}

fn default_price_reference_source() -> PriceReferenceSource {
    PriceReferenceSource::MovingAverage {
        ma_type: crate::indicator::ma::MAType::SMA,
        period: 20,
    }
}

fn default_price_reference_gap_threshold() -> f64 {
    0.02
}

fn default_price_reference_include_current_candle() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PriceReferenceGapParams {
    pub reference_source: PriceReferenceSource,
    pub filter_type: PriceReferenceGapFilterType,
    pub gap_threshold: f64,
    pub consecutive_n: usize,
    pub p: usize,
}

impl Default for PriceReferenceGapParams {
    fn default() -> Self {
        Self {
            reference_source: default_price_reference_source(),
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: default_price_reference_gap_threshold(),
            consecutive_n: 1,
            p: 0,
        }
    }
}

/// CopyS 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// 볼린저밴드 기간 (기본값: 20)
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 배수 (기본값: 2.0)
    pub bband_multiplier: f64,
    /// 이동평균 기간 목록 (기본값: [5, 20, 60, 120, 200, 240])
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
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
}

/// SuperTrend 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
}

/// Volume 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// VolumeStable 필터의 최소 임계값 (기본값: 0.1)
    pub stable_min_threshold: f64,
}

fn default_volume_stable_min_threshold() -> f64 {
    0.1
}

/// ThreeRSI 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
    /// 교차 판단 임계값 (기본값: 50.0)
    pub cross_threshold: f64,
}

/// CandlePattern 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
}

/// SupportResistance 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
    pub p: usize,
}

/// Momentum 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
            bband_period: default_copys_bband_period(),
            bband_multiplier: default_copys_bband_multiplier(),
            ma_periods: default_copys_ma_periods(),
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
            stable_min_threshold: default_volume_stable_min_threshold(),
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
            cross_threshold: default_rsi_cross_threshold(),
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

/// Slope 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SlopeFilterType {
    Upward,
    Downward,
    Sideways,
    StrengthAboveThreshold,
    Accelerating,
    Decelerating,
    StrongUpward,
    StrongDownward,
    HighRSquared,
}

impl_filter_type_fromstr!(
    SlopeFilterType,
    UnknownSlopeFilterType,
    no_parse_i32,
    [
        Upward,
        Downward,
        Sideways,
        StrengthAboveThreshold,
        Accelerating,
        Decelerating,
        StrongUpward,
        StrongDownward,
        HighRSquared,
    ]
);

impl_filter_type_deserialize!(SlopeFilterType, SlopeFilterTypeVisitor, "Slope");

impl_filter_type_display!(
    RSIFilterType,
    MACDFilterType,
    BollingerBandFilterType,
    ADXFilterType,
    MovingAverageFilterType,
    IchimokuFilterType,
    VWAPFilterType,
    PriceReferenceGapFilterType,
    CopysFilterType,
    ATRFilterType,
    SuperTrendFilterType,
    VolumeFilterType,
    ThreeRSIFilterType,
    CandlePatternFilterType,
    SupportResistanceFilterType,
    MomentumFilterType,
    SlopeFilterType,
);

/// Slope 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SlopeParams {
    /// 분석할 지표 타입 설정
    pub indicator_type: crate::analyzer::IndicatorType,
    /// 분석 기간 (기본값: 20)
    pub period: usize,
    /// 필터 유형
    pub filter_type: SlopeFilterType,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    pub p: usize,
    /// 선형 회귀 사용 여부 (기본값: false)
    pub use_linear_regression: Option<bool>,
    /// 기울기 강도 임계값 (기본값: 0.01)
    pub strength_threshold: Option<f64>,
    /// R² 임계값 (기본값: 0.7)
    pub r_squared_threshold: Option<f64>,
    /// 단기 기간 (가속도/감속도 분석용, 기본값: period / 2)
    pub short_period: Option<usize>,
}

impl Default for SlopeParams {
    fn default() -> Self {
        Self {
            indicator_type: crate::analyzer::IndicatorType::ClosePrice,
            period: 20,
            filter_type: SlopeFilterType::Upward,
            consecutive_n: 1,
            p: 0,
            use_linear_regression: None,
            strength_threshold: None,
            r_squared_threshold: None,
            short_period: None,
        }
    }
}

/// 기술적 필터 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum TechnicalFilterConfig {
    /// RSI 필터 설정
    RSI(RSIParams),
    /// MACD 필터 설정
    MACD(MACDParams),
    /// 볼린저 밴드 필터 설정
    #[serde(rename = "BOLLINGER_BAND")]
    BollingerBand(BollingerBandParams),
    /// ADX 필터 설정
    ADX(ADXParams),
    /// 이동평균선 필터 설정
    #[serde(rename = "MOVING_AVERAGE")]
    MovingAverage(MovingAverageParams),
    #[serde(rename = "PRICE_REFERENCE_GAP")]
    PriceReferenceGap(PriceReferenceGapParams),
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
    /// Slope 필터 설정
    #[serde(rename = "SLOPE")]
    Slope(SlopeParams),
}

impl TechnicalFilterConfig {
    pub fn filter_type(&self) -> TechnicalFilterType {
        match self {
            Self::RSI(_) => TechnicalFilterType::RSI,
            Self::MACD(_) => TechnicalFilterType::MACD,
            Self::BollingerBand(_) => TechnicalFilterType::BollingerBand,
            Self::ADX(_) => TechnicalFilterType::ADX,
            Self::MovingAverage(_) => TechnicalFilterType::MovingAverage,
            Self::PriceReferenceGap(_) => TechnicalFilterType::PriceReferenceGap,
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
            Self::Slope(_) => TechnicalFilterType::Slope,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::RSI(params) => {
                utils::validate_period(params.period, "RSI")?;
                utils::validate_percentage_threshold(params.cross_threshold, "RSI cross_threshold")
            }
            Self::MACD(params) => {
                utils::validate_period(params.fast_period, "MACD fast_period")?;
                utils::validate_period(params.slow_period, "MACD slow_period")?;
                utils::validate_period(params.signal_period, "MACD signal_period")
            }
            Self::BollingerBand(params) => utils::validate_period(params.period, "BollingerBand"),
            Self::ADX(params) => {
                utils::validate_period(params.period, "ADX")?;
                if !(0.0..=100.0).contains(&params.threshold) {
                    return Err(FilterError::InvalidAdxThreshold);
                }
                Ok(())
            }
            Self::MovingAverage(params) => {
                for period in &params.periods {
                    utils::validate_period(*period, "MovingAverage")?;
                }
                Ok(())
            }
            Self::PriceReferenceGap(params) => {
                if params.consecutive_n == 0 {
                    return Err(FilterError::InvalidConsecutiveN {
                        param_name: "PriceReferenceGap consecutive_n".to_string(),
                    });
                }

                utils::validate_ratio_threshold(
                    params.gap_threshold,
                    "PriceReferenceGap gap_threshold",
                )?;

                match &params.reference_source {
                    PriceReferenceSource::MovingAverage { ma_type, period } => {
                        utils::validate_period(*period, "PriceReferenceGap moving_average period")?;

                        match ma_type {
                            crate::indicator::ma::MAType::EMA
                            | crate::indicator::ma::MAType::SMA => Ok(()),
                            _ => Err(FilterError::UnsupportedPriceReferenceGapMaType {
                                ma_type: ma_type.to_string(),
                            }),
                        }
                    }
                    PriceReferenceSource::VWAP { period } => {
                        utils::validate_period(*period, "PriceReferenceGap VWAP period")
                    }
                    PriceReferenceSource::HighestHigh {
                        lookback_period, ..
                    } => utils::validate_period(
                        *lookback_period,
                        "PriceReferenceGap highest_high lookback_period",
                    ),
                    PriceReferenceSource::LowestLow {
                        lookback_period, ..
                    } => utils::validate_period(
                        *lookback_period,
                        "PriceReferenceGap lowest_low lookback_period",
                    ),
                }
            }
            Self::Ichimoku(params) => {
                utils::validate_period(params.tenkan_period, "Ichimoku tenkan_period")?;
                utils::validate_period(params.kijun_period, "Ichimoku kijun_period")?;
                utils::validate_period(params.senkou_span_b_period, "Ichimoku senkou_span_b_period")
            }
            Self::VWAP(params) => utils::validate_period(params.period, "VWAP"),
            Self::Copys(params) => {
                utils::validate_period(params.rsi_period, "Copys rsi_period")?;
                utils::validate_period(params.bband_period, "Copys bband_period")?;
                for period in &params.ma_periods {
                    utils::validate_period(*period, "Copys ma_period")?;
                }
                Ok(())
            }
            Self::ATR(params) => utils::validate_period(params.period, "ATR"),
            Self::SuperTrend(params) => utils::validate_period(params.period, "SuperTrend"),
            Self::Volume(params) => utils::validate_period(params.period, "Volume"),
            Self::ThreeRSI(params) => {
                for period in &params.rsi_periods {
                    utils::validate_period(*period, "ThreeRSI rsi_period")?;
                }
                utils::validate_period(params.ma_period, "ThreeRSI ma_period")?;
                utils::validate_period(params.adx_period, "ThreeRSI adx_period")?;
                utils::validate_percentage_threshold(
                    params.cross_threshold,
                    "ThreeRSI cross_threshold",
                )
            }
            Self::CandlePattern(params) => {
                if params.pattern_history_length == 0 {
                    return Err(FilterError::InvalidCandlePatternHistoryLength);
                }
                Ok(())
            }
            Self::SupportResistance(params) => {
                utils::validate_period(
                    params.lookback_period,
                    "SupportResistance lookback_period",
                )?;
                if params.min_touch_count == 0 {
                    return Err(FilterError::InvalidSupportResistanceMinTouchCount);
                }
                Ok(())
            }
            Self::Momentum(params) => {
                utils::validate_period(params.rsi_period, "Momentum rsi_period")?;
                utils::validate_period(params.stoch_period, "Momentum stoch_period")?;
                utils::validate_period(params.williams_period, "Momentum williams_period")?;
                utils::validate_period(params.roc_period, "Momentum roc_period")?;
                utils::validate_period(params.cci_period, "Momentum cci_period")?;
                utils::validate_period(params.momentum_period, "Momentum momentum_period")
            }
            Self::Slope(params) => utils::validate_period(params.period, "Slope"),
        }
    }
}

// Filter functions are now pub(crate) and accessed through TechnicalFilter::matches_filter
pub use ichimoku::IchimokuValues;

/// 기술적 지표 필터링 적용
pub struct TechnicalFilter;

impl TechnicalFilter {
    /// 개별 코인에 대한 기술적 필터 적용
    pub fn matches_filter<C: Candle + 'static>(
        symbol: &str,
        filter: &TechnicalFilterConfig,
        candles: &[C],
    ) -> Result<bool> {
        // CandleStore를 생성하고 내부 matches_filter_internal 사용
        let candle_store = utils::create_candle_store(candles);
        Self::matches_filter_internal(symbol, filter, &candle_store)
    }

    /// 개별 코인에 여러 기술적 필터 적용
    pub fn matches_filters<C: Candle + 'static>(
        symbol: &str,
        filters: &[TechnicalFilterConfig],
        candles: &[C],
    ) -> Result<bool> {
        // CandleStore를 한 번만 생성하여 재사용
        let candle_store = utils::create_candle_store(candles);

        for filter in filters {
            log::debug!(
                "코인 {} 기술적 필터 적용 중: {:?}",
                symbol,
                filter.filter_type()
            );

            // 각 필터 적용 결과 확인 (CandleStore 재사용)
            match Self::matches_filter_internal(symbol, filter, &candle_store) {
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

    /// 개별 코인에 대한 기술적 필터 적용 (내부 헬퍼 함수, CandleStore 재사용)
    fn matches_filter_internal<C: Candle + 'static>(
        symbol: &str,
        filter: &TechnicalFilterConfig,
        candle_store: &crate::candle_store::CandleStore<C>,
    ) -> Result<bool> {
        filter.validate()?;

        match filter {
            TechnicalFilterConfig::RSI(params) => rsi::filter_rsi(symbol, params, candle_store),
            TechnicalFilterConfig::MACD(params) => macd::filter_macd(symbol, params, candle_store),
            TechnicalFilterConfig::BollingerBand(params) => {
                bollinger_band::filter_bollinger_band(symbol, params, candle_store)
            }
            TechnicalFilterConfig::ADX(params) => adx::filter_adx(symbol, params, candle_store),
            TechnicalFilterConfig::MovingAverage(params) => {
                moving_average::filter_moving_average(symbol, params, candle_store)
            }
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                price_reference_gap::filter_price_reference_gap(symbol, params, candle_store)
            }
            TechnicalFilterConfig::Ichimoku(params) => {
                ichimoku::filter_ichimoku(symbol, params, candle_store)
            }
            TechnicalFilterConfig::VWAP(params) => vwap::filter_vwap(symbol, params, candle_store),
            TechnicalFilterConfig::Copys(params) => {
                copys::filter_copys(symbol, params, candle_store)
            }
            TechnicalFilterConfig::ATR(params) => atr::filter_atr(symbol, params, candle_store),
            TechnicalFilterConfig::SuperTrend(params) => {
                supertrend::filter_supertrend(symbol, params, candle_store)
            }
            TechnicalFilterConfig::Volume(params) => {
                volume::filter_volume(symbol, params, candle_store)
            }
            TechnicalFilterConfig::ThreeRSI(params) => {
                let ma_type = match params.ma_type.as_str() {
                    "EMA" => crate::indicator::ma::MAType::EMA,
                    "WMA" => crate::indicator::ma::MAType::WMA,
                    _ => crate::indicator::ma::MAType::SMA,
                };
                three_rsi::filter_three_rsi(symbol, params, candle_store, ma_type)
            }
            TechnicalFilterConfig::CandlePattern(params) => {
                candle_pattern::filter_candle_pattern(symbol, params, candle_store)
            }
            TechnicalFilterConfig::SupportResistance(params) => {
                support_resistance::filter_support_resistance(symbol, params, candle_store)
            }
            TechnicalFilterConfig::Momentum(params) => {
                momentum::filter_momentum(symbol, params, candle_store)
            }
            TechnicalFilterConfig::Slope(params) => {
                slope::filter_slope(symbol, params, candle_store)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn test_candle(timestamp: i64, close: f64, high: f64, low: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1_000.0,
        }
    }

    /// RSI 필터 생성 유틸리티 함수
    pub fn create_rsi_filter(
        period: usize,
        oversold: f64,
        overbought: f64,
        filter_type: RSIFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::RSI(RSIParams {
            period,
            oversold,
            overbought,
            filter_type,
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
        filter_type: MACDFilterType,
        consecutive_n: usize,
        threshold: f64,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::MACD(MACDParams {
            fast_period,
            slow_period,
            signal_period,
            filter_type,
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
        filter_type: BollingerBandFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::BollingerBand(BollingerBandParams {
            period,
            dev_mult,
            filter_type,
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    /// ADX 필터 생성 유틸리티 함수
    pub fn create_adx_filter(
        period: usize,
        threshold: f64,
        filter_type: ADXFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::ADX(ADXParams {
            period,
            threshold,
            filter_type,
            consecutive_n,
            p: 0,
        })
    }

    /// 이동평균선 필터 생성 유틸리티 함수
    pub fn create_moving_average_filter(
        periods: Vec<usize>,
        filter_type: MovingAverageFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::MovingAverage(MovingAverageParams {
            periods,
            filter_type,
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    pub fn create_price_reference_gap_filter(
        reference_source: PriceReferenceSource,
        filter_type: PriceReferenceGapFilterType,
        gap_threshold: f64,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            reference_source,
            filter_type,
            gap_threshold,
            consecutive_n,
            p: 0,
        })
    }

    /// 이치모쿠 필터 생성 유틸리티 함수
    #[allow(dead_code)]
    pub fn create_ichimoku_filter(
        tenkan_period: usize,
        kijun_period: usize,
        senkou_span_b_period: usize,
        filter_type: IchimokuFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::Ichimoku(IchimokuParams {
            tenkan_period,
            kijun_period,
            senkou_span_b_period,
            filter_type,
            consecutive_n,
            p: 0,
        })
    }

    /// VWAP 필터 생성 유틸리티 함수
    #[allow(dead_code)]
    pub fn create_vwap_filter(
        period: usize,
        filter_type: VWAPFilterType,
        consecutive_n: usize,
        threshold: f64,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::VWAP(VWAPParams {
            period,
            filter_type,
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
        filter_type: CopysFilterType,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::Copys(CopysParams {
            rsi_period,
            rsi_upper,
            rsi_lower,
            filter_type,
            consecutive_n,
            p: 0,
            ..Default::default()
        })
    }

    #[test]
    fn test_technical_filter_config() {
        // RSI 필터 생성 테스트
        let rsi_filter = create_rsi_filter(14, 30.0, 70.0, RSIFilterType::Overbought, 1);
        assert_eq!(rsi_filter.filter_type(), TechnicalFilterType::RSI);
        if let TechnicalFilterConfig::RSI(params) = rsi_filter {
            assert_eq!(params.period, 14);
            assert_eq!(params.oversold, 30.0);
            assert_eq!(params.overbought, 70.0);
            assert_eq!(params.cross_threshold, 50.0);
            assert_eq!(params.filter_type, RSIFilterType::Overbought);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("잘못된 필터 타입");
        }

        // MACD 필터 생성 테스트
        let macd_filter = create_macd_filter(12, 26, 9, MACDFilterType::MacdAboveSignal, 1, 0.0);
        assert_eq!(macd_filter.filter_type(), TechnicalFilterType::MACD);

        // 볼린저 밴드 필터 생성 테스트
        let bb_filter =
            create_bollinger_band_filter(20, 2.0, BollingerBandFilterType::BelowLowerBand, 1);
        assert_eq!(bb_filter.filter_type(), TechnicalFilterType::BollingerBand);

        let price_gap_filter = create_price_reference_gap_filter(
            PriceReferenceSource::MovingAverage {
                ma_type: crate::indicator::ma::MAType::EMA,
                period: 20,
            },
            PriceReferenceGapFilterType::GapAboveThreshold,
            0.02,
            1,
        );
        assert_eq!(
            price_gap_filter.filter_type(),
            TechnicalFilterType::PriceReferenceGap
        );

        // CopyS 필터 생성 테스트
        let copys_filter = create_copys_filter(14, 70.0, 30.0, CopysFilterType::BasicBuySignal, 1);
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
            create_rsi_filter(14, 30.0, 70.0, RSIFilterType::Overbought, 1),
            // 이동평균선 필터 (5일선이 20일선 위에 있을 때)
            create_moving_average_filter(
                vec![5, 20],
                MovingAverageFilterType::FirstMAAboveLastMA,
                3,
            ),
            // MACD 필터 (MACD가 시그널선 위에 있는 코인만 포함)
            create_macd_filter(12, 26, 9, MACDFilterType::MacdAboveSignal, 2, 0.0),
            // ADX 필터 (추세가 강한 코인만 포함)
            create_adx_filter(14, 25.0, ADXFilterType::AboveThreshold, 1),
            create_price_reference_gap_filter(
                PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::EMA,
                    period: 20,
                },
                PriceReferenceGapFilterType::GapBelowThreshold,
                0.02,
                1,
            ),
        ];

        // filter_list 검증
        assert_eq!(filter_list.len(), 5);
        assert_eq!(filter_list[0].filter_type(), TechnicalFilterType::RSI);
        assert_eq!(
            filter_list[1].filter_type(),
            TechnicalFilterType::MovingAverage
        );
        assert_eq!(filter_list[2].filter_type(), TechnicalFilterType::MACD);
        assert_eq!(filter_list[3].filter_type(), TechnicalFilterType::ADX);
        assert_eq!(
            filter_list[4].filter_type(),
            TechnicalFilterType::PriceReferenceGap
        );
    }

    #[test]
    fn test_filter_parameter_validation() {
        // RSI 필터 파라미터 검증
        let rsi_filter = create_rsi_filter(14, 30.0, 70.0, RSIFilterType::Overbought, 1);
        if let TechnicalFilterConfig::RSI(params) = rsi_filter {
            assert_eq!(params.period, 14);
            assert_eq!(params.oversold, 30.0);
            assert_eq!(params.overbought, 70.0);
            assert_eq!(params.cross_threshold, 50.0);
            assert_eq!(params.filter_type, RSIFilterType::Overbought);
            assert_eq!(params.consecutive_n, 1);
        } else {
            panic!("RSI 필터 파라미터 검증 실패");
        }

        // MACD 필터 파라미터 검증
        let macd_filter = create_macd_filter(12, 26, 9, MACDFilterType::MacdAboveSignal, 1, 0.0);
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
        let ma_filter = create_moving_average_filter(
            vec![5, 20],
            MovingAverageFilterType::FirstMAAboveLastMA,
            1,
        );
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

        let price_gap_filter = create_price_reference_gap_filter(
            PriceReferenceSource::VWAP { period: 20 },
            PriceReferenceGapFilterType::GapAboveThreshold,
            0.03,
            2,
        );
        if let TechnicalFilterConfig::PriceReferenceGap(params) = price_gap_filter {
            assert_eq!(
                params.reference_source,
                PriceReferenceSource::VWAP { period: 20 }
            );
            assert_eq!(
                params.filter_type,
                PriceReferenceGapFilterType::GapAboveThreshold
            );
            assert_eq!(params.gap_threshold, 0.03);
            assert_eq!(params.consecutive_n, 2);
            assert_eq!(params.p, 0);
        } else {
            panic!("PriceReferenceGap 필터 파라미터 검증 실패");
        }
    }

    #[test]
    fn test_technical_filter_config_validate_rejects_invalid_params() {
        let invalid_rsi = TechnicalFilterConfig::RSI(RSIParams {
            period: 0,
            ..RSIParams::default()
        });
        assert!(invalid_rsi.validate().is_err());

        let invalid_rsi_cross_threshold = TechnicalFilterConfig::RSI(RSIParams {
            cross_threshold: 101.0,
            ..RSIParams::default()
        });
        assert!(invalid_rsi_cross_threshold.validate().is_err());

        let invalid_adx = TechnicalFilterConfig::ADX(ADXParams {
            threshold: 101.0,
            ..ADXParams::default()
        });
        assert!(invalid_adx.validate().is_err());

        let invalid_copys = TechnicalFilterConfig::Copys(CopysParams {
            rsi_period: 0,
            ..CopysParams::default()
        });
        assert!(invalid_copys.validate().is_err());

        let invalid_three_rsi_cross_threshold = TechnicalFilterConfig::ThreeRSI(ThreeRSIParams {
            cross_threshold: -1.0,
            ..ThreeRSIParams::default()
        });
        assert!(invalid_three_rsi_cross_threshold.validate().is_err());

        let invalid_price_gap_threshold =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                gap_threshold: 1.5,
                ..PriceReferenceGapParams::default()
            });
        assert!(invalid_price_gap_threshold.validate().is_err());

        let invalid_price_gap_ma_type =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::WMA,
                    period: 20,
                },
                ..PriceReferenceGapParams::default()
            });
        assert!(invalid_price_gap_ma_type.validate().is_err());
    }

    #[test]
    fn test_filter_combination() {
        // 여러 필터 조합 테스트
        let filters = [
            create_rsi_filter(14, 30.0, 70.0, RSIFilterType::Overbought, 1),
            create_macd_filter(12, 26, 9, MACDFilterType::MacdAboveSignal, 1, 0.0),
            create_moving_average_filter(
                vec![5, 20],
                MovingAverageFilterType::FirstMAAboveLastMA,
                1,
            ),
            create_price_reference_gap_filter(
                PriceReferenceSource::HighestHigh {
                    lookback_period: 10,
                    include_current_candle: true,
                },
                PriceReferenceGapFilterType::GapAboveThreshold,
                0.02,
                1,
            ),
            create_copys_filter(14, 70.0, 30.0, CopysFilterType::BasicBuySignal, 1),
        ];

        assert_eq!(filters.len(), 5);
        assert_eq!(filters[0].filter_type(), TechnicalFilterType::RSI);
        assert_eq!(filters[1].filter_type(), TechnicalFilterType::MACD);
        assert_eq!(filters[2].filter_type(), TechnicalFilterType::MovingAverage);
        assert_eq!(
            filters[3].filter_type(),
            TechnicalFilterType::PriceReferenceGap
        );
        assert_eq!(filters[4].filter_type(), TechnicalFilterType::Copys);
    }

    #[test]
    fn test_default_parameters() {
        // 기본값 검증
        let rsi_params = RSIParams::default();
        assert_eq!(rsi_params.period, 14);
        assert_eq!(rsi_params.oversold, 30.0);
        assert_eq!(rsi_params.overbought, 70.0);
        assert_eq!(rsi_params.cross_threshold, 50.0);
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

        let price_gap_params = PriceReferenceGapParams::default();
        assert_eq!(
            price_gap_params.reference_source,
            PriceReferenceSource::MovingAverage {
                ma_type: crate::indicator::ma::MAType::SMA,
                period: 20,
            }
        );
        assert_eq!(
            price_gap_params.filter_type,
            PriceReferenceGapFilterType::GapAboveThreshold
        );
        assert_eq!(price_gap_params.gap_threshold, 0.02);
        assert_eq!(price_gap_params.consecutive_n, 1);
        assert_eq!(price_gap_params.p, 0);
    }

    #[test]
    fn test_rsi_params_deserialize_uses_defaults_for_missing_fields() {
        let params: RSIParams = serde_json::from_str(r#"{"filter_type":"CrossAbove"}"#).unwrap();

        assert_eq!(params.period, 14);
        assert_eq!(params.oversold, 30.0);
        assert_eq!(params.overbought, 70.0);
        assert_eq!(params.filter_type, RSIFilterType::CrossAbove);
        assert_eq!(params.consecutive_n, 1);
        assert_eq!(params.p, 0);
        assert_eq!(params.sideways_threshold, 0.02);
        assert_eq!(params.momentum_threshold, 3.0);
        assert_eq!(params.cross_threshold, 50.0);
    }

    #[test]
    fn test_macd_params_deserialize_uses_defaults_for_missing_fields() {
        let params: MACDParams =
            serde_json::from_str(r#"{"filter_type":"MacdAboveSignal"}"#).unwrap();

        assert_eq!(params.fast_period, 12);
        assert_eq!(params.slow_period, 26);
        assert_eq!(params.signal_period, 9);
        assert_eq!(params.filter_type, MACDFilterType::MacdAboveSignal);
        assert_eq!(params.consecutive_n, 1);
        assert_eq!(params.threshold, 0.0);
        assert_eq!(params.p, 0);
        assert_eq!(params.overbought_threshold, 0.02);
        assert_eq!(params.oversold_threshold, 0.02);
        assert_eq!(params.sideways_threshold, 0.05);
    }

    #[test]
    fn test_price_reference_gap_params_deserialize_uses_defaults_for_missing_fields() {
        let params: PriceReferenceGapParams = serde_json::from_str(
            r#"{
                "reference_source": {
                    "type": "LOWEST_LOW",
                    "lookback_period": 7
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            params.reference_source,
            PriceReferenceSource::LowestLow {
                lookback_period: 7,
                include_current_candle: true,
            }
        );
        assert_eq!(
            params.filter_type,
            PriceReferenceGapFilterType::GapAboveThreshold
        );
        assert_eq!(params.gap_threshold, 0.02);
        assert_eq!(params.consecutive_n, 1);
        assert_eq!(params.p, 0);
    }

    #[test]
    fn test_three_rsi_params_deserialize_uses_defaults_for_missing_fields() {
        let params: ThreeRSIParams =
            serde_json::from_str(r#"{"filter_type":"RSICrossAbove"}"#).unwrap();

        assert_eq!(params.rsi_periods, vec![7, 14, 21]);
        assert_eq!(params.ma_type, "SMA");
        assert_eq!(params.ma_period, 20);
        assert_eq!(params.adx_period, 14);
        assert_eq!(params.filter_type, ThreeRSIFilterType::RSICrossAbove);
        assert_eq!(params.consecutive_n, 1);
        assert_eq!(params.p, 0);
        assert_eq!(params.cross_threshold, 50.0);
    }

    #[test]
    fn test_rsi_filter_type_deserialize_supports_numeric_values() {
        let params: RSIParams = serde_json::from_str(r#"{"filter_type":0}"#).unwrap();

        assert_eq!(params.filter_type, RSIFilterType::Overbought);
    }

    #[test]
    fn test_slope_filter_type_deserialize_supports_numeric_values() {
        let params: SlopeParams = serde_json::from_str(r#"{"filter_type":3}"#).unwrap();

        assert_eq!(params.filter_type, SlopeFilterType::StrengthAboveThreshold);
    }

    #[test]
    fn test_price_reference_gap_filter_type_deserialize_supports_numeric_values() {
        let params: PriceReferenceGapParams = serde_json::from_str(r#"{"filter_type":1}"#).unwrap();

        assert_eq!(
            params.filter_type,
            PriceReferenceGapFilterType::GapBelowThreshold
        );

        let directional_params: PriceReferenceGapParams =
            serde_json::from_str(r#"{"filter_type":3}"#).unwrap();

        assert_eq!(
            directional_params.filter_type,
            PriceReferenceGapFilterType::GapBelowReferenceThreshold
        );
    }

    #[test]
    fn test_technical_filter_config_deserializes_price_reference_gap_json() {
        let filter: TechnicalFilterConfig = serde_json::from_str(
            r#"{
                "type": "PRICE_REFERENCE_GAP",
                "reference_source": {
                    "type": "MOVING_AVERAGE",
                    "ma_type": "EMA",
                    "period": 12
                },
                "filter_type": "GapAboveThreshold",
                "gap_threshold": 0.05,
                "consecutive_n": 2,
                "p": 1
            }"#,
        )
        .unwrap();

        match filter {
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                assert_eq!(
                    params.reference_source,
                    PriceReferenceSource::MovingAverage {
                        ma_type: crate::indicator::ma::MAType::EMA,
                        period: 12,
                    }
                );
                assert_eq!(
                    params.filter_type,
                    PriceReferenceGapFilterType::GapAboveThreshold
                );
                assert_eq!(params.gap_threshold, 0.05);
                assert_eq!(params.consecutive_n, 2);
                assert_eq!(params.p, 1);
            }
            _ => panic!("잘못된 필터 타입"),
        }
    }

    #[test]
    fn test_technical_filter_config_deserializes_price_reference_gap_toml() {
        let filter: TechnicalFilterConfig = toml::from_str(
            r#"
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowThreshold"
gap_threshold = 0.03
consecutive_n = 1

[reference_source]
type = "HIGHEST_HIGH"
lookback_period = 5
"#,
        )
        .unwrap();

        match filter {
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                assert_eq!(
                    params.reference_source,
                    PriceReferenceSource::HighestHigh {
                        lookback_period: 5,
                        include_current_candle: true,
                    }
                );
                assert_eq!(
                    params.filter_type,
                    PriceReferenceGapFilterType::GapBelowThreshold
                );
                assert_eq!(params.gap_threshold, 0.03);
                assert_eq!(params.consecutive_n, 1);
                assert_eq!(params.p, 0);
            }
            _ => panic!("잘못된 필터 타입"),
        }
    }

    #[test]
    fn test_technical_filter_config_deserializes_price_reference_gap_previous_bars_only_json() {
        let filter: TechnicalFilterConfig = serde_json::from_str(
            r#"{
                "type": "PRICE_REFERENCE_GAP",
                "reference_source": {
                    "type": "LOWEST_LOW",
                    "lookback_period": 5,
                    "include_current_candle": false
                },
                "filter_type": "GapBelowReferenceThreshold",
                "gap_threshold": 0.03,
                "consecutive_n": 1,
                "p": 0
            }"#,
        )
        .unwrap();

        match filter {
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                assert_eq!(
                    params.reference_source,
                    PriceReferenceSource::LowestLow {
                        lookback_period: 5,
                        include_current_candle: false,
                    }
                );
                assert_eq!(
                    params.filter_type,
                    PriceReferenceGapFilterType::GapBelowReferenceThreshold
                );
                assert_eq!(params.gap_threshold, 0.03);
            }
            _ => panic!("잘못된 필터 타입"),
        }
    }

    #[test]
    fn test_technical_filter_config_deserializes_price_reference_gap_vwap_json() {
        let filter: TechnicalFilterConfig = serde_json::from_str(
            r#"{
                "type": "PRICE_REFERENCE_GAP",
                "reference_source": {
                    "type": "VWAP",
                    "period": 14
                },
                "filter_type": "GapAboveThreshold",
                "gap_threshold": 0.02,
                "consecutive_n": 1,
                "p": 0
            }"#,
        )
        .unwrap();

        match filter {
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                assert_eq!(
                    params.reference_source,
                    PriceReferenceSource::VWAP { period: 14 }
                );
                assert_eq!(
                    params.filter_type,
                    PriceReferenceGapFilterType::GapAboveThreshold
                );
                assert_eq!(params.gap_threshold, 0.02);
                assert_eq!(params.consecutive_n, 1);
                assert_eq!(params.p, 0);
            }
            _ => panic!("잘못된 필터 타입"),
        }
    }

    #[test]
    fn test_technical_filter_config_deserializes_price_reference_gap_inline_table_toml() {
        let filter: TechnicalFilterConfig = toml::from_str(
            r#"
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.02
consecutive_n = 1
reference_source = { type = "HIGHEST_HIGH", lookback_period = 20, include_current_candle = false }
"#,
        )
        .unwrap();

        match filter {
            TechnicalFilterConfig::PriceReferenceGap(params) => {
                assert_eq!(
                    params.reference_source,
                    PriceReferenceSource::HighestHigh {
                        lookback_period: 20,
                        include_current_candle: false,
                    }
                );
                assert_eq!(
                    params.filter_type,
                    PriceReferenceGapFilterType::GapAboveReferenceThreshold
                );
                assert_eq!(params.gap_threshold, 0.02);
                assert_eq!(params.consecutive_n, 1);
                assert_eq!(params.p, 0);
            }
            _ => panic!("잘못된 필터 타입"),
        }
    }

    #[test]
    fn test_price_reference_gap_validate_rejects_zero_period_and_negative_gap() {
        let zero_period = TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            reference_source: PriceReferenceSource::VWAP { period: 0 },
            ..PriceReferenceGapParams::default()
        });
        assert!(zero_period.validate().is_err());

        let zero_highest_high_lookback =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::HighestHigh {
                    lookback_period: 0,
                    include_current_candle: false,
                },
                ..PriceReferenceGapParams::default()
            });
        assert!(zero_highest_high_lookback.validate().is_err());

        let zero_lowest_low_lookback =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::LowestLow {
                    lookback_period: 0,
                    include_current_candle: false,
                },
                ..PriceReferenceGapParams::default()
            });
        assert!(zero_lowest_low_lookback.validate().is_err());

        let zero_consecutive_n =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                consecutive_n: 0,
                ..PriceReferenceGapParams::default()
            });
        assert!(zero_consecutive_n.validate().is_err());

        let negative_gap = TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            gap_threshold: -0.01,
            ..PriceReferenceGapParams::default()
        });
        assert!(negative_gap.validate().is_err());
    }

    #[test]
    fn test_price_reference_gap_validate_rejects_wma_when_scope_is_ema_sma_only() {
        let filter = TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: crate::indicator::ma::MAType::WMA,
                period: 20,
            },
            ..PriceReferenceGapParams::default()
        });

        assert!(filter.validate().is_err());
    }

    #[test]
    fn test_technical_filter_matches_price_reference_gap() {
        let candles = vec![
            test_candle(1, 100.0, 101.0, 99.0),
            test_candle(2, 100.0, 101.0, 99.0),
            test_candle(3, 100.0, 101.0, 99.0),
            test_candle(4, 130.0, 131.0, 129.0),
        ];
        let filter = TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: crate::indicator::ma::MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapAboveThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        });

        let result = TechnicalFilter::matches_filter("TEST/USDT", &filter, &candles).unwrap();

        assert!(result);
    }

    #[test]
    fn test_technical_filter_matches_directional_price_reference_gap() {
        let candles = vec![
            test_candle(1, 100.0, 101.0, 99.0),
            test_candle(2, 100.0, 101.0, 99.0),
            test_candle(3, 100.0, 101.0, 99.0),
            test_candle(4, 70.0, 71.0, 69.0),
        ];
        let filter = TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
            reference_source: PriceReferenceSource::MovingAverage {
                ma_type: crate::indicator::ma::MAType::SMA,
                period: 3,
            },
            filter_type: PriceReferenceGapFilterType::GapBelowReferenceThreshold,
            gap_threshold: 0.10,
            consecutive_n: 1,
            p: 0,
        });

        let result = TechnicalFilter::matches_filter("TEST/USDT", &filter, &candles).unwrap();

        assert!(result);
    }

    #[test]
    fn test_technical_filter_matches_zero_threshold_directional_price_reference_gap_includes_equality()
     {
        let candles = vec![
            test_candle(1, 100.0, 101.0, 99.0),
            test_candle(2, 100.0, 101.0, 99.0),
            test_candle(3, 100.0, 101.0, 99.0),
        ];
        let above_or_equal_filter =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::SMA,
                    period: 3,
                },
                filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
                gap_threshold: 0.0,
                consecutive_n: 1,
                p: 0,
            });
        let below_or_equal_filter =
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::SMA,
                    period: 3,
                },
                filter_type: PriceReferenceGapFilterType::GapBelowReferenceThreshold,
                gap_threshold: 0.0,
                consecutive_n: 1,
                p: 0,
            });

        let above_result =
            TechnicalFilter::matches_filter("TEST/USDT", &above_or_equal_filter, &candles).unwrap();
        let below_result =
            TechnicalFilter::matches_filter("TEST/USDT", &below_or_equal_filter, &candles).unwrap();

        assert!(above_result);
        assert!(below_result);
    }

    #[test]
    fn test_technical_filter_matches_multiple_price_reference_gap_filters() {
        let candles = vec![
            test_candle(1, 100.0, 101.0, 99.0),
            test_candle(2, 100.0, 101.0, 99.0),
            test_candle(3, 100.0, 101.0, 99.0),
            test_candle(4, 100.0, 101.0, 99.0),
            test_candle(5, 100.0, 101.0, 99.0),
            test_candle(6, 100.0, 101.0, 99.0),
            test_candle(7, 100.0, 101.0, 99.0),
            test_candle(8, 100.0, 101.0, 99.0),
            test_candle(9, 100.0, 101.0, 99.0),
            test_candle(10, 100.0, 101.0, 99.0),
            test_candle(11, 100.0, 101.0, 99.0),
            test_candle(12, 100.0, 101.0, 99.0),
            test_candle(13, 100.0, 101.0, 99.0),
            test_candle(14, 100.0, 101.0, 99.0),
            test_candle(15, 100.0, 101.0, 99.0),
            test_candle(16, 100.0, 101.0, 99.0),
            test_candle(17, 100.0, 101.0, 99.0),
            test_candle(18, 100.0, 101.0, 99.0),
            test_candle(19, 100.0, 101.0, 99.0),
            test_candle(20, 101.0, 102.0, 100.0),
        ];
        let filters = vec![
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::EMA,
                    period: 20,
                },
                filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
                gap_threshold: 0.0,
                consecutive_n: 1,
                p: 0,
            }),
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::EMA,
                    period: 20,
                },
                filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
                gap_threshold: 0.02,
                consecutive_n: 1,
                p: 0,
            }),
        ];

        let result = TechnicalFilter::matches_filters("TEST/USDT", &filters, &candles).unwrap();

        assert!(result);
    }

    #[test]
    fn test_technical_filter_matches_multiple_price_reference_gap_filters_rejects_out_of_range_gap()
    {
        let candles = vec![
            test_candle(1, 100.0, 101.0, 99.0),
            test_candle(2, 100.0, 101.0, 99.0),
            test_candle(3, 100.0, 101.0, 99.0),
            test_candle(4, 100.0, 101.0, 99.0),
            test_candle(5, 100.0, 101.0, 99.0),
            test_candle(6, 100.0, 101.0, 99.0),
            test_candle(7, 100.0, 101.0, 99.0),
            test_candle(8, 100.0, 101.0, 99.0),
            test_candle(9, 100.0, 101.0, 99.0),
            test_candle(10, 100.0, 101.0, 99.0),
            test_candle(11, 100.0, 101.0, 99.0),
            test_candle(12, 100.0, 101.0, 99.0),
            test_candle(13, 100.0, 101.0, 99.0),
            test_candle(14, 100.0, 101.0, 99.0),
            test_candle(15, 100.0, 101.0, 99.0),
            test_candle(16, 100.0, 101.0, 99.0),
            test_candle(17, 100.0, 101.0, 99.0),
            test_candle(18, 100.0, 101.0, 99.0),
            test_candle(19, 100.0, 101.0, 99.0),
            test_candle(20, 103.0, 104.0, 102.0),
        ];
        let filters = vec![
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::EMA,
                    period: 20,
                },
                filter_type: PriceReferenceGapFilterType::GapAboveReferenceThreshold,
                gap_threshold: 0.0,
                consecutive_n: 1,
                p: 0,
            }),
            TechnicalFilterConfig::PriceReferenceGap(PriceReferenceGapParams {
                reference_source: PriceReferenceSource::MovingAverage {
                    ma_type: crate::indicator::ma::MAType::EMA,
                    period: 20,
                },
                filter_type: PriceReferenceGapFilterType::GapBelowThreshold,
                gap_threshold: 0.02,
                consecutive_n: 1,
                p: 0,
            }),
        ];

        let result = TechnicalFilter::matches_filters("TEST/USDT", &filters, &candles).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_copys_filter_usage() {
        // CopyS 필터 사용 예시
        let copys_filters = [
            // CopyS 매수 신호 필터
            create_copys_filter(14, 70.0, 30.0, CopysFilterType::BasicBuySignal, 2),
            // CopyS 매도 신호 필터
            create_copys_filter(14, 70.0, 30.0, CopysFilterType::BasicSellSignal, 1),
            // CopyS MA 정배열 필터
            create_copys_filter(14, 70.0, 30.0, CopysFilterType::RSIOversold, 1),
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

    #[test]
    fn test_technical_filter_config_rejects_legacy_bb_alias() {
        let err = serde_json::from_str::<TechnicalFilterConfig>(r#"{"type":"BB"}"#)
            .expect_err("legacy BB alias should now be rejected");

        assert!(err.to_string().contains("unknown variant `BB`"));
    }

    #[test]
    fn test_technical_filter_config_rejects_legacy_ma_alias() {
        let err = serde_json::from_str::<TechnicalFilterConfig>(r#"{"type":"MA"}"#)
            .expect_err("legacy MA alias should now be rejected");

        assert!(err.to_string().contains("unknown variant `MA`"));
    }

    #[test]
    fn test_price_reference_source_rejects_legacy_ma_alias() {
        let err = serde_json::from_str::<PriceReferenceGapParams>(
            r#"{
                "reference_source": {
                    "type": "MA",
                    "ma_type": "EMA",
                    "period": 20
                }
            }"#,
        )
        .expect_err("legacy MA reference source alias should now be rejected");

        assert!(err.to_string().contains("unknown variant `MA`"));
    }

    #[test]
    fn test_technical_filter_type_from_str_rejects_legacy_shorthand_aliases() {
        assert!("BB".parse::<TechnicalFilterType>().is_err());
        assert!("MA".parse::<TechnicalFilterType>().is_err());
    }
}
