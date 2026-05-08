use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::candle_pattern as indicator_candle_pattern;
pub use crate::indicator::candle_pattern::{
    MultiCandlePattern, PatternAnalysis, PatternReliability, PatternSignal, SingleCandlePattern,
};
use std::fmt::Display;
use trading_chart::Candle;

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
        indicator_candle_pattern::calculate_pattern_clustering_score(
            &self.recent_patterns,
            &self.pattern_analysis.signal,
        )
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
        let pattern_analysis = indicator_candle_pattern::analyze_pattern(
            &recent_candles,
            self.min_body_ratio,
            self.min_shadow_ratio,
        );

        // 최근 패턴 히스토리 수집
        let recent_patterns: Vec<PatternAnalysis> = self
            .items
            .iter()
            .take(self.pattern_history_length)
            .map(|item| item.pattern_analysis.clone())
            .collect();

        let pattern_continuity_score =
            indicator_candle_pattern::calculate_pattern_continuity_score(&recent_patterns);
        let market_context_score =
            indicator_candle_pattern::calculate_market_context_score(&recent_candles);

        CandlePatternAnalyzerData::new(
            candle,
            pattern_analysis,
            recent_patterns,
            pattern_continuity_score,
            market_context_score,
        )
    }

    fn items(&self) -> &Vec<CandlePatternAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<CandlePatternAnalyzerData<C>> {
        &mut self.items
    }
}
