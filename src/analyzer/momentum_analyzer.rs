use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::momentum::MomentumBuilder;
pub use crate::indicator::momentum::MomentumIndicators;
use std::fmt::Display;
use trading_chart::Candle;

/// 모멘텀 방향
#[derive(Debug, Clone, PartialEq)]
pub enum MomentumDirection {
    StrongPositive,
    Positive,
    Neutral,
    Negative,
    StrongNegative,
}

/// 모멘텀 상태
#[derive(Debug, Clone, PartialEq)]
pub enum MomentumState {
    Accelerating,
    Stable,
    Decelerating,
    Reverting,
}

/// 모멘텀 다이버전스 분석
#[derive(Debug, Clone)]
pub struct MomentumDivergence {
    /// 가격 다이버전스 유형
    pub price_divergence_type: DivergenceType,
    /// RSI 다이버전스
    pub rsi_divergence: bool,
    /// 스토캐스틱 다이버전스
    pub stochastic_divergence: bool,
    /// 다이버전스 강도
    pub divergence_strength: f64,
    /// 다이버전스 신뢰도
    pub divergence_confidence: f64,
}

/// 다이버전스 타입
#[derive(Debug, Clone, PartialEq)]
pub enum DivergenceType {
    /// 불리시 다이버전스 (가격 상승, 지표 하락)
    Bullish,
    /// 베어리시 다이버전스 (가격 하락, 지표 상승)
    Bearish,
    /// 숨겨진 불리시 다이버전스
    HiddenBullish,
    /// 숨겨진 베어리시 다이버전스
    HiddenBearish,
    /// 다이버전스 없음
    None,
}

/// 모멘텀 분석 결과
#[derive(Debug, Clone)]
pub struct MomentumAnalysis {
    /// 모멘텀 방향
    pub momentum_direction: MomentumDirection,
    /// 모멘텀 상태
    pub momentum_state: MomentumState,
    /// 모멘텀 강도 (0.0-1.0)
    pub momentum_strength: f64,
    /// 모멘텀 지속성 점수
    pub momentum_persistence: f64,
    /// 과매수/과매도 여부
    pub overbought_oversold: OverBoughtOverSold,
    /// 모멘텀 다이버전스 분석
    pub divergence_analysis: MomentumDivergence,
    /// 모멘텀 변화율
    pub momentum_change_rate: f64,
    /// 모멘텀 안정성 점수
    pub momentum_stability: f64,
}

/// 과매수/과매도 상태
#[derive(Debug, Clone, PartialEq)]
pub enum OverBoughtOverSold {
    /// 심각한 과매수
    ExtremeOverbought,
    /// 과매수
    Overbought,
    /// 중립
    Neutral,
    /// 과매도
    Oversold,
    /// 심각한 과매도
    ExtremeOversold,
}

/// Momentum 분석기 데이터
#[derive(Debug)]
pub struct MomentumAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 모멘텀 지표들
    pub momentum_indicators: MomentumIndicators,
    /// 모멘텀 분석 결과
    pub momentum_analysis: MomentumAnalysis,
    /// 모멘텀 히스토리
    pub momentum_history: Vec<MomentumAnalysis>,
    /// 모멘텀 극값 정보
    pub momentum_extremes: Vec<(usize, f64)>,
}

impl<C: Candle> MomentumAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        momentum_indicators: MomentumIndicators,
        momentum_analysis: MomentumAnalysis,
        momentum_history: Vec<MomentumAnalysis>,
        momentum_extremes: Vec<(usize, f64)>,
    ) -> MomentumAnalyzerData<C> {
        MomentumAnalyzerData {
            candle,
            momentum_indicators,
            momentum_analysis,
            momentum_history,
            momentum_extremes,
        }
    }

    /// 강한 상승 모멘텀인지 확인
    pub fn is_strong_positive_momentum(&self) -> bool {
        matches!(
            self.momentum_analysis.momentum_direction,
            MomentumDirection::StrongPositive
        ) && self.momentum_analysis.momentum_strength > 0.7
    }

    /// 강한 하락 모멘텀인지 확인
    pub fn is_strong_negative_momentum(&self) -> bool {
        matches!(
            self.momentum_analysis.momentum_direction,
            MomentumDirection::StrongNegative
        ) && self.momentum_analysis.momentum_strength > 0.7
    }

    /// 모멘텀 가속 중인지 확인
    pub fn is_accelerating_momentum(&self) -> bool {
        matches!(
            self.momentum_analysis.momentum_state,
            MomentumState::Accelerating
        )
    }

    /// 모멘텀 감속 중인지 확인
    pub fn is_decelerating_momentum(&self) -> bool {
        matches!(
            self.momentum_analysis.momentum_state,
            MomentumState::Decelerating
        )
    }

    /// 과매수 상태인지 확인
    pub fn is_overbought(&self) -> bool {
        matches!(
            self.momentum_analysis.overbought_oversold,
            OverBoughtOverSold::Overbought | OverBoughtOverSold::ExtremeOverbought
        )
    }

    /// 과매도 상태인지 확인
    pub fn is_oversold(&self) -> bool {
        matches!(
            self.momentum_analysis.overbought_oversold,
            OverBoughtOverSold::Oversold | OverBoughtOverSold::ExtremeOversold
        )
    }

    /// 모멘텀 다이버전스 존재 여부 확인
    pub fn has_momentum_divergence(&self) -> bool {
        !matches!(
            self.momentum_analysis
                .divergence_analysis
                .price_divergence_type,
            DivergenceType::None
        )
    }

    /// 불리시 다이버전스 확인
    pub fn is_bullish_divergence(&self) -> bool {
        matches!(
            self.momentum_analysis
                .divergence_analysis
                .price_divergence_type,
            DivergenceType::Bullish | DivergenceType::HiddenBullish
        ) && self
            .momentum_analysis
            .divergence_analysis
            .divergence_confidence
            > 0.6
    }

    /// 베어리시 다이버전스 확인
    pub fn is_bearish_divergence(&self) -> bool {
        matches!(
            self.momentum_analysis
                .divergence_analysis
                .price_divergence_type,
            DivergenceType::Bearish | DivergenceType::HiddenBearish
        ) && self
            .momentum_analysis
            .divergence_analysis
            .divergence_confidence
            > 0.6
    }

    /// 모멘텀 지속성이 높은지 확인
    pub fn is_persistent_momentum(&self) -> bool {
        self.momentum_analysis.momentum_persistence > 0.7
    }

    /// 모멘텀 안정성이 높은지 확인
    pub fn is_stable_momentum(&self) -> bool {
        self.momentum_analysis.momentum_stability > 0.6
    }

    /// 모멘텀 반전 신호 확인
    pub fn is_momentum_reversal_signal(&self) -> bool {
        matches!(
            self.momentum_analysis.momentum_state,
            MomentumState::Reverting
        ) && (self.is_overbought() || self.is_oversold())
    }

    /// 모멘텀 극값 근처인지 확인
    pub fn is_near_momentum_extreme(&self, threshold: f64) -> bool {
        let current_momentum = self.momentum_analysis.momentum_strength;
        self.momentum_extremes
            .iter()
            .any(|(_, extreme_value)| (current_momentum - extreme_value).abs() < threshold)
    }

    /// 모멘텀 일관성 점수 계산
    pub fn calculate_momentum_consistency(&self, lookback: usize) -> f64 {
        if self.momentum_history.len() < lookback {
            return 0.5;
        }

        let recent_directions: Vec<&MomentumDirection> = self
            .momentum_history
            .iter()
            .take(lookback)
            .map(|m| &m.momentum_direction)
            .collect();

        let consistent_count = recent_directions
            .iter()
            .filter(|&&dir| *dir == self.momentum_analysis.momentum_direction)
            .count();

        consistent_count as f64 / lookback as f64
    }
}

impl<C: Candle> GetCandle<C> for MomentumAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for MomentumAnalyzerData<C> {}

impl<C: Candle> Display for MomentumAnalyzerData<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "캔들: {}, 모멘텀: {:?}, 강도: {:.2}, RSI: {:.2}",
            self.candle,
            self.momentum_analysis.momentum_direction,
            self.momentum_analysis.momentum_strength,
            self.momentum_indicators.rsi
        )
    }
}

/// Momentum 분석기
#[derive(Debug)]
pub struct MomentumAnalyzer<C: Candle> {
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<MomentumAnalyzerData<C>>,
    /// RSI 기간
    pub rsi_period: usize,
    /// 스토캐스틱 기간
    pub stoch_period: usize,
    /// 윌리엄스 %R 기간
    pub williams_period: usize,
    /// ROC 기간
    pub roc_period: usize,
    /// CCI 기간
    pub cci_period: usize,
    /// 모멘텀 기간
    pub momentum_period: usize,
    /// 히스토리 길이
    pub history_length: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MomentumAnalyzerParams {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub williams_period: usize,
    pub roc_period: usize,
    pub cci_period: usize,
    pub momentum_period: usize,
    pub history_length: usize,
}

impl<C: Candle> Display for MomentumAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "{first}"),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + Clone + 'static> MomentumAnalyzer<C> {
    /// 새 Momentum 분석기 생성
    pub fn new(storage: &CandleStore<C>, params: MomentumAnalyzerParams) -> MomentumAnalyzer<C> {
        let MomentumAnalyzerParams {
            rsi_period,
            stoch_period,
            williams_period,
            roc_period,
            cci_period,
            momentum_period,
            history_length,
        } = params;
        let mut analyzer = MomentumAnalyzer {
            items: Vec::new(),
            rsi_period,
            stoch_period,
            williams_period,
            roc_period,
            cci_period,
            momentum_period,
            history_length,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> MomentumAnalyzer<C> {
        Self::new(
            storage,
            MomentumAnalyzerParams {
                rsi_period: 14,
                stoch_period: 14,
                williams_period: 14,
                roc_period: 10,
                cci_period: 20,
                momentum_period: 10,
                history_length: 20,
            },
        )
    }

    /// 모멘텀 방향 결정
    fn determine_momentum_direction(&self, indicators: &MomentumIndicators) -> MomentumDirection {
        let positive_signals = [
            indicators.rsi > 60.0,
            indicators.stoch_k > 60.0,
            indicators.williams_r > -40.0,
            indicators.roc > 2.0,
            indicators.cci > 100.0,
            indicators.momentum > 0.0,
            indicators.ultimate_oscillator > 60.0,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        let negative_signals = [
            indicators.rsi < 40.0,
            indicators.stoch_k < 40.0,
            indicators.williams_r < -60.0,
            indicators.roc < -2.0,
            indicators.cci < -100.0,
            indicators.momentum < 0.0,
            indicators.ultimate_oscillator < 40.0,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        match (positive_signals, negative_signals) {
            (p, _) if p >= 6 => MomentumDirection::StrongPositive,
            (p, _) if p >= 4 => MomentumDirection::Positive,
            (_, n) if n >= 6 => MomentumDirection::StrongNegative,
            (_, n) if n >= 4 => MomentumDirection::Negative,
            _ => MomentumDirection::Neutral,
        }
    }

    /// 모멘텀 상태 결정
    fn determine_momentum_state(
        &self,
        current_strength: f64,
        previous_strength: f64,
    ) -> MomentumState {
        let change = current_strength - previous_strength;
        let change_threshold = 0.05;

        if change > change_threshold {
            MomentumState::Accelerating
        } else if change < -change_threshold {
            MomentumState::Decelerating
        } else if change.abs() < change_threshold / 2.0 {
            MomentumState::Stable
        } else {
            MomentumState::Reverting
        }
    }

    /// 모멘텀 강도 계산
    fn calculate_momentum_strength(&self, indicators: &MomentumIndicators) -> f64 {
        let rsi_strength = (indicators.rsi - 50.0).abs() / 50.0;
        let stoch_strength =
            ((indicators.stoch_k - 50.0).abs() + (indicators.stoch_d - 50.0).abs()) / 100.0;
        let williams_strength = (indicators.williams_r + 50.0).abs() / 50.0;
        let roc_strength = (indicators.roc.abs() / 10.0).min(1.0);
        let cci_strength = (indicators.cci.abs() / 200.0).min(1.0);
        let momentum_strength = (indicators.momentum.abs() / 5.0).min(1.0);
        let uo_strength = (indicators.ultimate_oscillator - 50.0).abs() / 50.0;

        (rsi_strength
            + stoch_strength
            + williams_strength
            + roc_strength
            + cci_strength
            + momentum_strength
            + uo_strength)
            / 7.0
    }

    /// 과매수/과매도 상태 결정
    fn determine_overbought_oversold(&self, indicators: &MomentumIndicators) -> OverBoughtOverSold {
        let overbought_count = [
            indicators.rsi > 80.0,
            indicators.stoch_k > 80.0,
            indicators.williams_r > -20.0,
            indicators.cci > 200.0,
            indicators.ultimate_oscillator > 80.0,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        let oversold_count = [
            indicators.rsi < 20.0,
            indicators.stoch_k < 20.0,
            indicators.williams_r < -80.0,
            indicators.cci < -200.0,
            indicators.ultimate_oscillator < 20.0,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        match (overbought_count, oversold_count) {
            (o, _) if o >= 4 => OverBoughtOverSold::ExtremeOverbought,
            (o, _) if o >= 2 => OverBoughtOverSold::Overbought,
            (_, s) if s >= 4 => OverBoughtOverSold::ExtremeOversold,
            (_, s) if s >= 2 => OverBoughtOverSold::Oversold,
            _ => OverBoughtOverSold::Neutral,
        }
    }

    /// 다이버전스 분석
    fn analyze_divergence(
        &self,
        candles: &[C],
        indicators_history: &[MomentumIndicators],
    ) -> MomentumDivergence {
        if candles.len() < 10 || indicators_history.len() < 10 {
            return MomentumDivergence {
                price_divergence_type: DivergenceType::None,
                rsi_divergence: false,
                stochastic_divergence: false,
                divergence_strength: 0.0,
                divergence_confidence: 0.0,
            };
        }

        // 간소화된 다이버전스 분석
        let recent_prices: Vec<f64> = candles[..10].iter().map(|c| c.close_price()).collect();
        let recent_rsi: Vec<f64> = indicators_history[..10].iter().map(|i| i.rsi).collect();

        let price_trend = recent_prices[0] - recent_prices[9];
        let rsi_trend = recent_rsi[0] - recent_rsi[9];

        let divergence_type = if price_trend > 0.0 && rsi_trend < 0.0 {
            DivergenceType::Bearish
        } else if price_trend < 0.0 && rsi_trend > 0.0 {
            DivergenceType::Bullish
        } else {
            DivergenceType::None
        };

        let divergence_strength = if divergence_type != DivergenceType::None {
            (price_trend.abs() + rsi_trend.abs()) / 2.0
        } else {
            0.0
        };

        MomentumDivergence {
            price_divergence_type: divergence_type,
            rsi_divergence: price_trend * rsi_trend < 0.0,
            stochastic_divergence: false, // 간소화
            divergence_strength: divergence_strength.min(1.0),
            divergence_confidence: if divergence_strength > 0.5 { 0.7 } else { 0.3 },
        }
    }

    /// 모멘텀 지속성 계산
    fn calculate_momentum_persistence(&self, momentum_history: &[MomentumAnalysis]) -> f64 {
        if momentum_history.len() < 5 {
            return 0.5;
        }

        let consistent_directions = momentum_history
            .windows(2)
            .filter(|w| w[0].momentum_direction == w[1].momentum_direction)
            .count();

        consistent_directions as f64 / (momentum_history.len() - 1) as f64
    }

    /// 모멘텀 극값 식별
    fn identify_momentum_extremes(
        &self,
        indicators_history: &[MomentumIndicators],
    ) -> Vec<(usize, f64)> {
        let mut extremes = Vec::new();

        if indicators_history.len() < 5 {
            return extremes;
        }

        for i in 2..indicators_history.len() - 2 {
            let rsi = indicators_history[i].rsi;
            let prev_rsi = indicators_history[i - 1].rsi;
            let next_rsi = indicators_history[i + 1].rsi;

            // 극값 확인 (간소화된 버전)
            if (rsi > prev_rsi && rsi > next_rsi && rsi > 70.0)
                || (rsi < prev_rsi && rsi < next_rsi && rsi < 30.0)
            {
                extremes.push((i, rsi));
            }
        }

        extremes
    }

    /// 모멘텀 안정성 계산
    fn calculate_momentum_stability(&self, indicators_history: &[MomentumIndicators]) -> f64 {
        if indicators_history.len() < 5 {
            return 0.5;
        }

        let rsi_values: Vec<f64> = indicators_history.iter().map(|i| i.rsi).collect();
        let mean_rsi = rsi_values.iter().sum::<f64>() / rsi_values.len() as f64;
        let variance = rsi_values
            .iter()
            .map(|&rsi| (rsi - mean_rsi).powi(2))
            .sum::<f64>()
            / rsi_values.len() as f64;

        let stability = 1.0 - (variance.sqrt() / 50.0).min(1.0);
        stability.max(0.0)
    }

    /// 강한 모멘텀 신호 확인
    pub fn is_strong_momentum_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_positive_momentum() || data.is_strong_negative_momentum()
        } else {
            false
        }
    }

    /// 모멘텀 다이버전스 신호 확인
    pub fn is_momentum_divergence_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.has_momentum_divergence()
                && data
                    .momentum_analysis
                    .divergence_analysis
                    .divergence_confidence
                    > 0.7
        } else {
            false
        }
    }

    /// 모멘텀 반전 신호 확인
    pub fn is_momentum_reversal_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_momentum_reversal_signal()
        } else {
            false
        }
    }

    /// 지속적인 모멘텀 신호 확인
    pub fn is_persistent_momentum_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_persistent_momentum() && data.is_stable_momentum()
        } else {
            false
        }
    }

    /// 강한 양의 모멘텀 신호 확인 (n개 연속 강한 양의 모멘텀, 이전 m개는 아님)
    pub fn is_strong_positive_momentum_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_positive_momentum(), n, m, p)
    }

    /// 강한 음의 모멘텀 신호 확인 (n개 연속 강한 음의 모멘텀, 이전 m개는 아님)
    pub fn is_strong_negative_momentum_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_negative_momentum(), n, m, p)
    }

    /// 모멘텀 가속 신호 확인 (n개 연속 모멘텀 가속, 이전 m개는 아님)
    pub fn is_accelerating_momentum_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_accelerating_momentum(), n, m, p)
    }

    /// 모멘텀 감속 신호 확인 (n개 연속 모멘텀 감속, 이전 m개는 아님)
    pub fn is_decelerating_momentum_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_decelerating_momentum(), n, m, p)
    }

    /// 과매수 신호 확인 (n개 연속 과매수, 이전 m개는 아님)
    pub fn is_overbought_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_overbought(), n, m, p)
    }

    /// 과매도 신호 확인 (n개 연속 과매도, 이전 m개는 아님)
    pub fn is_oversold_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_oversold(), n, m, p)
    }

    /// 모멘텀 다이버전스 돌파 신호 확인 (n개 연속 모멘텀 다이버전스, 이전 m개는 아님)
    pub fn is_momentum_divergence_breakthrough(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.has_momentum_divergence(), n, m, p)
    }

    /// 불리시 다이버전스 신호 확인 (n개 연속 불리시 다이버전스, 이전 m개는 아님)
    pub fn is_bullish_divergence_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_bullish_divergence(), n, m, p)
    }

    /// 베어리시 다이버전스 신호 확인 (n개 연속 베어리시 다이버전스, 이전 m개는 아님)
    pub fn is_bearish_divergence_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_bearish_divergence(), n, m, p)
    }

    /// 지속적인 모멘텀 돌파 신호 확인 (n개 연속 지속적인 모멘텀, 이전 m개는 아님)
    pub fn is_persistent_momentum_breakthrough(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_persistent_momentum(), n, m, p)
    }

    /// 안정적인 모멘텀 신호 확인 (n개 연속 안정적인 모멘텀, 이전 m개는 아님)
    pub fn is_stable_momentum_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_stable_momentum(), n, m, p)
    }

    /// 모멘텀 반전 돌파 신호 확인 (n개 연속 모멘텀 반전 신호, 이전 m개는 아님)
    pub fn is_momentum_reversal_breakthrough(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_momentum_reversal_signal(), n, m, p)
    }

    /// 모멘텀 극값 근처 신호 확인 (n개 연속 모멘텀 극값 근처, 이전 m개는 아님)
    pub fn is_near_momentum_extreme_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_near_momentum_extreme(threshold),
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 강한 양의 모멘텀인지 확인
    pub fn is_strong_positive_momentum(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_strong_positive_momentum(), n, p)
    }

    /// n개의 연속 데이터에서 강한 음의 모멘텀인지 확인
    pub fn is_strong_negative_momentum(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_strong_negative_momentum(), n, p)
    }

    /// n개의 연속 데이터에서 모멘텀 가속인지 확인
    pub fn is_accelerating_momentum(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_accelerating_momentum(), n, p)
    }

    /// n개의 연속 데이터에서 모멘텀 감속인지 확인
    pub fn is_decelerating_momentum(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_decelerating_momentum(), n, p)
    }

    /// n개의 연속 데이터에서 과매수인지 확인
    pub fn is_overbought(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_overbought(), n, p)
    }

    /// n개의 연속 데이터에서 과매도인지 확인
    pub fn is_oversold(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_oversold(), n, p)
    }

    /// n개의 연속 데이터에서 모멘텀 다이버전스인지 확인
    pub fn is_momentum_divergence(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.has_momentum_divergence(), n, p)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<MomentumAnalyzerData<C>, C> for MomentumAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> MomentumAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = 50;
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 모멘텀 지표들 계산
        let chronological_candles: Vec<C> = recent_candles.iter().rev().cloned().collect();
        let momentum_indicators = MomentumBuilder::new(
            self.rsi_period,
            self.stoch_period,
            self.williams_period,
            self.roc_period,
            self.cci_period,
            self.momentum_period,
        )
        .build(&chronological_candles);

        // 모멘텀 분석
        let momentum_direction = self.determine_momentum_direction(&momentum_indicators);
        let momentum_strength = self.calculate_momentum_strength(&momentum_indicators);

        let momentum_state = if let Some(prev_data) = self.items.first() {
            self.determine_momentum_state(
                momentum_strength,
                prev_data.momentum_analysis.momentum_strength,
            )
        } else {
            MomentumState::Stable
        };

        let overbought_oversold = self.determine_overbought_oversold(&momentum_indicators);

        // 지표 히스토리 수집
        let mut indicators_history = vec![momentum_indicators.clone()];
        for item in self.items.iter().take(self.history_length - 1) {
            indicators_history.push(item.momentum_indicators.clone());
        }

        let divergence_analysis = self.analyze_divergence(&recent_candles, &indicators_history);

        // 모멘텀 히스토리 수집
        let momentum_history: Vec<MomentumAnalysis> = self
            .items
            .iter()
            .take(self.history_length)
            .map(|item| item.momentum_analysis.clone())
            .collect();

        let momentum_persistence = self.calculate_momentum_persistence(&momentum_history);
        let momentum_change_rate = if let Some(prev_data) = self.items.first() {
            momentum_strength - prev_data.momentum_analysis.momentum_strength
        } else {
            0.0
        };
        let momentum_stability = self.calculate_momentum_stability(&indicators_history);

        let momentum_analysis = MomentumAnalysis {
            momentum_direction,
            momentum_state,
            momentum_strength,
            momentum_persistence,
            overbought_oversold,
            divergence_analysis,
            momentum_change_rate,
            momentum_stability,
        };

        let momentum_extremes = self.identify_momentum_extremes(&indicators_history);

        MomentumAnalyzerData::new(
            candle,
            momentum_indicators,
            momentum_analysis,
            momentum_history,
            momentum_extremes,
        )
    }

    fn items(&self) -> &Vec<MomentumAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<MomentumAnalyzerData<C>> {
        &mut self.items
    }
}
