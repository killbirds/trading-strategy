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

pub fn analyze_pattern<C: Candle>(
    candles_newest_first: &[C],
    min_body_ratio: f64,
    min_shadow_ratio: f64,
) -> PatternAnalysis {
    let single_pattern = candles_newest_first
        .first()
        .map(|candle| identify_single_candle_pattern(candle, min_body_ratio, min_shadow_ratio))
        .unwrap_or(SingleCandlePattern::Normal);
    let multi_pattern = identify_multi_candle_pattern(candles_newest_first);
    let reliability = calculate_pattern_reliability(&single_pattern, &multi_pattern);
    let signal = calculate_pattern_signal(&single_pattern, &multi_pattern);
    let volume_confirmation = check_volume_confirmation(candles_newest_first);
    let trend_alignment = check_trend_alignment(candles_newest_first);
    let confidence_score =
        calculate_confidence_score(&reliability, volume_confirmation, trend_alignment);
    let pattern_strength = calculate_pattern_strength(candles_newest_first);

    PatternAnalysis {
        single_pattern,
        multi_pattern,
        reliability,
        signal,
        confidence_score,
        pattern_strength,
        volume_confirmation,
        trend_alignment,
    }
}

pub fn identify_single_candle_pattern<C: Candle>(
    candle: &C,
    min_body_ratio: f64,
    min_shadow_ratio: f64,
) -> SingleCandlePattern {
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

    if body_ratio < min_body_ratio {
        if upper_shadow_ratio > 0.6 && lower_shadow_ratio < 0.1 {
            return SingleCandlePattern::GravestoneDoji;
        }
        if lower_shadow_ratio > 0.6 && upper_shadow_ratio < 0.1 {
            return SingleCandlePattern::DragonFlyDoji;
        }
        return SingleCandlePattern::Doji;
    }

    if lower_shadow_ratio > min_shadow_ratio * 2.0
        && upper_shadow_ratio < min_shadow_ratio
        && body_ratio < 0.3
    {
        if close > open {
            return SingleCandlePattern::Hammer;
        } else {
            return SingleCandlePattern::HangingMan;
        }
    }

    if upper_shadow_ratio > min_shadow_ratio * 2.0
        && lower_shadow_ratio < min_shadow_ratio
        && body_ratio < 0.3
    {
        if close > open {
            return SingleCandlePattern::InvertedHammer;
        } else {
            return SingleCandlePattern::ShootingStar;
        }
    }

    if body_ratio > 0.9 && upper_shadow_ratio < 0.05 && lower_shadow_ratio < 0.05 {
        return SingleCandlePattern::Marubozu;
    }

    if body_ratio < 0.3 && upper_shadow_ratio > 0.2 && lower_shadow_ratio > 0.2 {
        return SingleCandlePattern::SpinningTop;
    }

    SingleCandlePattern::Normal
}

pub fn identify_multi_candle_pattern<C: Candle>(candles: &[C]) -> MultiCandlePattern {
    if candles.len() < 2 {
        return MultiCandlePattern::None;
    }

    if let (Some(prev), Some(curr)) = (candles.get(1), candles.first()) {
        if is_engulfing_pattern(prev, curr) {
            if curr.close_price() > curr.open_price() {
                return MultiCandlePattern::BullishEngulfing;
            } else {
                return MultiCandlePattern::BearishEngulfing;
            }
        }

        if is_piercing_pattern(prev, curr) {
            return MultiCandlePattern::PiercingPattern;
        }

        if is_dark_cloud_cover(prev, curr) {
            return MultiCandlePattern::DarkCloudCover;
        }

        if is_harami_pattern(prev, curr) {
            return MultiCandlePattern::Harami;
        }

        if is_tweezer_pattern(prev, curr) {
            if prev.close_price() < prev.open_price() && curr.close_price() > curr.open_price() {
                return MultiCandlePattern::TweezerBottom;
            } else if prev.close_price() > prev.open_price()
                && curr.close_price() < curr.open_price()
            {
                return MultiCandlePattern::TweezerTop;
            }
        }
    }

    if let (Some(third), Some(second), Some(first)) =
        (candles.get(2), candles.get(1), candles.first())
    {
        if is_morning_star_pattern(third, second, first) {
            return MultiCandlePattern::MorningStar;
        }

        if is_evening_star_pattern(third, second, first) {
            return MultiCandlePattern::EveningStar;
        }

        if is_three_white_soldiers(third, second, first) {
            return MultiCandlePattern::ThreeWhiteSoldiers;
        }

        if is_three_black_crows(third, second, first) {
            return MultiCandlePattern::ThreeBlackCrows;
        }

        if is_three_inside_up(third, second, first) {
            return MultiCandlePattern::ThreeInsideUp;
        }

        if is_three_inside_down(third, second, first) {
            return MultiCandlePattern::ThreeInsideDown;
        }
    }

    MultiCandlePattern::None
}

pub fn is_engulfing_pattern<C: Candle>(prev: &C, curr: &C) -> bool {
    let prev_body_top = prev.close_price().max(prev.open_price());
    let prev_body_bottom = prev.close_price().min(prev.open_price());
    let curr_body_top = curr.close_price().max(curr.open_price());
    let curr_body_bottom = curr.close_price().min(curr.open_price());

    curr_body_top > prev_body_top
        && curr_body_bottom < prev_body_bottom
        && (prev.close_price() > prev.open_price()) != (curr.close_price() > curr.open_price())
}

pub fn is_piercing_pattern<C: Candle>(prev: &C, curr: &C) -> bool {
    prev.close_price() < prev.open_price()
        && curr.close_price() > curr.open_price()
        && curr.open_price() < prev.close_price()
        && curr.close_price() > (prev.open_price() + prev.close_price()) / 2.0
        && curr.close_price() < prev.open_price()
}

pub fn is_dark_cloud_cover<C: Candle>(prev: &C, curr: &C) -> bool {
    prev.close_price() > prev.open_price()
        && curr.close_price() < curr.open_price()
        && curr.open_price() > prev.close_price()
        && curr.close_price() < (prev.open_price() + prev.close_price()) / 2.0
        && curr.close_price() > prev.open_price()
}

pub fn is_harami_pattern<C: Candle>(prev: &C, curr: &C) -> bool {
    let prev_body_top = prev.close_price().max(prev.open_price());
    let prev_body_bottom = prev.close_price().min(prev.open_price());
    let curr_body_top = curr.close_price().max(curr.open_price());
    let curr_body_bottom = curr.close_price().min(curr.open_price());

    curr_body_top < prev_body_top
        && curr_body_bottom > prev_body_bottom
        && (prev.close_price() - prev.open_price()).abs()
            > (curr.close_price() - curr.open_price()).abs()
}

pub fn is_tweezer_pattern<C: Candle>(prev: &C, curr: &C) -> bool {
    let high_diff = (prev.high_price() - curr.high_price()).abs();
    let low_diff = (prev.low_price() - curr.low_price()).abs();
    let price_tolerance = (prev.high_price() - prev.low_price()) * 0.01;

    high_diff < price_tolerance || low_diff < price_tolerance
}

pub fn is_morning_star_pattern<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    let first_bearish = first.close_price() < first.open_price();
    let first_body_size = (first.close_price() - first.open_price()).abs();
    let second_body_size = (second.close_price() - second.open_price()).abs();
    let second_small = second_body_size < first_body_size * 0.3;
    let third_bullish = third.close_price() > third.open_price();
    let third_body_size = (third.close_price() - third.open_price()).abs();
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

pub fn is_evening_star_pattern<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    let first_bullish = first.close_price() > first.open_price();
    let first_body_size = (first.close_price() - first.open_price()).abs();
    let second_body_size = (second.close_price() - second.open_price()).abs();
    let second_small = second_body_size < first_body_size * 0.3;
    let third_bearish = third.close_price() < third.open_price();
    let third_body_size = (third.close_price() - third.open_price()).abs();
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

pub fn is_three_white_soldiers<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    let all_bullish = first.close_price() > first.open_price()
        && second.close_price() > second.open_price()
        && third.close_price() > third.open_price();
    let consecutive_higher =
        first.close_price() < second.close_price() && second.close_price() < third.close_price();
    let proper_opens = second.open_price() > first.open_price()
        && second.open_price() < first.close_price()
        && third.open_price() > second.open_price()
        && third.open_price() < second.close_price();

    all_bullish && consecutive_higher && proper_opens
}

pub fn is_three_black_crows<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    let all_bearish = first.close_price() < first.open_price()
        && second.close_price() < second.open_price()
        && third.close_price() < third.open_price();
    let consecutive_lower =
        first.close_price() > second.close_price() && second.close_price() > third.close_price();
    let proper_opens = second.open_price() < first.open_price()
        && second.open_price() > first.close_price()
        && third.open_price() < second.open_price()
        && third.open_price() > second.close_price();

    all_bearish && consecutive_lower && proper_opens
}

pub fn is_three_inside_up<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    is_harami_pattern(first, second)
        && first.close_price() < first.open_price()
        && second.close_price() > second.open_price()
        && third.close_price() > third.open_price()
        && third.close_price() > first.close_price()
}

pub fn is_three_inside_down<C: Candle>(first: &C, second: &C, third: &C) -> bool {
    is_harami_pattern(first, second)
        && first.close_price() > first.open_price()
        && second.close_price() < second.open_price()
        && third.close_price() < third.open_price()
        && third.close_price() < first.close_price()
}

pub fn calculate_pattern_reliability(
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

pub fn calculate_pattern_signal(
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
        MultiCandlePattern::None => match single_pattern {
            SingleCandlePattern::Hammer
            | SingleCandlePattern::InvertedHammer
            | SingleCandlePattern::DragonFlyDoji => PatternSignal::Bullish,
            SingleCandlePattern::HangingMan
            | SingleCandlePattern::ShootingStar
            | SingleCandlePattern::GravestoneDoji => PatternSignal::Bearish,
            SingleCandlePattern::Marubozu
            | SingleCandlePattern::Doji
            | SingleCandlePattern::SpinningTop
            | SingleCandlePattern::Normal => PatternSignal::Neutral,
        },
    }
}

pub fn calculate_confidence_score(
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

pub fn calculate_pattern_strength<C: Candle>(candles: &[C]) -> f64 {
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

pub fn check_volume_confirmation<C: Candle>(candles: &[C]) -> bool {
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

pub fn check_trend_alignment<C: Candle>(candles: &[C]) -> bool {
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

pub fn calculate_pattern_continuity_score(recent_patterns: &[PatternAnalysis]) -> f64 {
    if recent_patterns.is_empty() {
        return 0.0;
    }

    let consistent_signals = recent_patterns
        .iter()
        .filter(|p| p.signal != PatternSignal::Neutral)
        .count();

    (consistent_signals as f64 / recent_patterns.len() as f64).min(1.0)
}

pub fn calculate_market_context_score<C: Candle>(candles: &[C]) -> f64 {
    if candles.len() < 10 {
        return 0.5;
    }

    let recent_prices: Vec<f64> = candles.iter().take(10).map(|c| c.close_price()).collect();
    let volatility = calculate_volatility(&recent_prices);
    let trend_strength = calculate_trend_strength(&recent_prices);

    (1.0 - volatility + trend_strength) / 2.0
}

pub fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let returns: Vec<f64> = prices.windows(2).map(|w| (w[0] - w[1]) / w[1]).collect();
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

    variance.sqrt().min(1.0)
}

pub fn calculate_trend_strength(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let first_price = prices[prices.len() - 1];
    let last_price = prices[0];
    let price_change = (last_price - first_price).abs() / first_price;

    price_change.min(1.0)
}

pub fn calculate_pattern_clustering_score(
    recent_patterns: &[PatternAnalysis],
    current_signal: &PatternSignal,
) -> f64 {
    let similar_patterns = recent_patterns
        .iter()
        .filter(|p| &p.signal == current_signal)
        .count();

    (similar_patterns as f64 / recent_patterns.len().max(1) as f64).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn identifies_single_candle_hammer() {
        let candle = TestCandle {
            timestamp: 1,
            open: 10.0,
            high: 10.6,
            low: 8.0,
            close: 10.5,
            volume: 1.0,
        };

        assert_eq!(
            identify_single_candle_pattern(&candle, 0.1, 0.3),
            SingleCandlePattern::Hammer
        );
    }

    #[test]
    fn identifies_bullish_engulfing() {
        let candles = vec![
            TestCandle {
                timestamp: 2,
                open: 8.0,
                high: 12.0,
                low: 7.0,
                close: 12.0,
                volume: 2.0,
            },
            TestCandle {
                timestamp: 1,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 9.0,
                volume: 1.0,
            },
        ];

        assert_eq!(
            identify_multi_candle_pattern(&candles),
            MultiCandlePattern::BullishEngulfing
        );
    }

    #[test]
    fn analyzes_pattern_with_volume_confirmation() {
        let candles = vec![
            TestCandle {
                timestamp: 2,
                open: 8.0,
                high: 12.0,
                low: 7.0,
                close: 12.0,
                volume: 2.0,
            },
            TestCandle {
                timestamp: 1,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 9.0,
                volume: 1.0,
            },
        ];

        let analysis = analyze_pattern(&candles, 0.1, 0.3);

        assert_eq!(analysis.multi_pattern, MultiCandlePattern::BullishEngulfing);
        assert!(analysis.volume_confirmation);
        assert_eq!(analysis.signal, PatternSignal::StrongBullish);
    }
}
