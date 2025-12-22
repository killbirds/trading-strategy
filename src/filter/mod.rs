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

/// RSI 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIParams {
    /// RSI 계산 기간 (기본값: 14)
    pub period: usize,
    /// 과매도 기준점 (기본값: 30)
    pub oversold: f64,
    /// 과매수 기준점 (기본값: 70)
    pub overbought: f64,
    /// 필터 유형 (0: 과매수, 1: 과매도, 2: 정상 범위, 3: 상향 돌파, 4: 하향 돌파, 5: 50 상향 돌파, 6: 50 하향 돌파, 7: RSI 상승 추세, 8: RSI 하락 추세)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for RSIParams {
    fn default() -> Self {
        Self {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
    /// 필터 유형 (0: MACD > 시그널, 1: MACD < 시그널, 2: 시그널 상향돌파, 3: 시그널 하향돌파, 4: 히스토그램 > 임계값, 5: 히스토그램 < 임계값, 6: 제로라인 상향돌파, 7: 제로라인 하향돌파, 8: 히스토그램 음전환, 9: 히스토그램 양전환, 10: 강한 상승 추세)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 히스토그램 임계값 (기본값: 0)
    pub threshold: f64,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for MACDParams {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            filter_type: 0,
            consecutive_n: 1,
            threshold: 0.0,
            p: 0,
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
    /// 필터 유형 (0: 상단밴드 위, 1: 하단밴드 아래, 2: 밴드 내부, 3: 밴드 외부, 4: 중간밴드 위, 5: 중간밴드 아래, 6: 밴드 폭 충분, 7: 하단밴드 상향돌파, 8: 스퀴즈 돌파, 9: 향상된 스퀴즈 돌파, 10: 스퀴즈 상태, 11: 밴드 폭 좁아짐, 12: 스퀴즈 확장 시작)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for BollingerBandParams {
    fn default() -> Self {
        Self {
            period: 20,
            dev_mult: 2.0,
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
    /// 필터 유형 (0: ADX < 임계값, 1: ADX > 임계값, 2: +DI > -DI, 3: -DI > +DI, 4: ADX > 임계값 & +DI > -DI, 5: ADX > 임계값 & -DI > +DI, 6: ADX 상승, 7: ADX 하락, 8: DI 간격 확대, 9: DI 간격 축소)
    pub filter_type: i32,
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
            filter_type: 0,
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
    /// 필터 유형 (0: 가격 > 첫번째 MA, 1: 가격 > 마지막 MA, 2: 정규 배열, 3: 첫번째 MA > 마지막 MA, 4: 첫번째 MA < 마지막 MA, 5: 골든 크로스, 6: 가격이 첫번째와 마지막 MA 사이, 7: MA 수렴, 8: MA 발산, 9: 모든 MA 위, 10: 모든 MA 아래)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

impl Default for MovingAverageParams {
    fn default() -> Self {
        Self {
            periods: vec![5, 20],
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
    /// 필터 유형 (0: 가격 > 구름, 1: 가격 < 구름, 2: 전환선 > 기준선, 3: 골든 크로스, 4: 데드 크로스, 5: 구름 상향돌파, 6: 구름 하향돌파, 7: 매수 신호, 8: 매도 신호, 9: 구름 두께 증가, 10: 완벽 정렬, 11: 완벽 역배열, 12: 강한 매수 신호)
    pub filter_type: i32,
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
            filter_type: 0,
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
    /// 필터 유형 (0: 가격 > VWAP, 1: 가격 < VWAP, 2: 가격 ≈ VWAP, 3: VWAP 상향돌파, 4: VWAP 하향돌파, 5: VWAP 리바운드, 6: VWAP 간격 확대, 7: VWAP 간격 축소, 8: 강한 상승, 9: 강한 하락, 10: 추세 강화, 11: 추세 약화)
    pub filter_type: i32,
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
            filter_type: 0,
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
    /// 필터 유형 (0: 기본 매수 신호, 1: 기본 매도 신호, 2: RSI 과매도, 3: RSI 과매수, 4: 볼린저밴드 하단 터치, 5: 볼린저밴드 상단 터치, 6: 이평선 지지, 7: 이평선 저항, 8: 강한 매수 신호, 9: 강한 매도 신호, 10: 약한 매수 신호, 11: 약한 매도 신호, 12: RSI 중립대, 13: 볼린저밴드 내부, 14: 이평선 정배열, 15: 이평선 역배열)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
}

/// ATR 필터 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATRParams {
    /// ATR 계산 기간 (기본값: 14)
    pub period: usize,
    /// ATR 임계값 (기본값: 0.01)
    pub threshold: f64,
    /// 필터 유형 (0: ATR이 임계값 이상, 1: 변동성 확장, 2: 변동성 수축, 3: 높은 변동성, 4: 낮은 변동성, 5: 변동성 증가, 6: 변동성 감소)
    pub filter_type: i32,
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
    /// 필터 유형 (0: 모든 설정에서 상승 추세, 1: 모든 설정에서 하락 추세, 2: 가격이 슈퍼트렌드 위에 있음, 3: 가격이 슈퍼트렌드 아래에 있음, 4: 가격이 슈퍼트렌드를 상향 돌파, 5: 가격이 슈퍼트렌드를 하향 돌파, 6: 추세 변경, 7: 특정 설정에서 상승 추세, 8: 특정 설정에서 하락 추세)
    pub filter_type: i32,
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
    /// 필터 유형 (0: 볼륨이 평균 이상, 1: 볼륨이 평균 이하, 2: 볼륨 급등, 3: 볼륨 감소, 4: 볼륨이 현저히 높음, 5: 상승과 함께 볼륨 증가, 6: 하락과 함께 볼륨 증가, 7: 상승 추세에서 볼륨 증가, 8: 하락 추세에서 볼륨 감소)
    pub filter_type: i32,
    /// 연속 캔들 수 (기본값: 1)
    pub consecutive_n: usize,
    /// 과거 시점 확인을 위한 오프셋 (기본값: 0)
    #[serde(default)]
    pub p: usize,
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
    /// 필터 유형 (0: 모든 RSI가 50 미만, 1: 모든 RSI가 50 이상, 2: RSI 역순 배열, 3: RSI 정상 배열, 4: 캔들 저가가 이동평균 아래, 5: 캔들 고가가 이동평균 위, 6: ADX가 20 이상)
    pub filter_type: i32,
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
    /// 필터 유형 (0: 강한 상승 패턴, 1: 강한 하락 패턴, 2: 반전 패턴, 3: 지속 패턴, 4: 볼륨으로 확인된 패턴, 5: 높은 신뢰도 패턴, 6: 컨텍스트에 맞는 패턴, 7: 강한 반전 신호, 8: 높은 신뢰도 신호, 9: 볼륨 확인 신호, 10: 패턴 클러스터링 신호)
    pub filter_type: i32,
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
    /// 필터 유형 (0: 지지선 하향 돌파, 1: 저항선 상향 돌파, 2: 지지선 반등, 3: 저항선 거부, 4: 강한 지지선 근처, 5: 강한 저항선 근처, 6: 지지선 위에 있음, 7: 저항선 아래에 있음, 8: 지지선 근처, 9: 저항선 근처)
    pub filter_type: i32,
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
    /// 필터 유형 (0: 강한 양의 모멘텀, 1: 강한 음의 모멘텀, 2: 가속하는 모멘텀, 3: 감속하는 모멘텀, 4: 과매수 상태, 5: 과매도 상태, 6: 모멘텀 다이버전스, 7: 불리시 다이버전스, 8: 베어리시 다이버전스, 9: 지속적인 모멘텀, 10: 안정적인 모멘텀, 11: 모멘텀 반전 신호)
    pub filter_type: i32,
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
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
        }
    }
}

impl Default for ATRParams {
    fn default() -> Self {
        Self {
            period: 14,
            threshold: 0.01,
            filter_type: 0,
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
            filter_type: 0,
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
            filter_type: 0,
            consecutive_n: 1,
            p: 0,
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
            filter_type: 0,
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
            filter_type: 0,
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
            filter_type: 0,
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
            filter_type: 0,
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
            TechnicalFilterConfig::ADX(params) => filter_adx(symbol, params.clone(), candles),
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
            filter_type,
            consecutive_n,
            p: 0,
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
            filter_type,
            consecutive_n,
            threshold,
            p: 0,
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
            filter_type,
            consecutive_n,
            p: 0,
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
            filter_type,
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
            filter_type,
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
        filter_type: i32,
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
        filter_type: i32,
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
        filter_type: i32,
        consecutive_n: usize,
    ) -> TechnicalFilterConfig {
        TechnicalFilterConfig::Copys(CopysParams {
            rsi_period,
            rsi_upper,
            rsi_lower,
            filter_type,
            consecutive_n,
            p: 0,
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
            assert_eq!(params.filter_type, 0);
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
            assert_eq!(params.filter_type, 0);
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
            assert_eq!(params.filter_type, 0);
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
            assert_eq!(params.filter_type, 0);
            assert_eq!(params.consecutive_n, 1);
            assert_eq!(params.threshold, 0.0);
        } else {
            panic!("MACD 필터 파라미터 검증 실패");
        }

        // 이동평균선 필터 파라미터 검증
        let ma_filter = create_moving_average_filter(vec![5, 20], 3, 1);
        if let TechnicalFilterConfig::MovingAverage(params) = ma_filter {
            assert_eq!(params.periods, vec![5, 20]);
            assert_eq!(params.filter_type, 3);
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
        assert_eq!(rsi_params.filter_type, 0);
        assert_eq!(rsi_params.consecutive_n, 1);

        let macd_params = MACDParams::default();
        assert_eq!(macd_params.fast_period, 12);
        assert_eq!(macd_params.slow_period, 26);
        assert_eq!(macd_params.signal_period, 9);
        assert_eq!(macd_params.filter_type, 0);
        assert_eq!(macd_params.consecutive_n, 1);
        assert_eq!(macd_params.threshold, 0.0);

        let ma_params = MovingAverageParams::default();
        assert_eq!(ma_params.periods, vec![5, 20]);
        assert_eq!(ma_params.filter_type, 0);
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
            assert_eq!(params.filter_type, 0); // 매수 신호
            assert_eq!(params.consecutive_n, 2); // 2개 연속 캔들
        } else {
            panic!("잘못된 필터 타입");
        }

        // 두 번째 필터 파라미터 검증
        if let TechnicalFilterConfig::Copys(params) = &copys_filters[1] {
            assert_eq!(params.filter_type, 1); // 매도 신호
            assert_eq!(params.consecutive_n, 1); // 1개 캔들
        } else {
            panic!("잘못된 필터 타입");
        }
    }
}
