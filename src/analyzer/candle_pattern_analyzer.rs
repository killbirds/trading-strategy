use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 단일 캔들 패턴 타입
#[derive(Debug, Clone, PartialEq)]
pub enum SingleCandlePattern {
    /// 도지 - 시장 우유부단함
    Doji,
    /// 그레이브스톤 도지 - 상단 반전 신호
    GravestoneDoji,
    /// 드래곤플라이 도지 - 하단 반전 신호
    DragonFlyDoji,
    /// 망치 - 하단 반전 신호
    Hammer,
    /// 행잉맨 - 상단 반전 신호
    HangingMan,
    /// 역망치 - 하단 반전 신호
    InvertedHammer,
    /// 슈팅스타 - 상단 반전 신호
    ShootingStar,
    /// 마리보즈 - 강한 방향성
    Marubozu,
    /// 스피닝 탑 - 우유부단함
    SpinningTop,
    /// 일반 캔들
    Normal,
}

/// 다중 캔들 패턴 타입
#[derive(Debug, Clone, PartialEq)]
pub enum MultiCandlePattern {
    /// 불리시 엔걸핑 - 상승 반전
    BullishEngulfing,
    /// 베어리시 엔걸핑 - 하락 반전
    BearishEngulfing,
    /// 피어싱 패턴 - 상승 반전
    PiercingPattern,
    /// 다크 클라우드 커버 - 하락 반전
    DarkCloudCover,
    /// 모닝 스타 - 상승 반전
    MorningStar,
    /// 이브닝 스타 - 하락 반전
    EveningStar,
    /// 쓰리 화이트 솔저 - 강한 상승
    ThreeWhiteSoldiers,
    /// 쓰리 블랙 크로우 - 강한 하락
    ThreeBlackCrows,
    /// 쓰리 인사이드 업 - 상승 반전
    ThreeInsideUp,
    /// 쓰리 인사이드 다운 - 하락 반전
    ThreeInsideDown,
    /// 하라미 - 추세 약화
    Harami,
    /// 트위저 탑 - 상단 반전
    TweezerTop,
    /// 트위저 바텀 - 하단 반전
    TweezerBottom,
    /// 패턴 없음
    None,
}

/// 패턴 신뢰도 레벨
#[derive(Debug, Clone, PartialEq)]
pub enum PatternReliability {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

/// 패턴 시그널 방향
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSignal {
    StrongBullish,
    Bullish,
    Neutral,
    Bearish,
    StrongBearish,
}

/// 캔들 패턴 분석 결과
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// 단일 캔들 패턴
    pub single_pattern: SingleCandlePattern,
    /// 다중 캔들 패턴
    pub multi_pattern: MultiCandlePattern,
    /// 패턴 신뢰도
    pub reliability: PatternReliability,
    /// 패턴 시그널
    pub signal: PatternSignal,
    /// 신뢰도 점수 (0.0-1.0)
    pub confidence_score: f64,
    /// 패턴 강도 (0.0-1.0)
    pub pattern_strength: f64,
    /// 볼륨 확인 여부
    pub volume_confirmation: bool,
    /// 추세 일치 여부
    pub trend_alignment: bool,
}

/// Candle Pattern 분석기 데이터
#[derive(Debug)]
pub struct CandlePatternAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 패턴 분석 결과
    pub pattern_analysis: PatternAnalysis,
    /// 최근 패턴 히스토리
    pub recent_patterns: Vec<PatternAnalysis>,
    /// 패턴 연속성 점수
    pub pattern_continuity_score: f64,
    /// 시장 컨텍스트 점수
    pub market_context_score: f64,
}

impl<C: Candle> CandlePatternAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        pattern_analysis: PatternAnalysis,
        recent_patterns: Vec<PatternAnalysis>,
        pattern_continuity_score: f64,
        market_context_score: f64,
    ) -> CandlePatternAnalyzerData<C> {
        CandlePatternAnalyzerData {
            candle,
            pattern_analysis,
            recent_patterns,
            pattern_continuity_score,
            market_context_score,
        }
    }

    /// 강한 불리시 패턴인지 확인
    pub fn is_strong_bullish_pattern(&self) -> bool {
        matches!(self.pattern_analysis.signal, PatternSignal::StrongBullish)
            && self.pattern_analysis.confidence_score > 0.7
    }

    /// 강한 베어리시 패턴인지 확인
    pub fn is_strong_bearish_pattern(&self) -> bool {
        matches!(self.pattern_analysis.signal, PatternSignal::StrongBearish)
            && self.pattern_analysis.confidence_score > 0.7
    }

    /// 반전 패턴인지 확인
    pub fn is_reversal_pattern(&self) -> bool {
        matches!(
            self.pattern_analysis.single_pattern,
            SingleCandlePattern::Hammer
                | SingleCandlePattern::InvertedHammer
                | SingleCandlePattern::HangingMan
                | SingleCandlePattern::ShootingStar
                | SingleCandlePattern::GravestoneDoji
                | SingleCandlePattern::DragonFlyDoji
        ) || matches!(
            self.pattern_analysis.multi_pattern,
            MultiCandlePattern::BullishEngulfing
                | MultiCandlePattern::BearishEngulfing
                | MultiCandlePattern::PiercingPattern
                | MultiCandlePattern::DarkCloudCover
                | MultiCandlePattern::MorningStar
                | MultiCandlePattern::EveningStar
                | MultiCandlePattern::ThreeInsideUp
                | MultiCandlePattern::ThreeInsideDown
                | MultiCandlePattern::TweezerTop
                | MultiCandlePattern::TweezerBottom
        )
    }

    /// 지속 패턴인지 확인
    pub fn is_continuation_pattern(&self) -> bool {
        matches!(
            self.pattern_analysis.multi_pattern,
            MultiCandlePattern::ThreeWhiteSoldiers | MultiCandlePattern::ThreeBlackCrows
        ) || (matches!(
            self.pattern_analysis.single_pattern,
            SingleCandlePattern::Marubozu
        ) && self.pattern_analysis.pattern_strength > 0.6)
    }

    /// 볼륨 확인된 패턴인지 확인
    pub fn is_volume_confirmed_pattern(&self) -> bool {
        self.pattern_analysis.volume_confirmation && self.pattern_analysis.confidence_score > 0.6
    }

    /// 패턴 신뢰도가 높은지 확인
    pub fn is_high_reliability_pattern(&self) -> bool {
        matches!(
            self.pattern_analysis.reliability,
            PatternReliability::High | PatternReliability::VeryHigh
        )
    }

    /// 시장 컨텍스트와 일치하는 패턴인지 확인
    pub fn is_context_aligned_pattern(&self) -> bool {
        self.pattern_analysis.trend_alignment && self.market_context_score > 0.6
    }

    /// 패턴 클러스터링 점수 계산
    pub fn calculate_pattern_clustering_score(&self) -> f64 {
        let similar_patterns = self
            .recent_patterns
            .iter()
            .filter(|p| p.signal == self.pattern_analysis.signal)
            .count();

        (similar_patterns as f64 / self.recent_patterns.len().max(1) as f64).min(1.0)
    }
}

impl<C: Candle> GetCandle<C> for CandlePatternAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for CandlePatternAnalyzerData<C> {}

impl<C: Candle> Display for CandlePatternAnalyzerData<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "캔들: {}, 패턴: {:?}, 신뢰도: {:.2}",
            self.candle,
            self.pattern_analysis.multi_pattern,
            self.pattern_analysis.confidence_score
        )
    }
}

/// Candle Pattern 분석기
#[derive(Debug)]
pub struct CandlePatternAnalyzer<C: Candle> {
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<CandlePatternAnalyzerData<C>>,
    /// 최소 바디 크기 비율
    pub min_body_ratio: f64,
    /// 최소 꼬리 크기 비율
    pub min_shadow_ratio: f64,
    /// 패턴 히스토리 길이
    pub pattern_history_length: usize,
}

impl<C: Candle> Display for CandlePatternAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "{first}"),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + Clone + 'static> CandlePatternAnalyzer<C> {
    /// 새 Candle Pattern 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        min_body_ratio: f64,
        min_shadow_ratio: f64,
        pattern_history_length: usize,
    ) -> CandlePatternAnalyzer<C> {
        let mut analyzer = CandlePatternAnalyzer {
            items: Vec::new(),
            min_body_ratio,
            min_shadow_ratio,
            pattern_history_length,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> CandlePatternAnalyzer<C> {
        Self::new(storage, 0.1, 0.3, 10)
    }

    /// 단일 캔들 패턴 식별
    fn identify_single_candle_pattern(&self, candle: &C) -> SingleCandlePattern {
        let open = candle.open_price();
        let high = candle.high_price();
        let low = candle.low_price();
        let close = candle.close_price();

        let body_size = (close - open).abs();
        let total_range = high - low;
        let upper_shadow = high - close.max(open);
        let lower_shadow = close.min(open) - low;

        if total_range == 0.0 {
            return SingleCandlePattern::Normal;
        }

        let body_ratio = body_size / total_range;
        let upper_shadow_ratio = upper_shadow / total_range;
        let lower_shadow_ratio = lower_shadow / total_range;

        // 도지 패턴들
        if body_ratio < self.min_body_ratio {
            if upper_shadow_ratio > 0.6 && lower_shadow_ratio < 0.1 {
                return SingleCandlePattern::GravestoneDoji;
            }
            if lower_shadow_ratio > 0.6 && upper_shadow_ratio < 0.1 {
                return SingleCandlePattern::DragonFlyDoji;
            }
            return SingleCandlePattern::Doji;
        }

        // 망치 패턴들
        if lower_shadow_ratio > self.min_shadow_ratio * 2.0
            && upper_shadow_ratio < self.min_shadow_ratio
            && body_ratio < 0.3
        {
            if close > open {
                return SingleCandlePattern::Hammer;
            } else {
                return SingleCandlePattern::HangingMan;
            }
        }

        // 역망치 패턴들
        if upper_shadow_ratio > self.min_shadow_ratio * 2.0
            && lower_shadow_ratio < self.min_shadow_ratio
            && body_ratio < 0.3
        {
            if close > open {
                return SingleCandlePattern::InvertedHammer;
            } else {
                return SingleCandlePattern::ShootingStar;
            }
        }

        // 마리보즈 패턴
        if body_ratio > 0.9 && upper_shadow_ratio < 0.05 && lower_shadow_ratio < 0.05 {
            return SingleCandlePattern::Marubozu;
        }

        // 스피닝 탑 패턴
        if body_ratio < 0.3 && upper_shadow_ratio > 0.2 && lower_shadow_ratio > 0.2 {
            return SingleCandlePattern::SpinningTop;
        }

        SingleCandlePattern::Normal
    }

    /// 다중 캔들 패턴 식별
    fn identify_multi_candle_pattern(&self, candles: &[C]) -> MultiCandlePattern {
        if candles.len() < 2 {
            return MultiCandlePattern::None;
        }

        // 2캔들 패턴
        if let (Some(prev), Some(curr)) = (candles.get(1), candles.first()) {
            // 엔걸핑 패턴
            if self.is_engulfing_pattern(prev, curr) {
                if curr.close_price() > curr.open_price() {
                    return MultiCandlePattern::BullishEngulfing;
                } else {
                    return MultiCandlePattern::BearishEngulfing;
                }
            }

            // 피어싱 패턴
            if self.is_piercing_pattern(prev, curr) {
                return MultiCandlePattern::PiercingPattern;
            }

            // 다크 클라우드 커버
            if self.is_dark_cloud_cover(prev, curr) {
                return MultiCandlePattern::DarkCloudCover;
            }

            // 하라미 패턴
            if self.is_harami_pattern(prev, curr) {
                return MultiCandlePattern::Harami;
            }

            // 트위저 패턴
            if self.is_tweezer_pattern(prev, curr) {
                if prev.close_price() < prev.open_price() && curr.close_price() > curr.open_price()
                {
                    return MultiCandlePattern::TweezerBottom;
                } else if prev.close_price() > prev.open_price()
                    && curr.close_price() < curr.open_price()
                {
                    return MultiCandlePattern::TweezerTop;
                }
            }
        }

        // 3캔들 패턴
        if let (Some(third), Some(second), Some(first)) =
            (candles.get(2), candles.get(1), candles.first())
        {
            // 모닝 스타
            if self.is_morning_star_pattern(third, second, first) {
                return MultiCandlePattern::MorningStar;
            }

            // 이브닝 스타
            if self.is_evening_star_pattern(third, second, first) {
                return MultiCandlePattern::EveningStar;
            }

            // 쓰리 화이트 솔저
            if self.is_three_white_soldiers(third, second, first) {
                return MultiCandlePattern::ThreeWhiteSoldiers;
            }

            // 쓰리 블랙 크로우
            if self.is_three_black_crows(third, second, first) {
                return MultiCandlePattern::ThreeBlackCrows;
            }

            // 쓰리 인사이드 업
            if self.is_three_inside_up(third, second, first) {
                return MultiCandlePattern::ThreeInsideUp;
            }

            // 쓰리 인사이드 다운
            if self.is_three_inside_down(third, second, first) {
                return MultiCandlePattern::ThreeInsideDown;
            }
        }

        MultiCandlePattern::None
    }

    /// 엔걸핑 패턴 확인
    fn is_engulfing_pattern(&self, prev: &C, curr: &C) -> bool {
        let prev_body_top = prev.close_price().max(prev.open_price());
        let prev_body_bottom = prev.close_price().min(prev.open_price());
        let curr_body_top = curr.close_price().max(curr.open_price());
        let curr_body_bottom = curr.close_price().min(curr.open_price());

        // 현재 캔들이 이전 캔들의 몸통을 완전히 감싸야 함
        curr_body_top > prev_body_top
            && curr_body_bottom < prev_body_bottom
            && (prev.close_price() > prev.open_price()) != (curr.close_price() > curr.open_price())
    }

    /// 피어싱 패턴 확인
    fn is_piercing_pattern(&self, prev: &C, curr: &C) -> bool {
        prev.close_price() < prev.open_price()
            && curr.close_price() > curr.open_price()
            && curr.open_price() < prev.close_price()
            && curr.close_price() > (prev.open_price() + prev.close_price()) / 2.0
            && curr.close_price() < prev.open_price()
    }

    /// 다크 클라우드 커버 확인
    fn is_dark_cloud_cover(&self, prev: &C, curr: &C) -> bool {
        prev.close_price() > prev.open_price()
            && curr.close_price() < curr.open_price()
            && curr.open_price() > prev.close_price()
            && curr.close_price() < (prev.open_price() + prev.close_price()) / 2.0
            && curr.close_price() > prev.open_price()
    }

    /// 하라미 패턴 확인
    fn is_harami_pattern(&self, prev: &C, curr: &C) -> bool {
        let prev_body_top = prev.close_price().max(prev.open_price());
        let prev_body_bottom = prev.close_price().min(prev.open_price());
        let curr_body_top = curr.close_price().max(curr.open_price());
        let curr_body_bottom = curr.close_price().min(curr.open_price());

        // 현재 캔들이 이전 캔들의 몸통 안에 있어야 함
        curr_body_top < prev_body_top
            && curr_body_bottom > prev_body_bottom
            && (prev.close_price() - prev.open_price()).abs()
                > (curr.close_price() - curr.open_price()).abs()
    }

    /// 트위저 패턴 확인
    fn is_tweezer_pattern(&self, prev: &C, curr: &C) -> bool {
        let high_diff = (prev.high_price() - curr.high_price()).abs();
        let low_diff = (prev.low_price() - curr.low_price()).abs();
        let price_tolerance = (prev.high_price() - prev.low_price()) * 0.01;

        high_diff < price_tolerance || low_diff < price_tolerance
    }

    /// 모닝 스타 패턴 확인
    fn is_morning_star_pattern(&self, first: &C, second: &C, third: &C) -> bool {
        // 첫 번째: 긴 음봉
        let first_bearish = first.close_price() < first.open_price();
        let first_body_size = (first.close_price() - first.open_price()).abs();

        // 두 번째: 작은 몸통 (도지 또는 스피닝 탑)
        let second_body_size = (second.close_price() - second.open_price()).abs();
        let second_small = second_body_size < first_body_size * 0.3;

        // 세 번째: 긴 양봉
        let third_bullish = third.close_price() > third.open_price();
        let third_body_size = (third.close_price() - third.open_price()).abs();

        // 갭 확인
        let gap_down = second.high_price() < first.close_price();
        let gap_up = third.open_price() > second.high_price();

        first_bearish
            && second_small
            && third_bullish
            && gap_down
            && gap_up
            && first_body_size > (first.high_price() - first.low_price()) * 0.6
            && third_body_size > (third.high_price() - third.low_price()) * 0.6
    }

    /// 이브닝 스타 패턴 확인
    fn is_evening_star_pattern(&self, first: &C, second: &C, third: &C) -> bool {
        // 첫 번째: 긴 양봉
        let first_bullish = first.close_price() > first.open_price();
        let first_body_size = (first.close_price() - first.open_price()).abs();

        // 두 번째: 작은 몸통 (도지 또는 스피닝 탑)
        let second_body_size = (second.close_price() - second.open_price()).abs();
        let second_small = second_body_size < first_body_size * 0.3;

        // 세 번째: 긴 음봉
        let third_bearish = third.close_price() < third.open_price();
        let third_body_size = (third.close_price() - third.open_price()).abs();

        // 갭 확인
        let gap_up = second.low_price() > first.close_price();
        let gap_down = third.open_price() < second.low_price();

        first_bullish
            && second_small
            && third_bearish
            && gap_up
            && gap_down
            && first_body_size > (first.high_price() - first.low_price()) * 0.6
            && third_body_size > (third.high_price() - third.low_price()) * 0.6
    }

    /// 쓰리 화이트 솔저 패턴 확인
    fn is_three_white_soldiers(&self, first: &C, second: &C, third: &C) -> bool {
        // 모두 양봉이어야 함
        let all_bullish = first.close_price() > first.open_price()
            && second.close_price() > second.open_price()
            && third.close_price() > third.open_price();

        // 연속적인 상승
        let consecutive_higher = first.close_price() < second.close_price()
            && second.close_price() < third.close_price();

        // 각 캔들의 오픈이 이전 캔들의 몸통 안에 있어야 함
        let proper_opens = second.open_price() > first.open_price()
            && second.open_price() < first.close_price()
            && third.open_price() > second.open_price()
            && third.open_price() < second.close_price();

        all_bullish && consecutive_higher && proper_opens
    }

    /// 쓰리 블랙 크로우 패턴 확인
    fn is_three_black_crows(&self, first: &C, second: &C, third: &C) -> bool {
        // 모두 음봉이어야 함
        let all_bearish = first.close_price() < first.open_price()
            && second.close_price() < second.open_price()
            && third.close_price() < third.open_price();

        // 연속적인 하락
        let consecutive_lower = first.close_price() > second.close_price()
            && second.close_price() > third.close_price();

        // 각 캔들의 오픈이 이전 캔들의 몸통 안에 있어야 함
        let proper_opens = second.open_price() < first.open_price()
            && second.open_price() > first.close_price()
            && third.open_price() < second.open_price()
            && third.open_price() > second.close_price();

        all_bearish && consecutive_lower && proper_opens
    }

    /// 쓰리 인사이드 업 패턴 확인
    fn is_three_inside_up(&self, first: &C, second: &C, third: &C) -> bool {
        // 하라미 패턴 + 확인 캔들
        self.is_harami_pattern(first, second)
            && first.close_price() < first.open_price()
            && second.close_price() > second.open_price()
            && third.close_price() > third.open_price()
            && third.close_price() > first.close_price()
    }

    /// 쓰리 인사이드 다운 패턴 확인
    fn is_three_inside_down(&self, first: &C, second: &C, third: &C) -> bool {
        // 하라미 패턴 + 확인 캔들
        self.is_harami_pattern(first, second)
            && first.close_price() > first.open_price()
            && second.close_price() < second.open_price()
            && third.close_price() < third.open_price()
            && third.close_price() < first.close_price()
    }

    /// 패턴 신뢰도 계산
    fn calculate_pattern_reliability(
        &self,
        single_pattern: &SingleCandlePattern,
        multi_pattern: &MultiCandlePattern,
    ) -> PatternReliability {
        match multi_pattern {
            MultiCandlePattern::BullishEngulfing | MultiCandlePattern::BearishEngulfing => {
                PatternReliability::High
            }
            MultiCandlePattern::MorningStar | MultiCandlePattern::EveningStar => {
                PatternReliability::VeryHigh
            }
            MultiCandlePattern::ThreeWhiteSoldiers | MultiCandlePattern::ThreeBlackCrows => {
                PatternReliability::High
            }
            MultiCandlePattern::PiercingPattern | MultiCandlePattern::DarkCloudCover => {
                PatternReliability::Medium
            }
            MultiCandlePattern::ThreeInsideUp | MultiCandlePattern::ThreeInsideDown => {
                PatternReliability::Medium
            }
            MultiCandlePattern::TweezerTop | MultiCandlePattern::TweezerBottom => {
                PatternReliability::Medium
            }
            MultiCandlePattern::Harami => PatternReliability::Low,
            MultiCandlePattern::None => match single_pattern {
                SingleCandlePattern::Hammer | SingleCandlePattern::InvertedHammer => {
                    PatternReliability::Medium
                }
                SingleCandlePattern::HangingMan | SingleCandlePattern::ShootingStar => {
                    PatternReliability::Medium
                }
                SingleCandlePattern::GravestoneDoji | SingleCandlePattern::DragonFlyDoji => {
                    PatternReliability::Medium
                }
                SingleCandlePattern::Marubozu => PatternReliability::High,
                SingleCandlePattern::Doji => PatternReliability::Low,
                SingleCandlePattern::SpinningTop => PatternReliability::VeryLow,
                SingleCandlePattern::Normal => PatternReliability::VeryLow,
            },
        }
    }

    /// 패턴 시그널 계산
    fn calculate_pattern_signal(
        &self,
        single_pattern: &SingleCandlePattern,
        multi_pattern: &MultiCandlePattern,
    ) -> PatternSignal {
        match multi_pattern {
            MultiCandlePattern::BullishEngulfing
            | MultiCandlePattern::PiercingPattern
            | MultiCandlePattern::MorningStar
            | MultiCandlePattern::ThreeWhiteSoldiers
            | MultiCandlePattern::ThreeInsideUp
            | MultiCandlePattern::TweezerBottom => PatternSignal::StrongBullish,
            MultiCandlePattern::BearishEngulfing
            | MultiCandlePattern::DarkCloudCover
            | MultiCandlePattern::EveningStar
            | MultiCandlePattern::ThreeBlackCrows
            | MultiCandlePattern::ThreeInsideDown
            | MultiCandlePattern::TweezerTop => PatternSignal::StrongBearish,
            MultiCandlePattern::Harami => PatternSignal::Neutral,
            MultiCandlePattern::None => {
                match single_pattern {
                    SingleCandlePattern::Hammer
                    | SingleCandlePattern::InvertedHammer
                    | SingleCandlePattern::DragonFlyDoji => PatternSignal::Bullish,
                    SingleCandlePattern::HangingMan
                    | SingleCandlePattern::ShootingStar
                    | SingleCandlePattern::GravestoneDoji => PatternSignal::Bearish,
                    SingleCandlePattern::Marubozu => PatternSignal::Neutral, // 방향에 따라 다름
                    SingleCandlePattern::Doji | SingleCandlePattern::SpinningTop => {
                        PatternSignal::Neutral
                    }
                    SingleCandlePattern::Normal => PatternSignal::Neutral,
                }
            }
        }
    }

    /// 신뢰도 점수 계산
    fn calculate_confidence_score(
        &self,
        reliability: &PatternReliability,
        volume_confirmation: bool,
        trend_alignment: bool,
    ) -> f64 {
        let base_score: f64 = match reliability {
            PatternReliability::VeryHigh => 0.9,
            PatternReliability::High => 0.75,
            PatternReliability::Medium => 0.6,
            PatternReliability::Low => 0.4,
            PatternReliability::VeryLow => 0.2,
        };

        let volume_bonus: f64 = if volume_confirmation { 0.1 } else { 0.0 };
        let trend_bonus: f64 = if trend_alignment { 0.1 } else { 0.0 };

        (base_score + volume_bonus + trend_bonus).min(1.0)
    }

    /// 패턴 강도 계산
    fn calculate_pattern_strength(&self, candles: &[C]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }

        let current = match candles.first() {
            Some(c) => c,
            None => return 0.0,
        };
        let body_size = (current.close_price() - current.open_price()).abs();
        let total_range = current.high_price() - current.low_price();

        if total_range == 0.0 {
            return 0.0;
        }

        let body_ratio = body_size / total_range;
        let volume_factor = if let Some(prev) = candles.get(1) {
            let prev_volume = prev.volume();
            if prev_volume > 0.0 {
                (current.volume() / prev_volume).min(2.0)
            } else {
                1.0
            }
        } else {
            1.0
        };

        (body_ratio * volume_factor).min(1.0)
    }

    /// 볼륨 확인
    fn check_volume_confirmation(&self, candles: &[C]) -> bool {
        if candles.len() < 2 {
            return false;
        }

        let current_volume = match candles.first() {
            Some(c) => c.volume(),
            None => return false,
        };
        let prev_volume = match candles.get(1) {
            Some(c) => c.volume(),
            None => return false,
        };

        current_volume > prev_volume * 1.2
    }

    /// 추세 일치 확인
    fn check_trend_alignment(&self, candles: &[C]) -> bool {
        if candles.len() < 5 {
            return false;
        }

        let recent_closes: Vec<f64> = candles.iter().take(5).map(|c| c.close_price()).collect();
        if recent_closes.len() < 5 {
            return false;
        }
        let first_close = match recent_closes.get(4) {
            Some(&close) => close,
            None => return false,
        };
        let last_close = match recent_closes.first() {
            Some(&close) => close,
            None => return false,
        };

        let trend_direction = last_close - first_close;
        let current_direction = match candles.first() {
            Some(c) => c.close_price() - c.open_price(),
            None => return false,
        };

        trend_direction * current_direction > 0.0
    }

    /// 패턴 연속성 점수 계산
    fn calculate_pattern_continuity_score(&self, recent_patterns: &[PatternAnalysis]) -> f64 {
        if recent_patterns.is_empty() {
            return 0.0;
        }

        let consistent_signals = recent_patterns
            .iter()
            .filter(|p| p.signal != PatternSignal::Neutral)
            .count();

        (consistent_signals as f64 / recent_patterns.len() as f64).min(1.0)
    }

    /// 시장 컨텍스트 점수 계산
    fn calculate_market_context_score(&self, candles: &[C]) -> f64 {
        if candles.len() < 10 {
            return 0.5;
        }

        let recent_prices: Vec<f64> = candles.iter().take(10).map(|c| c.close_price()).collect();
        let volatility = self.calculate_volatility(&recent_prices);
        let trend_strength = self.calculate_trend_strength(&recent_prices);

        (1.0 - volatility + trend_strength) / 2.0
    }

    /// 변동성 계산
    fn calculate_volatility(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = prices.windows(2).map(|w| (w[0] - w[1]) / w[1]).collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

        variance.sqrt().min(1.0)
    }

    /// 추세 강도 계산
    fn calculate_trend_strength(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let first_price = prices[prices.len() - 1];
        let last_price = prices[0];
        let price_change = (last_price - first_price).abs() / first_price;

        price_change.min(1.0)
    }

    /// 강한 반전 패턴 신호 확인
    pub fn is_strong_reversal_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_bullish_pattern() || data.is_strong_bearish_pattern()
        } else {
            false
        }
    }

    /// 높은 신뢰도 패턴 신호 확인
    pub fn is_high_confidence_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_high_reliability_pattern() && data.pattern_analysis.confidence_score > 0.8
        } else {
            false
        }
    }

    /// 볼륨 확인된 패턴 신호 확인
    pub fn is_volume_confirmed_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_volume_confirmed_pattern()
        } else {
            false
        }
    }

    /// 패턴 클러스터링 신호 확인
    pub fn is_pattern_clustering_signal(&self, threshold: f64) -> bool {
        if let Some(data) = self.items.first() {
            data.calculate_pattern_clustering_score() > threshold
        } else {
            false
        }
    }

    /// 강한 반전 패턴 신호 확인 (n개 연속 강한 반전 패턴, 이전 m개는 아님)
    pub fn is_strong_reversal_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_strong_bullish_pattern() || data.is_strong_bearish_pattern(),
            n,
            m,
            p,
        )
    }

    /// 볼륨 확인된 패턴 신호 확인 (n개 연속 볼륨 확인된 패턴, 이전 m개는 아님)
    pub fn is_volume_confirmed_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_volume_confirmed_pattern(), n, m, p)
    }

    /// 높은 신뢰도 패턴 신호 확인 (n개 연속 높은 신뢰도 패턴, 이전 m개는 아님)
    pub fn is_high_reliability_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_reliability_pattern(), n, m, p)
    }

    /// 컨텍스트 정렬된 패턴 신호 확인 (n개 연속 컨텍스트 정렬된 패턴, 이전 m개는 아님)
    pub fn is_context_aligned_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_context_aligned_pattern(), n, m, p)
    }

    /// 반전 패턴 신호 확인 (n개 연속 반전 패턴, 이전 m개는 아님)
    pub fn is_reversal_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_reversal_pattern(), n, m, p)
    }

    /// 계속 패턴 신호 확인 (n개 연속 계속 패턴, 이전 m개는 아님)
    pub fn is_continuation_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_continuation_pattern(), n, m, p)
    }

    /// 패턴 클러스터링 신호 확인 (n개 연속 패턴 클러스터링 임계값 초과, 이전 m개는 아님)
    pub fn is_pattern_clustering_signal_breakthrough(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.calculate_pattern_clustering_score() > threshold,
            n,
            m,
            p,
        )
    }

    /// 강한 불리시 패턴 신호 확인 (n개 연속 강한 불리시 패턴, 이전 m개는 아님)
    pub fn is_strong_bullish_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_bullish_pattern(), n, m, p)
    }

    /// 강한 베어리시 패턴 신호 확인 (n개 연속 강한 베어리시 패턴, 이전 m개는 아님)
    pub fn is_strong_bearish_pattern_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_bearish_pattern(), n, m, p)
    }

    /// n개의 연속 데이터에서 강한 반전 패턴인지 확인
    pub fn is_strong_reversal_pattern(&self, n: usize, p: usize) -> bool {
        self.is_all(
            |data| data.is_strong_bullish_pattern() || data.is_strong_bearish_pattern(),
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 볼륨 확인된 패턴인지 확인
    pub fn is_volume_confirmed_pattern(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_volume_confirmed_pattern(), n, p)
    }

    /// n개의 연속 데이터에서 높은 신뢰도 패턴인지 확인
    pub fn is_high_reliability_pattern(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_high_reliability_pattern(), n, p)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<CandlePatternAnalyzerData<C>, C>
    for CandlePatternAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> CandlePatternAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = 20;
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 패턴 분석
        let single_pattern = self.identify_single_candle_pattern(&candle);
        let multi_pattern = self.identify_multi_candle_pattern(&recent_candles);
        let reliability = self.calculate_pattern_reliability(&single_pattern, &multi_pattern);
        let signal = self.calculate_pattern_signal(&single_pattern, &multi_pattern);
        let volume_confirmation = self.check_volume_confirmation(&recent_candles);
        let trend_alignment = self.check_trend_alignment(&recent_candles);
        let confidence_score =
            self.calculate_confidence_score(&reliability, volume_confirmation, trend_alignment);
        let pattern_strength = self.calculate_pattern_strength(&recent_candles);

        let pattern_analysis = PatternAnalysis {
            single_pattern,
            multi_pattern,
            reliability,
            signal,
            confidence_score,
            pattern_strength,
            volume_confirmation,
            trend_alignment,
        };

        // 최근 패턴 히스토리 수집
        let recent_patterns: Vec<PatternAnalysis> = self
            .items
            .iter()
            .take(self.pattern_history_length)
            .map(|item| item.pattern_analysis.clone())
            .collect();

        let pattern_continuity_score = self.calculate_pattern_continuity_score(&recent_patterns);
        let market_context_score = self.calculate_market_context_score(&recent_candles);

        CandlePatternAnalyzerData::new(
            candle,
            pattern_analysis,
            recent_patterns,
            pattern_continuity_score,
            market_context_score,
        )
    }

    fn datum(&self) -> &Vec<CandlePatternAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<CandlePatternAnalyzerData<C>> {
        &mut self.items
    }
}
