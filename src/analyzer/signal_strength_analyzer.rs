use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 신호 강도 레벨
#[derive(Debug, Clone, PartialEq)]
pub enum SignalStrengthLevel {
    VeryStrong,
    Strong,
    Moderate,
    Weak,
    VeryWeak,
}

/// 신호 방향
#[derive(Debug, Clone, PartialEq)]
pub enum SignalDirection {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

/// 신호 품질 평가
#[derive(Debug, Clone)]
pub struct SignalQuality {
    /// 신호 일치도 (0.0-1.0)
    pub consensus_score: f64,
    /// 신호 강도 (0.0-1.0)
    pub strength_score: f64,
    /// 신호 신뢰도 (0.0-1.0)
    pub reliability_score: f64,
    /// 시장 조건 점수 (0.0-1.0)
    pub market_condition_score: f64,
    /// 종합 점수 (0.0-1.0)
    pub overall_score: f64,
}

/// 개별 분석기 신호 가중치
#[derive(Debug, Clone)]
pub struct AnalyzerWeights {
    /// 추세 분석기 가중치
    pub trend_weight: f64,
    /// 모멘텀 분석기 가중치
    pub momentum_weight: f64,
    /// 변동성 분석기 가중치
    pub volatility_weight: f64,
    /// 볼륨 분석기 가중치
    pub volume_weight: f64,
    /// 지지/저항 분석기 가중치
    pub support_resistance_weight: f64,
    /// 가격 액션 분석기 가중치
    pub price_action_weight: f64,
    /// 시장 구조 분석기 가중치
    pub market_structure_weight: f64,
    /// 리스크 관리 분석기 가중치
    pub risk_management_weight: f64,
    /// 캔들 패턴 분석기 가중치
    pub candle_pattern_weight: f64,
}

impl Default for AnalyzerWeights {
    fn default() -> Self {
        AnalyzerWeights {
            trend_weight: 0.15,
            momentum_weight: 0.12,
            volatility_weight: 0.08,
            volume_weight: 0.10,
            support_resistance_weight: 0.15,
            price_action_weight: 0.12,
            market_structure_weight: 0.13,
            risk_management_weight: 0.10,
            candle_pattern_weight: 0.05,
        }
    }
}

/// 신호 강도 분석 결과
#[derive(Debug, Clone)]
pub struct SignalAnalysis {
    /// 매수 신호 강도 (0.0-1.0)
    pub buy_signal_strength: f64,
    /// 매도 신호 강도 (0.0-1.0)
    pub sell_signal_strength: f64,
    /// 최종 신호 방향
    pub signal_direction: SignalDirection,
    /// 신호 강도 레벨
    pub strength_level: SignalStrengthLevel,
    /// 신호 품질 평가
    pub signal_quality: SignalQuality,
    /// 개별 분석기 기여도
    pub analyzer_contributions: Vec<(String, f64)>,
    /// 시장 상황 평가
    pub market_conditions: String,
    /// 추천 액션
    pub recommended_action: String,
}

/// Signal Strength 분석기 데이터
#[derive(Debug)]
pub struct SignalStrengthAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 신호 분석 결과
    pub signal_analysis: SignalAnalysis,
    /// 신호 히스토리
    pub signal_history: Vec<SignalAnalysis>,
    /// 신호 안정성 점수
    pub signal_stability_score: f64,
    /// 시장 상황 변화율
    pub market_condition_change_rate: f64,
}

impl<C: Candle> SignalStrengthAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        signal_analysis: SignalAnalysis,
        signal_history: Vec<SignalAnalysis>,
        signal_stability_score: f64,
        market_condition_change_rate: f64,
    ) -> SignalStrengthAnalyzerData<C> {
        SignalStrengthAnalyzerData {
            candle,
            signal_analysis,
            signal_history,
            signal_stability_score,
            market_condition_change_rate,
        }
    }

    /// 강한 매수 신호인지 확인
    pub fn is_strong_buy_signal(&self) -> bool {
        matches!(
            self.signal_analysis.signal_direction,
            SignalDirection::StrongBuy
        ) && self.signal_analysis.buy_signal_strength > 0.75
    }

    /// 강한 매도 신호인지 확인
    pub fn is_strong_sell_signal(&self) -> bool {
        matches!(
            self.signal_analysis.signal_direction,
            SignalDirection::StrongSell
        ) && self.signal_analysis.sell_signal_strength > 0.75
    }

    /// 신호 강도가 높은지 확인
    pub fn is_high_strength_signal(&self) -> bool {
        matches!(
            self.signal_analysis.strength_level,
            SignalStrengthLevel::Strong | SignalStrengthLevel::VeryStrong
        )
    }

    /// 신호 품질이 높은지 확인
    pub fn is_high_quality_signal(&self) -> bool {
        self.signal_analysis.signal_quality.overall_score > 0.75
    }

    /// 신호가 안정적인지 확인
    pub fn is_stable_signal(&self) -> bool {
        self.signal_stability_score > 0.7
    }

    /// 신호 방향이 일관된지 확인
    pub fn is_consistent_signal(&self, lookback: usize) -> bool {
        if self.signal_history.len() < lookback {
            return false;
        }

        let recent_signals: Vec<&SignalDirection> = self
            .signal_history
            .iter()
            .take(lookback)
            .map(|s| &s.signal_direction)
            .collect();

        let consistent_count = recent_signals
            .iter()
            .filter(|&&dir| *dir == self.signal_analysis.signal_direction)
            .count();

        consistent_count as f64 / lookback as f64 > 0.6
    }

    /// 시장 조건 점수가 좋은지 확인
    pub fn is_good_market_condition(&self) -> bool {
        self.signal_analysis.signal_quality.market_condition_score > 0.6
    }

    /// 종합 신호 점수 계산
    pub fn calculate_overall_signal_score(&self) -> f64 {
        let strength_factor = match self.signal_analysis.strength_level {
            SignalStrengthLevel::VeryStrong => 1.0,
            SignalStrengthLevel::Strong => 0.8,
            SignalStrengthLevel::Moderate => 0.6,
            SignalStrengthLevel::Weak => 0.4,
            SignalStrengthLevel::VeryWeak => 0.2,
        };

        let quality_factor = self.signal_analysis.signal_quality.overall_score;
        let stability_factor = self.signal_stability_score;
        let market_factor = self.signal_analysis.signal_quality.market_condition_score;

        (strength_factor * 0.3
            + quality_factor * 0.3
            + stability_factor * 0.2
            + market_factor * 0.2)
            .min(1.0)
    }
}

impl<C: Candle> GetCandle<C> for SignalStrengthAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for SignalStrengthAnalyzerData<C> {}

impl<C: Candle> Display for SignalStrengthAnalyzerData<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "캔들: {}, 신호: {:?}, 강도: {:.2}, 품질: {:.2}",
            self.candle,
            self.signal_analysis.signal_direction,
            self.signal_analysis
                .buy_signal_strength
                .max(self.signal_analysis.sell_signal_strength),
            self.signal_analysis.signal_quality.overall_score
        )
    }
}

/// Signal Strength 분석기
#[derive(Debug)]
pub struct SignalStrengthAnalyzer<C: Candle> {
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<SignalStrengthAnalyzerData<C>>,
    /// 분석기 가중치
    pub weights: AnalyzerWeights,
    /// 신호 히스토리 길이
    pub signal_history_length: usize,
    /// 최소 신호 강도 임계값
    pub min_signal_threshold: f64,
    /// 최소 품질 임계값
    pub min_quality_threshold: f64,
}

impl<C: Candle> Display for SignalStrengthAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "{first}"),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + Clone + 'static> SignalStrengthAnalyzer<C> {
    /// 새 Signal Strength 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        weights: AnalyzerWeights,
        signal_history_length: usize,
        min_signal_threshold: f64,
        min_quality_threshold: f64,
    ) -> SignalStrengthAnalyzer<C> {
        let mut analyzer = SignalStrengthAnalyzer {
            items: Vec::new(),
            weights,
            signal_history_length,
            min_signal_threshold,
            min_quality_threshold,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> SignalStrengthAnalyzer<C> {
        Self::new(storage, AnalyzerWeights::default(), 20, 0.3, 0.4)
    }

    /// 추세 분석기 신호 계산
    fn calculate_trend_signals(&self, candles: &[C]) -> (f64, f64) {
        // 이동평균 기반 추세 분석
        if candles.len() < 20 {
            return (0.0, 0.0);
        }

        let short_ma = self.calculate_sma(&candles[..10]);
        let long_ma = self.calculate_sma(&candles[..20]);
        let current_price = candles[0].close_price();

        let trend_strength = ((short_ma - long_ma) / long_ma).abs();
        let _price_position = (current_price - long_ma) / long_ma;

        let buy_signal = if short_ma > long_ma && current_price > short_ma {
            (trend_strength * 2.0).min(1.0)
        } else {
            0.0
        };

        let sell_signal = if short_ma < long_ma && current_price < short_ma {
            (trend_strength * 2.0).min(1.0)
        } else {
            0.0
        };

        (buy_signal, sell_signal)
    }

    /// 모멘텀 분석기 신호 계산
    fn calculate_momentum_signals(&self, candles: &[C]) -> (f64, f64) {
        if candles.len() < 14 {
            return (0.0, 0.0);
        }

        let rsi = self.calculate_rsi(&candles[..14]);
        let momentum = self.calculate_momentum(&candles[..10]);

        let buy_signal = if rsi < 30.0 && momentum > 0.0 {
            ((30.0 - rsi) / 30.0 * momentum).min(1.0)
        } else if rsi > 30.0 && rsi < 50.0 && momentum > 0.02 {
            (momentum * 20.0).min(1.0)
        } else {
            0.0
        };

        let sell_signal = if rsi > 70.0 && momentum < 0.0 {
            ((rsi - 70.0) / 30.0 * momentum.abs()).min(1.0)
        } else if rsi < 70.0 && rsi > 50.0 && momentum < -0.02 {
            (momentum.abs() * 20.0).min(1.0)
        } else {
            0.0
        };

        (buy_signal, sell_signal)
    }

    /// 변동성 분석기 신호 계산
    fn calculate_volatility_signals(&self, candles: &[C]) -> (f64, f64) {
        if candles.len() < 20 {
            return (0.0, 0.0);
        }

        let volatility = self.calculate_volatility(&candles[..20]);
        let atr = self.calculate_atr(&candles[..14]);
        let current_range = candles[0].high_price() - candles[0].low_price();

        let volatility_factor = if volatility > 0.0 {
            (current_range / atr).min(2.0)
        } else {
            1.0
        };

        // 높은 변동성은 신호 감소, 낮은 변동성은 신호 증가
        let volatility_adjustment = if volatility > 0.05 { 0.5 } else { 1.0 };

        (
            volatility_adjustment * volatility_factor * 0.3,
            volatility_adjustment * volatility_factor * 0.3,
        )
    }

    /// 볼륨 분석기 신호 계산
    fn calculate_volume_signals(&self, candles: &[C]) -> (f64, f64) {
        if candles.len() < 10 {
            return (0.0, 0.0);
        }

        let avg_volume = candles[1..10].iter().map(|c| c.volume()).sum::<f64>() / 9.0;
        let current_volume = candles[0].volume();
        let volume_ratio = if avg_volume > 0.0 {
            current_volume / avg_volume
        } else {
            1.0
        };

        let is_bullish = candles[0].close_price() > candles[0].open_price();
        let volume_strength = (volume_ratio - 1.0).clamp(0.0, 1.0);

        let buy_signal = if is_bullish && volume_ratio > 1.2 {
            volume_strength
        } else {
            0.0
        };

        let sell_signal = if !is_bullish && volume_ratio > 1.2 {
            volume_strength
        } else {
            0.0
        };

        (buy_signal, sell_signal)
    }

    /// 지지/저항 분석기 신호 계산 (실제 분석기 사용)
    fn calculate_support_resistance_signals(&self, candles: &[C]) -> (f64, f64) {
        // 간소화된 지지/저항 계산
        if candles.len() < 20 {
            return (0.0, 0.0);
        }

        let current_price = candles[0].close_price();
        let recent_highs: Vec<f64> = candles[..20].iter().map(|c| c.high_price()).collect();
        let recent_lows: Vec<f64> = candles[..20].iter().map(|c| c.low_price()).collect();

        let resistance_level = recent_highs.iter().fold(0.0f64, |max, &x| max.max(x));
        let support_level = recent_lows.iter().fold(f64::MAX, |min, &x| min.min(x));

        let resistance_distance = (resistance_level - current_price) / current_price;
        let support_distance = (current_price - support_level) / current_price;

        let buy_signal = if support_distance < 0.02 {
            (0.02 - support_distance) * 50.0
        } else {
            0.0
        };

        let sell_signal = if resistance_distance < 0.02 {
            (0.02 - resistance_distance) * 50.0
        } else {
            0.0
        };

        (buy_signal.min(1.0), sell_signal.min(1.0))
    }

    /// 신호 강도 레벨 계산
    fn calculate_strength_level(&self, max_signal: f64) -> SignalStrengthLevel {
        match max_signal {
            s if s >= 0.8 => SignalStrengthLevel::VeryStrong,
            s if s >= 0.6 => SignalStrengthLevel::Strong,
            s if s >= 0.4 => SignalStrengthLevel::Moderate,
            s if s >= 0.2 => SignalStrengthLevel::Weak,
            _ => SignalStrengthLevel::VeryWeak,
        }
    }

    /// 신호 방향 계산
    fn calculate_signal_direction(&self, buy_signal: f64, sell_signal: f64) -> SignalDirection {
        let signal_diff = buy_signal - sell_signal;
        let max_signal = buy_signal.max(sell_signal);

        match signal_diff {
            d if d > 0.3 && max_signal > 0.7 => SignalDirection::StrongBuy,
            d if d > 0.1 && max_signal > 0.4 => SignalDirection::Buy,
            d if d < -0.3 && max_signal > 0.7 => SignalDirection::StrongSell,
            d if d < -0.1 && max_signal > 0.4 => SignalDirection::Sell,
            _ => SignalDirection::Neutral,
        }
    }

    /// 신호 품질 계산
    fn calculate_signal_quality(
        &self,
        buy_signal: f64,
        sell_signal: f64,
        analyzer_signals: &[(f64, f64)],
    ) -> SignalQuality {
        // 신호 일치도 계산
        let total_signals = analyzer_signals.len() as f64;
        let buy_agreements = analyzer_signals.iter().filter(|(b, s)| *b > *s).count() as f64;
        let sell_agreements = analyzer_signals.iter().filter(|(b, s)| *s > *b).count() as f64;
        let max_agreements = buy_agreements.max(sell_agreements);
        let consensus_score = max_agreements / total_signals;

        // 신호 강도 계산
        let max_signal = buy_signal.max(sell_signal);
        let strength_score = max_signal;

        // 신호 신뢰도 계산 (분산 기반)
        let signal_variance = analyzer_signals
            .iter()
            .map(|(b, s)| (b - buy_signal).powi(2) + (s - sell_signal).powi(2))
            .sum::<f64>()
            / total_signals;
        let reliability_score = (1.0 - signal_variance.sqrt()).max(0.0);

        // 시장 조건 점수 (간소화)
        let market_condition_score = (consensus_score + reliability_score) / 2.0;

        // 종합 점수
        let overall_score = consensus_score * 0.3
            + strength_score * 0.3
            + reliability_score * 0.2
            + market_condition_score * 0.2;

        SignalQuality {
            consensus_score,
            strength_score,
            reliability_score,
            market_condition_score,
            overall_score,
        }
    }

    /// 분석기 기여도 계산
    fn calculate_analyzer_contributions(
        &self,
        analyzer_signals: &[(f64, f64)],
    ) -> Vec<(String, f64)> {
        let analyzer_names = [
            "Trend",
            "Momentum",
            "Volatility",
            "Volume",
            "Support/Resistance",
            "Price Action",
            "Market Structure",
            "Risk Management",
            "Candle Pattern",
        ];

        let mut contributions = Vec::new();
        for (i, (buy, sell)) in analyzer_signals.iter().enumerate() {
            let max_signal = buy.max(*sell);
            if i < analyzer_names.len() {
                contributions.push((analyzer_names[i].to_string(), max_signal));
            }
        }

        contributions
    }

    /// 시장 상황 평가
    fn evaluate_market_conditions(&self, signal_quality: &SignalQuality) -> String {
        match signal_quality.overall_score {
            s if s >= 0.8 => "매우 좋음 - 높은 신뢰도의 신호 환경".to_string(),
            s if s >= 0.6 => "좋음 - 신뢰할 만한 신호 환경".to_string(),
            s if s >= 0.4 => "보통 - 신중한 접근 필요".to_string(),
            s if s >= 0.2 => "나쁨 - 신호 불일치, 관망 권장".to_string(),
            _ => "매우 나쁨 - 거래 지양 권장".to_string(),
        }
    }

    /// 추천 액션 생성
    fn generate_recommended_action(
        &self,
        signal_direction: &SignalDirection,
        signal_quality: &SignalQuality,
    ) -> String {
        match signal_direction {
            SignalDirection::StrongBuy if signal_quality.overall_score > 0.7 => {
                "적극적인 매수 포지션 진입 권장".to_string()
            }
            SignalDirection::Buy if signal_quality.overall_score > 0.5 => {
                "신중한 매수 포지션 진입 검토".to_string()
            }
            SignalDirection::StrongSell if signal_quality.overall_score > 0.7 => {
                "적극적인 매도 포지션 진입 권장".to_string()
            }
            SignalDirection::Sell if signal_quality.overall_score > 0.5 => {
                "신중한 매도 포지션 진입 검토".to_string()
            }
            SignalDirection::Neutral => "관망 또는 기존 포지션 유지 권장".to_string(),
            _ => "신호 품질 부족으로 거래 지양 권장".to_string(),
        }
    }

    /// 신호 안정성 계산
    fn calculate_signal_stability(&self, signal_history: &[SignalAnalysis]) -> f64 {
        if signal_history.len() < 5 {
            return 0.5;
        }

        let recent_signals = &signal_history[..5.min(signal_history.len())];
        let direction_consistency = recent_signals
            .iter()
            .filter(|s| s.signal_direction == signal_history[0].signal_direction)
            .count() as f64
            / recent_signals.len() as f64;

        let strength_variance = {
            let strengths: Vec<f64> = recent_signals
                .iter()
                .map(|s| s.buy_signal_strength.max(s.sell_signal_strength))
                .collect();
            let mean = strengths.iter().sum::<f64>() / strengths.len() as f64;
            let variance =
                strengths.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / strengths.len() as f64;
            1.0 - variance.sqrt().min(1.0)
        };

        (direction_consistency + strength_variance) / 2.0
    }

    /// 시장 상황 변화율 계산
    fn calculate_market_condition_change_rate(&self, signal_history: &[SignalAnalysis]) -> f64 {
        if signal_history.len() < 2 {
            return 0.0;
        }

        let current_condition = signal_history[0].signal_quality.market_condition_score;
        let previous_condition = signal_history[1].signal_quality.market_condition_score;

        (current_condition - previous_condition) / previous_condition.max(0.01)
    }

    /// 단순 이동평균 계산
    fn calculate_sma(&self, candles: &[C]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }
        candles.iter().map(|c| c.close_price()).sum::<f64>() / candles.len() as f64
    }

    /// RSI 계산 (간소화)
    fn calculate_rsi(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 50.0;
        }

        let gains_losses: Vec<f64> = candles
            .windows(2)
            .map(|w| w[0].close_price() - w[1].close_price())
            .collect();

        let avg_gain =
            gains_losses.iter().filter(|&&x| x > 0.0).sum::<f64>() / gains_losses.len() as f64;
        let avg_loss = gains_losses
            .iter()
            .filter(|&&x| x < 0.0)
            .map(|x| x.abs())
            .sum::<f64>()
            / gains_losses.len() as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// 모멘텀 계산
    fn calculate_momentum(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let current_price = candles[0].close_price();
        let past_price = candles[candles.len() - 1].close_price();

        (current_price - past_price) / past_price
    }

    /// 변동성 계산
    fn calculate_volatility(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = candles
            .windows(2)
            .map(|w| (w[0].close_price() - w[1].close_price()) / w[1].close_price())
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

        variance.sqrt()
    }

    /// ATR 계산
    fn calculate_atr(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let true_ranges: Vec<f64> = candles
            .windows(2)
            .map(|w| {
                let current = &w[0];
                let previous = &w[1];
                let tr1 = current.high_price() - current.low_price();
                let tr2 = (current.high_price() - previous.close_price()).abs();
                let tr3 = (current.low_price() - previous.close_price()).abs();
                tr1.max(tr2).max(tr3)
            })
            .collect();

        true_ranges.iter().sum::<f64>() / true_ranges.len() as f64
    }

    /// 강한 매수 신호 확인
    pub fn is_strong_buy_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_buy_signal()
        } else {
            false
        }
    }

    /// 강한 매도 신호 확인
    pub fn is_strong_sell_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_sell_signal()
        } else {
            false
        }
    }

    /// 고품질 신호 확인
    pub fn is_high_quality_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_high_quality_signal()
        } else {
            false
        }
    }

    /// 일관된 신호 확인
    pub fn is_consistent_signal(&self, lookback: usize) -> bool {
        if let Some(data) = self.items.first() {
            data.is_consistent_signal(lookback)
        } else {
            false
        }
    }

    /// 강한 매수 신호 확인 (n개 연속 강한 매수 신호, 이전 m개는 아님)
    pub fn is_strong_buy_signal_confirmed(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_buy_signal(), n, m)
    }

    /// 강한 매도 신호 확인 (n개 연속 강한 매도 신호, 이전 m개는 아님)
    pub fn is_strong_sell_signal_confirmed(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_sell_signal(), n, m)
    }

    /// 높은 강도 신호 확인 (n개 연속 높은 강도 신호, 이전 m개는 아님)
    pub fn is_high_strength_signal_confirmed(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_strength_signal(), n, m)
    }

    /// 높은 품질 신호 확인 (n개 연속 높은 품질 신호, 이전 m개는 아님)
    pub fn is_high_quality_signal_confirmed(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_quality_signal(), n, m)
    }

    /// 안정적인 신호 확인 (n개 연속 안정적인 신호, 이전 m개는 아님)
    pub fn is_stable_signal_confirmed(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_stable_signal(), n, m)
    }

    /// 좋은 시장 상황 신호 확인 (n개 연속 좋은 시장 상황, 이전 m개는 아님)
    pub fn is_good_market_condition_signal(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_good_market_condition(), n, m)
    }

    /// 일관된 신호 확인 (n개 연속 일관된 신호, 이전 m개는 아님)
    pub fn is_consistent_signal_confirmed(&self, n: usize, m: usize, lookback: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_consistent_signal(lookback), n, m)
    }

    /// 종합 신호 점수 임계값 돌파 확인 (n개 연속 임계값 초과, 이전 m개는 아님)
    pub fn is_overall_signal_score_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.calculate_overall_signal_score() > threshold,
            n,
            m,
        )
    }

    /// 신호 강도 임계값 돌파 확인 (n개 연속 매수 신호 강도 임계값 초과, 이전 m개는 아님)
    pub fn is_buy_signal_strength_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.signal_analysis.buy_signal_strength > threshold,
            n,
            m,
        )
    }

    /// 신호 강도 임계값 돌파 확인 (n개 연속 매도 신호 강도 임계값 초과, 이전 m개는 아님)
    pub fn is_sell_signal_strength_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.signal_analysis.sell_signal_strength > threshold,
            n,
            m,
        )
    }

    /// 신호 품질 임계값 돌파 확인 (n개 연속 품질 점수 임계값 초과, 이전 m개는 아님)
    pub fn is_signal_quality_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.signal_analysis.signal_quality.overall_score > threshold,
            n,
            m,
        )
    }

    /// 신호 일치도 임계값 돌파 확인 (n개 연속 일치도 임계값 초과, 이전 m개는 아님)
    pub fn is_consensus_score_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.signal_analysis.signal_quality.consensus_score > threshold,
            n,
            m,
        )
    }

    /// 신호 안정성 임계값 돌파 확인 (n개 연속 안정성 점수 임계값 초과, 이전 m개는 아님)
    pub fn is_signal_stability_breakthrough(&self, n: usize, m: usize, threshold: f64) -> bool {
        self.is_break_through_by_satisfying(|data| data.signal_stability_score > threshold, n, m)
    }

    /// n개의 연속 데이터에서 강한 매수 신호인지 확인
    pub fn is_strong_buy_signal_continuous(&self, n: usize) -> bool {
        self.is_all(|data| data.is_strong_buy_signal(), n)
    }

    /// n개의 연속 데이터에서 강한 매도 신호인지 확인
    pub fn is_strong_sell_signal_continuous(&self, n: usize) -> bool {
        self.is_all(|data| data.is_strong_sell_signal(), n)
    }

    /// n개의 연속 데이터에서 높은 강도 신호인지 확인
    pub fn is_high_strength_signal(&self, n: usize) -> bool {
        self.is_all(|data| data.is_high_strength_signal(), n)
    }

    /// n개의 연속 데이터에서 높은 품질 신호인지 확인
    pub fn is_high_quality_signal_continuous(&self, n: usize) -> bool {
        self.is_all(|data| data.is_high_quality_signal(), n)
    }

    /// n개의 연속 데이터에서 안정적인 신호인지 확인
    pub fn is_stable_signal(&self, n: usize) -> bool {
        self.is_all(|data| data.is_stable_signal(), n)
    }

    /// n개의 연속 데이터에서 좋은 시장 상황인지 확인
    pub fn is_good_market_condition(&self, n: usize) -> bool {
        self.is_all(|data| data.is_good_market_condition(), n)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<SignalStrengthAnalyzerData<C>, C>
    for SignalStrengthAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> SignalStrengthAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = 50;
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 각 분석기별 신호 계산
        let trend_signals = self.calculate_trend_signals(&recent_candles);
        let momentum_signals = self.calculate_momentum_signals(&recent_candles);
        let volatility_signals = self.calculate_volatility_signals(&recent_candles);
        let volume_signals = self.calculate_volume_signals(&recent_candles);
        let support_resistance_signals = self.calculate_support_resistance_signals(&recent_candles);

        // 간소화된 나머지 신호들 (실제 구현에서는 해당 분석기들을 사용)
        let price_action_signals = (0.3, 0.2); // 예시 값
        let market_structure_signals = (0.4, 0.3); // 예시 값
        let risk_management_signals = (0.2, 0.1); // 예시 값
        let candle_pattern_signals = (0.1, 0.2); // 예시 값

        let analyzer_signals = vec![
            trend_signals,
            momentum_signals,
            volatility_signals,
            volume_signals,
            support_resistance_signals,
            price_action_signals,
            market_structure_signals,
            risk_management_signals,
            candle_pattern_signals,
        ];

        // 가중치 적용하여 종합 신호 계산
        let weights = [
            self.weights.trend_weight,
            self.weights.momentum_weight,
            self.weights.volatility_weight,
            self.weights.volume_weight,
            self.weights.support_resistance_weight,
            self.weights.price_action_weight,
            self.weights.market_structure_weight,
            self.weights.risk_management_weight,
            self.weights.candle_pattern_weight,
        ];

        let weighted_buy_signal = analyzer_signals
            .iter()
            .zip(weights.iter())
            .map(|((buy, _), weight)| buy * weight)
            .sum::<f64>();

        let weighted_sell_signal = analyzer_signals
            .iter()
            .zip(weights.iter())
            .map(|((_, sell), weight)| sell * weight)
            .sum::<f64>();

        let signal_direction =
            self.calculate_signal_direction(weighted_buy_signal, weighted_sell_signal);
        let strength_level =
            self.calculate_strength_level(weighted_buy_signal.max(weighted_sell_signal));
        let signal_quality = self.calculate_signal_quality(
            weighted_buy_signal,
            weighted_sell_signal,
            &analyzer_signals,
        );
        let analyzer_contributions = self.calculate_analyzer_contributions(&analyzer_signals);
        let market_conditions = self.evaluate_market_conditions(&signal_quality);
        let recommended_action =
            self.generate_recommended_action(&signal_direction, &signal_quality);

        let signal_analysis = SignalAnalysis {
            buy_signal_strength: weighted_buy_signal,
            sell_signal_strength: weighted_sell_signal,
            signal_direction,
            strength_level,
            signal_quality,
            analyzer_contributions,
            market_conditions,
            recommended_action,
        };

        // 신호 히스토리 수집
        let signal_history: Vec<SignalAnalysis> = self
            .items
            .iter()
            .take(self.signal_history_length)
            .map(|item| item.signal_analysis.clone())
            .collect();

        let signal_stability_score = self.calculate_signal_stability(&signal_history);
        let market_condition_change_rate =
            self.calculate_market_condition_change_rate(&signal_history);

        SignalStrengthAnalyzerData::new(
            candle,
            signal_analysis,
            signal_history,
            signal_stability_score,
            market_condition_change_rate,
        )
    }

    fn datum(&self) -> &Vec<SignalStrengthAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<SignalStrengthAnalyzerData<C>> {
        &mut self.items
    }
}
