use trading_chart::Candle;

/// 캔들 패턴 타입
#[derive(Debug, Clone, PartialEq)]
pub enum CandlePattern {
    Hammer,
    InvertedHammer,
    Doji,
    BullishEngulfing,
    BearishEngulfing,
    PiercingPattern,
    DarkCloudCover,
    MorningStar,
    EveningStar,
    LongBullish,
    LongBearish,
    Normal,
}

/// 가격 추세 타입
#[derive(Debug, Clone, PartialEq)]
pub enum PriceTrend {
    StrongUptrend,
    WeakUptrend,
    Sideways,
    WeakDowntrend,
    StrongDowntrend,
}

/// 스윙 포인트 타입
#[derive(Debug, Clone)]
pub struct SwingPoint {
    pub index: usize,
    pub price: f64,
    pub swing_type: SwingType,
    pub strength: usize,
}

/// 스윙 타입
#[derive(Debug, Clone, PartialEq)]
pub enum SwingType {
    High,
    Low,
}

pub fn identify_candle_pattern<C: Candle>(candles: &[C]) -> CandlePattern {
    let current = match candles.first() {
        Some(c) => c,
        None => return CandlePattern::Normal,
    };
    let (body_ratio, upper_shadow_ratio, lower_shadow_ratio) = calculate_candle_ratios(current);

    if body_ratio < 0.1 {
        return CandlePattern::Doji;
    }

    if lower_shadow_ratio > 0.6 && upper_shadow_ratio < 0.1 && body_ratio < 0.3 {
        return CandlePattern::Hammer;
    }

    if upper_shadow_ratio > 0.6 && lower_shadow_ratio < 0.1 && body_ratio < 0.3 {
        return CandlePattern::InvertedHammer;
    }

    if let Some(previous) = candles.get(1) {
        let current_bullish = current.close_price() > current.open_price();
        let previous_bullish = previous.close_price() > previous.open_price();

        if current_bullish
            && !previous_bullish
            && current.open_price() < previous.close_price()
            && current.close_price() > previous.open_price()
        {
            return CandlePattern::BullishEngulfing;
        }

        if !current_bullish
            && previous_bullish
            && current.open_price() > previous.close_price()
            && current.close_price() < previous.open_price()
        {
            return CandlePattern::BearishEngulfing;
        }

        if current_bullish
            && !previous_bullish
            && current.open_price() < previous.close_price()
            && current.close_price() > (previous.open_price() + previous.close_price()) / 2.0
        {
            return CandlePattern::PiercingPattern;
        }

        if !current_bullish
            && previous_bullish
            && current.open_price() > previous.close_price()
            && current.close_price() < (previous.open_price() + previous.close_price()) / 2.0
        {
            return CandlePattern::DarkCloudCover;
        }
    }

    if body_ratio > 0.7 {
        if current.close_price() > current.open_price() {
            return CandlePattern::LongBullish;
        } else {
            return CandlePattern::LongBearish;
        }
    }

    CandlePattern::Normal
}

pub fn analyze_price_trend<C: Candle>(candles: &[C], trend_period: usize) -> PriceTrend {
    if candles.len() < trend_period {
        return PriceTrend::Sideways;
    }

    let recent_candles = &candles[..trend_period];
    let first_price = recent_candles
        .last()
        .map(|c| c.close_price())
        .unwrap_or(0.0);
    let last_price = recent_candles
        .first()
        .map(|c| c.close_price())
        .unwrap_or(0.0);
    let price_change = (last_price - first_price) / first_price;

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

pub fn identify_swing_points<C: Candle>(candles: &[C], strength: usize) -> Vec<SwingPoint> {
    let mut swing_points = Vec::new();

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

    swing_points.sort_by(|a, b| a.index.cmp(&b.index));
    swing_points.truncate(10);

    swing_points
}

pub fn calculate_avg_candle_size<C: Candle>(candles: &[C]) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }

    let total_size: f64 = candles
        .iter()
        .map(|c| (c.high_price() - c.low_price()).abs())
        .sum();
    total_size / candles.len() as f64
}

pub fn calculate_vwap<C: Candle>(candles: &[C]) -> f64 {
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

pub fn calculate_momentum<C: Candle>(candles: &[C], momentum_period: usize) -> f64 {
    if candles.len() < momentum_period {
        return 0.0;
    }

    let current_price = match candles.first() {
        Some(c) => c.close_price(),
        None => return 0.0,
    };
    let past_price = match candles.get(momentum_period - 1) {
        Some(c) => c.close_price(),
        None => return 0.0,
    };
    if past_price == 0.0 {
        return 0.0;
    }
    (current_price - past_price) / past_price
}

pub fn calculate_candle_ratios<C: Candle>(candle: &C) -> (f64, f64, f64) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn identifies_doji_before_other_patterns() {
        let candle = TestCandle {
            timestamp: 1,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 100.5,
            volume: 1.0,
        };

        assert_eq!(identify_candle_pattern(&[candle]), CandlePattern::Doji);
    }

    #[test]
    fn calculates_vwap_and_momentum() {
        let candles = vec![
            TestCandle {
                timestamp: 3,
                open: 120.0,
                high: 123.0,
                low: 117.0,
                close: 120.0,
                volume: 2.0,
            },
            TestCandle {
                timestamp: 2,
                open: 110.0,
                high: 113.0,
                low: 107.0,
                close: 110.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 1,
                open: 100.0,
                high: 103.0,
                low: 97.0,
                close: 100.0,
                volume: 1.0,
            },
        ];

        assert!((calculate_vwap(&candles) - 112.5).abs() < 1e-10);
        assert!((calculate_momentum(&candles, 3) - 0.2).abs() < 1e-10);
    }
}
