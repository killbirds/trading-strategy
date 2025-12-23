use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 캔들 패턴 타입
#[derive(Debug, Clone, PartialEq)]
pub enum CandlePattern {
    /// 망치 패턴
    Hammer,
    /// 역망치 패턴
    InvertedHammer,
    /// 도지 패턴
    Doji,
    /// 엔걸핑 패턴 (불리시)
    BullishEngulfing,
    /// 엔걸핑 패턴 (베어리시)
    BearishEngulfing,
    /// 피어싱 패턴
    PiercingPattern,
    /// 다크 클라우드 커버
    DarkCloudCover,
    /// 모닝 스타
    MorningStar,
    /// 이브닝 스타
    EveningStar,
    /// 롱 불리시 캔들
    LongBullish,
    /// 롱 베어리시 캔들
    LongBearish,
    /// 일반 캔들
    Normal,
}

/// 가격 추세 타입
#[derive(Debug, Clone, PartialEq)]
pub enum PriceTrend {
    /// 강한 상승 추세
    StrongUptrend,
    /// 약한 상승 추세
    WeakUptrend,
    /// 횡보
    Sideways,
    /// 약한 하락 추세
    WeakDowntrend,
    /// 강한 하락 추세
    StrongDowntrend,
}

/// 스윙 포인트 타입
#[derive(Debug, Clone)]
pub struct SwingPoint {
    /// 스윙 포인트 인덱스
    pub index: usize,
    /// 스윙 포인트 가격
    pub price: f64,
    /// 스윙 타입 (하이/로우)
    pub swing_type: SwingType,
    /// 강도 (주변 캔들 수)
    pub strength: usize,
}

/// 스윙 타입
#[derive(Debug, Clone, PartialEq)]
pub enum SwingType {
    High,
    Low,
}

/// Price Action 분석기 데이터
#[derive(Debug)]
pub struct PriceActionAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 캔들 패턴
    pub candle_pattern: CandlePattern,
    /// 가격 추세
    pub price_trend: PriceTrend,
    /// 최근 스윙 포인트들
    pub swing_points: Vec<SwingPoint>,
    /// 평균 캔들 크기
    pub avg_candle_size: f64,
    /// 현재 캔들 크기
    pub current_candle_size: f64,
    /// 볼륨 가중 평균 가격
    pub vwap: f64,
    /// 가격 모멘텀
    pub momentum: f64,
    /// 캔들 바디 비율
    pub body_ratio: f64,
    /// 윗꼬리 비율
    pub upper_shadow_ratio: f64,
    /// 아래꼬리 비율
    pub lower_shadow_ratio: f64,
}

impl<C: Candle> PriceActionAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        candle_pattern: CandlePattern,
        price_trend: PriceTrend,
        swing_points: Vec<SwingPoint>,
        avg_candle_size: f64,
        current_candle_size: f64,
        vwap: f64,
        momentum: f64,
        body_ratio: f64,
        upper_shadow_ratio: f64,
        lower_shadow_ratio: f64,
    ) -> PriceActionAnalyzerData<C> {
        PriceActionAnalyzerData {
            candle,
            candle_pattern,
            price_trend,
            swing_points,
            avg_candle_size,
            current_candle_size,
            vwap,
            momentum,
            body_ratio,
            upper_shadow_ratio,
            lower_shadow_ratio,
        }
    }

    /// 현재 캔들이 불리시인지 확인
    pub fn is_bullish(&self) -> bool {
        self.candle.close_price() > self.candle.open_price()
    }

    /// 현재 캔들이 베어리시인지 확인
    pub fn is_bearish(&self) -> bool {
        self.candle.close_price() < self.candle.open_price()
    }

    /// 현재 캔들이 도지인지 확인
    pub fn is_doji(&self) -> bool {
        self.candle_pattern == CandlePattern::Doji
    }

    /// 현재 캔들이 높은 볼륨을 가지는지 확인
    pub fn is_high_volume(&self, volume_threshold: f64) -> bool {
        self.candle.volume() > volume_threshold
    }

    /// 현재 캔들이 평균보다 큰지 확인
    pub fn is_large_candle(&self) -> bool {
        self.current_candle_size > self.avg_candle_size * 1.5
    }

    /// 현재 캔들이 평균보다 작은지 확인
    pub fn is_small_candle(&self) -> bool {
        self.current_candle_size < self.avg_candle_size * 0.5
    }

    /// 상승 추세인지 확인
    pub fn is_uptrend(&self) -> bool {
        matches!(
            self.price_trend,
            PriceTrend::StrongUptrend | PriceTrend::WeakUptrend
        )
    }

    /// 하락 추세인지 확인
    pub fn is_downtrend(&self) -> bool {
        matches!(
            self.price_trend,
            PriceTrend::StrongDowntrend | PriceTrend::WeakDowntrend
        )
    }

    /// 횡보인지 확인
    pub fn is_sideways(&self) -> bool {
        self.price_trend == PriceTrend::Sideways
    }

    /// 강한 추세인지 확인
    pub fn is_strong_trend(&self) -> bool {
        matches!(
            self.price_trend,
            PriceTrend::StrongUptrend | PriceTrend::StrongDowntrend
        )
    }

    /// 반전 패턴인지 확인
    pub fn is_reversal_pattern(&self) -> bool {
        matches!(
            self.candle_pattern,
            CandlePattern::Hammer
                | CandlePattern::InvertedHammer
                | CandlePattern::BullishEngulfing
                | CandlePattern::BearishEngulfing
                | CandlePattern::PiercingPattern
                | CandlePattern::DarkCloudCover
                | CandlePattern::MorningStar
                | CandlePattern::EveningStar
        )
    }

    /// 지속 패턴인지 확인
    pub fn is_continuation_pattern(&self) -> bool {
        matches!(
            self.candle_pattern,
            CandlePattern::LongBullish | CandlePattern::LongBearish
        )
    }

    /// 불확실 패턴인지 확인
    pub fn is_indecision_pattern(&self) -> bool {
        matches!(self.candle_pattern, CandlePattern::Doji)
    }

    /// 가격이 VWAP 위에 있는지 확인
    pub fn is_above_vwap(&self) -> bool {
        self.candle.close_price() > self.vwap
    }

    /// 가격이 VWAP 아래에 있는지 확인
    pub fn is_below_vwap(&self) -> bool {
        self.candle.close_price() < self.vwap
    }

    /// 긍정적인 모멘텀인지 확인
    pub fn is_positive_momentum(&self) -> bool {
        self.momentum > 0.0
    }

    /// 부정적인 모멘텀인지 확인
    pub fn is_negative_momentum(&self) -> bool {
        self.momentum < 0.0
    }

    /// 강한 모멘텀인지 확인
    pub fn is_strong_momentum(&self, threshold: f64) -> bool {
        self.momentum.abs() > threshold
    }

    /// 최근 스윙 하이 반환
    pub fn get_recent_swing_high(&self) -> Option<&SwingPoint> {
        self.swing_points
            .iter()
            .find(|point| point.swing_type == SwingType::High)
    }

    /// 최근 스윙 로우 반환
    pub fn get_recent_swing_low(&self) -> Option<&SwingPoint> {
        self.swing_points
            .iter()
            .find(|point| point.swing_type == SwingType::Low)
    }

    /// 상승 구조인지 확인 (Higher Highs, Higher Lows)
    pub fn is_higher_highs_higher_lows(&self) -> bool {
        let highs: Vec<&SwingPoint> = self
            .swing_points
            .iter()
            .filter(|p| p.swing_type == SwingType::High)
            .collect();
        let lows: Vec<&SwingPoint> = self
            .swing_points
            .iter()
            .filter(|p| p.swing_type == SwingType::Low)
            .collect();

        if highs.len() < 2 || lows.len() < 2 {
            return false;
        }

        let higher_highs = highs.windows(2).all(|w| w[0].price > w[1].price);
        let higher_lows = lows.windows(2).all(|w| w[0].price > w[1].price);

        higher_highs && higher_lows
    }

    /// 하락 구조인지 확인 (Lower Highs, Lower Lows)
    pub fn is_lower_highs_lower_lows(&self) -> bool {
        let highs: Vec<&SwingPoint> = self
            .swing_points
            .iter()
            .filter(|p| p.swing_type == SwingType::High)
            .collect();
        let lows: Vec<&SwingPoint> = self
            .swing_points
            .iter()
            .filter(|p| p.swing_type == SwingType::Low)
            .collect();

        if highs.len() < 2 || lows.len() < 2 {
            return false;
        }

        let lower_highs = highs.windows(2).all(|w| w[0].price < w[1].price);
        let lower_lows = lows.windows(2).all(|w| w[0].price < w[1].price);

        lower_highs && lower_lows
    }
}

impl<C: Candle> GetCandle<C> for PriceActionAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for PriceActionAnalyzerData<C> {}

/// Price Action 분석기
#[derive(Debug)]
pub struct PriceActionAnalyzer<C: Candle> {
    /// 분석 데이터 히스토리
    pub items: Vec<PriceActionAnalyzerData<C>>,
    /// 스윙 포인트 식별을 위한 주변 캔들 수
    pub swing_strength: usize,
    /// 추세 분석을 위한 기간
    pub trend_period: usize,
    /// 모멘텀 계산을 위한 기간
    pub momentum_period: usize,
}

impl<C: Candle> Display for PriceActionAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "PriceActionAnalyzer {{ candle: {}, pattern: {:?}, trend: {:?}, momentum: {:.4} }}",
                first.candle, first.candle_pattern, first.price_trend, first.momentum
            )
        } else {
            write!(f, "PriceActionAnalyzer {{ no data }}")
        }
    }
}

impl<C: Candle + Clone + 'static> PriceActionAnalyzer<C> {
    /// 새 Price Action 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        swing_strength: usize,
        trend_period: usize,
        momentum_period: usize,
    ) -> PriceActionAnalyzer<C> {
        let mut analyzer = PriceActionAnalyzer {
            items: Vec::new(),
            swing_strength,
            trend_period,
            momentum_period,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> PriceActionAnalyzer<C> {
        Self::new(storage, 2, 14, 10)
    }

    /// 캔들 패턴 식별
    fn identify_candle_pattern(&self, candles: &[C]) -> CandlePattern {
        let current = match candles.first() {
            Some(c) => c,
            None => return CandlePattern::Normal,
        };
        let body_size = (current.close_price() - current.open_price()).abs();
        let total_size = current.high_price() - current.low_price();
        let upper_shadow = current.high_price() - current.close_price().max(current.open_price());
        let lower_shadow = current.close_price().min(current.open_price()) - current.low_price();

        let body_ratio = if total_size > 0.0 {
            body_size / total_size
        } else {
            0.0
        };
        let upper_shadow_ratio = if total_size > 0.0 {
            upper_shadow / total_size
        } else {
            0.0
        };
        let lower_shadow_ratio = if total_size > 0.0 {
            lower_shadow / total_size
        } else {
            0.0
        };

        // 도지 패턴 - 몸통이 매우 작음
        if body_ratio < 0.1 {
            return CandlePattern::Doji;
        }

        // 망치 패턴 - 긴 아래꼬리, 짧은 위꼬리, 작은 몸통
        if lower_shadow_ratio > 0.6 && upper_shadow_ratio < 0.1 && body_ratio < 0.3 {
            return CandlePattern::Hammer;
        }

        // 역망치 패턴 - 긴 위꼬리, 짧은 아래꼬리, 작은 몸통
        if upper_shadow_ratio > 0.6 && lower_shadow_ratio < 0.1 && body_ratio < 0.3 {
            return CandlePattern::InvertedHammer;
        }

        // 엔걸핑 패턴 확인
        if let Some(previous) = candles.get(1) {
            let current_bullish = current.close_price() > current.open_price();
            let previous_bullish = previous.close_price() > previous.open_price();

            // 불리시 엔걸핑
            if current_bullish
                && !previous_bullish
                && current.open_price() < previous.close_price()
                && current.close_price() > previous.open_price()
            {
                return CandlePattern::BullishEngulfing;
            }

            // 베어리시 엔걸핑
            if !current_bullish
                && previous_bullish
                && current.open_price() > previous.close_price()
                && current.close_price() < previous.open_price()
            {
                return CandlePattern::BearishEngulfing;
            }

            // 피어싱 패턴
            if current_bullish
                && !previous_bullish
                && current.open_price() < previous.close_price()
                && current.close_price() > (previous.open_price() + previous.close_price()) / 2.0
            {
                return CandlePattern::PiercingPattern;
            }

            // 다크 클라우드 커버
            if !current_bullish
                && previous_bullish
                && current.open_price() > previous.close_price()
                && current.close_price() < (previous.open_price() + previous.close_price()) / 2.0
            {
                return CandlePattern::DarkCloudCover;
            }
        }

        // 긴 캔들 패턴
        if body_ratio > 0.7 {
            if current.close_price() > current.open_price() {
                return CandlePattern::LongBullish;
            } else {
                return CandlePattern::LongBearish;
            }
        }

        CandlePattern::Normal
    }

    /// 가격 추세 분석
    fn analyze_price_trend(&self, candles: &[C]) -> PriceTrend {
        if candles.len() < self.trend_period {
            return PriceTrend::Sideways;
        }

        let recent_candles = &candles[..self.trend_period];
        let first_price = recent_candles
            .last()
            .map(|c| c.close_price())
            .unwrap_or(0.0);
        let last_price = recent_candles
            .first()
            .map(|c| c.close_price())
            .unwrap_or(0.0);
        let price_change = (last_price - first_price) / first_price;

        // 가격 변화의 강도에 따라 추세 분류
        if price_change > 0.05 {
            PriceTrend::StrongUptrend
        } else if price_change > 0.02 {
            PriceTrend::WeakUptrend
        } else if price_change < -0.05 {
            PriceTrend::StrongDowntrend
        } else if price_change < -0.02 {
            PriceTrend::WeakDowntrend
        } else {
            PriceTrend::Sideways
        }
    }

    /// 스윙 포인트 식별
    fn identify_swing_points(&self, candles: &[C]) -> Vec<SwingPoint> {
        let mut swing_points = Vec::new();
        let strength = self.swing_strength;

        if candles.len() < strength * 2 + 1 {
            return swing_points;
        }

        for i in strength..candles.len() - strength {
            let current = &candles[i];
            let is_swing_high = (i.saturating_sub(strength)..i)
                .chain((i + 1)..(i + strength + 1).min(candles.len()))
                .all(|j| current.high_price() > candles[j].high_price());

            let is_swing_low = (i.saturating_sub(strength)..i)
                .chain((i + 1)..(i + strength + 1).min(candles.len()))
                .all(|j| current.low_price() < candles[j].low_price());

            if is_swing_high {
                swing_points.push(SwingPoint {
                    index: i,
                    price: current.high_price(),
                    swing_type: SwingType::High,
                    strength,
                });
            }

            if is_swing_low {
                swing_points.push(SwingPoint {
                    index: i,
                    price: current.low_price(),
                    swing_type: SwingType::Low,
                    strength,
                });
            }
        }

        // 최근 스윙 포인트들만 유지 (최대 10개)
        swing_points.sort_by(|a, b| b.index.cmp(&a.index));
        swing_points.truncate(10);

        swing_points
    }

    /// 평균 캔들 크기 계산
    fn calculate_avg_candle_size(&self, candles: &[C]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }

        let total_size: f64 = candles
            .iter()
            .map(|c| (c.high_price() - c.low_price()).abs())
            .sum();
        total_size / candles.len() as f64
    }

    /// VWAP 계산
    fn calculate_vwap(&self, candles: &[C]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }

        let total_volume: f64 = candles.iter().map(|c| c.volume()).sum();
        if total_volume == 0.0 {
            return match candles.first() {
                Some(c) => c.close_price(),
                None => 0.0,
            };
        }

        let vwap: f64 = candles
            .iter()
            .map(|c| {
                let typical_price = (c.high_price() + c.low_price() + c.close_price()) / 3.0;
                typical_price * c.volume()
            })
            .sum();

        vwap / total_volume
    }

    /// 모멘텀 계산
    fn calculate_momentum(&self, candles: &[C]) -> f64 {
        if candles.len() < self.momentum_period {
            return 0.0;
        }

        let current_price = match candles.first() {
            Some(c) => c.close_price(),
            None => return 0.0,
        };
        let past_price = match candles.get(self.momentum_period - 1) {
            Some(c) => c.close_price(),
            None => return 0.0,
        };
        if past_price == 0.0 {
            return 0.0;
        }
        (current_price - past_price) / past_price
    }

    /// 캔들 바디 및 꼬리 비율 계산
    fn calculate_candle_ratios(&self, candle: &C) -> (f64, f64, f64) {
        let body_size = (candle.close_price() - candle.open_price()).abs();
        let total_size = candle.high_price() - candle.low_price();
        let upper_shadow = candle.high_price() - candle.close_price().max(candle.open_price());
        let lower_shadow = candle.close_price().min(candle.open_price()) - candle.low_price();

        if total_size == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let body_ratio = body_size / total_size;
        let upper_shadow_ratio = upper_shadow / total_size;
        let lower_shadow_ratio = lower_shadow / total_size;

        (body_ratio, upper_shadow_ratio, lower_shadow_ratio)
    }

    /// 불리시 반전 신호 확인
    pub fn is_bullish_reversal_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            let is_reversal_pattern = matches!(
                data.candle_pattern,
                CandlePattern::Hammer
                    | CandlePattern::BullishEngulfing
                    | CandlePattern::PiercingPattern
                    | CandlePattern::MorningStar
            );

            is_reversal_pattern && data.is_downtrend() && data.is_positive_momentum()
        } else {
            false
        }
    }

    /// 베어리시 반전 신호 확인
    pub fn is_bearish_reversal_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            let is_reversal_pattern = matches!(
                data.candle_pattern,
                CandlePattern::InvertedHammer
                    | CandlePattern::BearishEngulfing
                    | CandlePattern::DarkCloudCover
                    | CandlePattern::EveningStar
            );

            is_reversal_pattern && data.is_uptrend() && data.is_negative_momentum()
        } else {
            false
        }
    }

    /// 강한 추세 지속 신호 확인
    pub fn is_strong_trend_continuation(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_trend()
                && data.is_continuation_pattern()
                && data.is_large_candle()
                && data.is_high_volume(data.candle.volume() * 1.5)
        } else {
            false
        }
    }

    /// 추세 약화 신호 확인
    pub fn is_trend_weakening(&self) -> bool {
        if self.items.len() < 3 {
            return false;
        }

        let recent_momentum: Vec<f64> = self.items.iter().take(3).map(|d| d.momentum).collect();
        let is_decreasing_momentum = recent_momentum.windows(2).all(|w| w[0].abs() < w[1].abs());

        is_decreasing_momentum
            && self
                .items
                .first()
                .map(|item| item.is_indecision_pattern())
                .unwrap_or(false)
    }

    /// 볼륨 확인 신호
    pub fn is_volume_confirmation(&self, volume_threshold: f64) -> bool {
        if let Some(data) = self.items.first() {
            data.is_high_volume(volume_threshold)
                && (data.is_bullish() && data.is_positive_momentum()
                    || data.is_bearish() && data.is_negative_momentum())
        } else {
            false
        }
    }

    /// 불리시 반전 신호 확인 (n개 연속 불리시 반전 신호, 이전 m개는 아님)
    pub fn is_bullish_reversal_signal_confirmed(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let is_reversal_pattern = matches!(
                    data.candle_pattern,
                    CandlePattern::Hammer
                        | CandlePattern::BullishEngulfing
                        | CandlePattern::PiercingPattern
                        | CandlePattern::MorningStar
                );
                is_reversal_pattern && data.is_downtrend() && data.is_positive_momentum()
            },
            n,
            m,
            p,
        )
    }

    /// 베어리시 반전 신호 확인 (n개 연속 베어리시 반전 신호, 이전 m개는 아님)
    pub fn is_bearish_reversal_signal_confirmed(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let is_reversal_pattern = matches!(
                    data.candle_pattern,
                    CandlePattern::InvertedHammer
                        | CandlePattern::BearishEngulfing
                        | CandlePattern::DarkCloudCover
                        | CandlePattern::EveningStar
                );
                is_reversal_pattern && data.is_uptrend() && data.is_negative_momentum()
            },
            n,
            m,
            p,
        )
    }

    /// 강한 추세 지속 신호 확인 (n개 연속 강한 추세 지속, 이전 m개는 아님)
    pub fn is_strong_trend_continuation_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                data.is_strong_trend() && data.is_continuation_pattern() && data.is_large_candle()
            },
            n,
            m,
            p,
        )
    }

    /// 추세 약화 신호 확인 (n개 연속 추세 약화, 이전 m개는 아님)
    pub fn is_trend_weakening_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_indecision_pattern(), n, m, p)
    }

    /// 볼륨 확인 신호 (n개 연속 볼륨 확인, 이전 m개는 아님)
    pub fn is_volume_confirmation_signal(
        &self,
        n: usize,
        m: usize,
        volume_threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                data.is_high_volume(volume_threshold)
                    && (data.is_bullish() && data.is_positive_momentum()
                        || data.is_bearish() && data.is_negative_momentum())
            },
            n,
            m,
            p,
        )
    }

    /// 상승 추세 신호 확인 (n개 연속 상승 추세, 이전 m개는 아님)
    pub fn is_uptrend_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_uptrend(), n, m, p)
    }

    /// 하락 추세 신호 확인 (n개 연속 하락 추세, 이전 m개는 아님)
    pub fn is_downtrend_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_downtrend(), n, m, p)
    }

    /// 강한 모멘텀 신호 확인 (n개 연속 강한 모멘텀, 이전 m개는 아님)
    pub fn is_strong_momentum_signal(&self, n: usize, m: usize, threshold: f64, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_momentum(threshold), n, m, p)
    }

    /// VWAP 위 신호 확인 (n개 연속 VWAP 위, 이전 m개는 아님)
    pub fn is_above_vwap_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_above_vwap(), n, m, p)
    }

    /// VWAP 아래 신호 확인 (n개 연속 VWAP 아래, 이전 m개는 아님)
    pub fn is_below_vwap_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_below_vwap(), n, m, p)
    }

    /// n개의 연속 데이터에서 상승 추세인지 확인
    pub fn is_uptrend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_uptrend(), n, p)
    }

    /// n개의 연속 데이터에서 하락 추세인지 확인
    pub fn is_downtrend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_downtrend(), n, p)
    }

    /// n개의 연속 데이터에서 강한 추세인지 확인
    pub fn is_strong_trend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_strong_trend(), n, p)
    }

    /// n개의 연속 데이터에서 반전 패턴인지 확인
    pub fn is_reversal_pattern(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_reversal_pattern(), n, p)
    }

    /// n개의 연속 데이터에서 계속 패턴인지 확인
    pub fn is_continuation_pattern(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_continuation_pattern(), n, p)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<PriceActionAnalyzerData<C>, C>
    for PriceActionAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> PriceActionAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = self.trend_period.max(self.momentum_period).max(50);
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 분석 수행
        let candle_pattern = self.identify_candle_pattern(&recent_candles);
        let price_trend = self.analyze_price_trend(&recent_candles);
        let swing_points = self.identify_swing_points(&recent_candles);
        let avg_candle_size = self.calculate_avg_candle_size(&recent_candles);
        let current_candle_size = candle.high_price() - candle.low_price();
        let vwap = self.calculate_vwap(&recent_candles);
        let momentum = self.calculate_momentum(&recent_candles);
        let (body_ratio, upper_shadow_ratio, lower_shadow_ratio) =
            self.calculate_candle_ratios(&candle);

        PriceActionAnalyzerData::new(
            candle,
            candle_pattern,
            price_trend,
            swing_points,
            avg_candle_size,
            current_candle_size,
            vwap,
            momentum,
            body_ratio,
            upper_shadow_ratio,
            lower_shadow_ratio,
        )
    }

    fn datum(&self) -> &Vec<PriceActionAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<PriceActionAnalyzerData<C>> {
        &mut self.items
    }
}
