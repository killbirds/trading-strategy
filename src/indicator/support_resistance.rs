use trading_chart::Candle;

/// 지지/저항 레벨 데이터
#[derive(Debug, Clone)]
pub struct SupportResistanceLevel {
    /// 레벨 가격
    pub price: f64,
    /// 터치 횟수 (강도)
    pub touch_count: usize,
    /// 레벨 타입 (지지/저항)
    pub level_type: LevelType,
    /// 마지막 터치 인덱스
    pub last_touch_index: usize,
    /// 신뢰도 점수
    pub confidence_score: f64,
}

/// 레벨 타입
#[derive(Debug, Clone, PartialEq)]
pub enum LevelType {
    Support,
    Resistance,
    Both,
}

pub fn identify_levels<C: Candle>(
    candles: &[C],
    touch_threshold: f64,
    min_touch_count: usize,
) -> Vec<SupportResistanceLevel> {
    let mut levels = Vec::new();
    let mut potential_levels = Vec::new();

    if candles.len() < 5 {
        return levels;
    }

    for i in 2..candles.len() - 2 {
        let current_high = candles[i].high_price();
        let current_low = candles[i].low_price();

        if current_high > candles[i - 1].high_price()
            && current_high > candles[i - 2].high_price()
            && current_high > candles[i + 1].high_price()
            && current_high > candles[i + 2].high_price()
        {
            potential_levels.push((current_high, LevelType::Resistance, i));
        }

        if current_low < candles[i - 1].low_price()
            && current_low < candles[i - 2].low_price()
            && current_low < candles[i + 1].low_price()
            && current_low < candles[i + 2].low_price()
        {
            potential_levels.push((current_low, LevelType::Support, i));
        }
    }

    for (price, level_type, index) in potential_levels {
        let mut touch_count = 1;
        let mut last_touch_index = index;

        for (j, candle) in candles.iter().enumerate() {
            if j == index {
                continue;
            }

            let is_touch = match level_type {
                LevelType::Support => (candle.low_price() - price).abs() <= touch_threshold,
                LevelType::Resistance => (candle.high_price() - price).abs() <= touch_threshold,
                LevelType::Both => {
                    (candle.low_price() - price).abs() <= touch_threshold
                        || (candle.high_price() - price).abs() <= touch_threshold
                }
            };

            if is_touch {
                touch_count += 1;
                last_touch_index = j;
            }
        }

        if touch_count >= min_touch_count {
            let confidence_score =
                calculate_confidence_score(touch_count, last_touch_index, candles.len());

            levels.push(SupportResistanceLevel {
                price,
                touch_count,
                level_type,
                last_touch_index,
                confidence_score,
            });
        }
    }

    levels
}

pub fn calculate_confidence_score(
    touch_count: usize,
    last_touch_index: usize,
    total_candles: usize,
) -> f64 {
    let touch_score = (touch_count as f64 - 1.0) * 0.2;
    let recency_denominator = total_candles.saturating_sub(1).max(1) as f64;
    let recency_score = 1.0 - (last_touch_index as f64 / recency_denominator);
    (touch_score + recency_score * 0.5).min(1.0)
}

pub fn find_nearest_levels(
    current_price: f64,
    levels: &[SupportResistanceLevel],
) -> (
    Option<SupportResistanceLevel>,
    Option<SupportResistanceLevel>,
) {
    let mut nearest_support = None;
    let mut nearest_resistance = None;
    let mut min_support_distance = f64::MAX;
    let mut min_resistance_distance = f64::MAX;

    for level in levels {
        let distance = (current_price - level.price).abs();

        match level.level_type {
            LevelType::Support => {
                if level.price < current_price && distance < min_support_distance {
                    min_support_distance = distance;
                    nearest_support = Some(level.clone());
                }
            }
            LevelType::Resistance => {
                if level.price > current_price && distance < min_resistance_distance {
                    min_resistance_distance = distance;
                    nearest_resistance = Some(level.clone());
                }
            }
            LevelType::Both => {
                if level.price < current_price && distance < min_support_distance {
                    min_support_distance = distance;
                    nearest_support = Some(level.clone());
                }
                if level.price > current_price && distance < min_resistance_distance {
                    min_resistance_distance = distance;
                    nearest_resistance = Some(level.clone());
                }
            }
        }
    }

    (nearest_support, nearest_resistance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn finds_nearest_support_and_resistance() {
        let levels = vec![
            SupportResistanceLevel {
                price: 95.0,
                touch_count: 2,
                level_type: LevelType::Support,
                last_touch_index: 1,
                confidence_score: 0.5,
            },
            SupportResistanceLevel {
                price: 110.0,
                touch_count: 2,
                level_type: LevelType::Resistance,
                last_touch_index: 2,
                confidence_score: 0.5,
            },
        ];

        let (support, resistance) = find_nearest_levels(100.0, &levels);

        assert_eq!(support.unwrap().price, 95.0);
        assert_eq!(resistance.unwrap().price, 110.0);
    }

    #[test]
    fn identifies_pivot_levels_with_touch_count() {
        let candles = vec![
            TestCandle {
                timestamp: 1,
                open: 10.0,
                high: 10.0,
                low: 8.0,
                close: 9.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 8.5,
                close: 9.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 3,
                open: 10.0,
                high: 15.0,
                low: 9.0,
                close: 14.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 8.5,
                close: 9.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 5,
                open: 10.0,
                high: 10.0,
                low: 8.0,
                close: 9.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 6,
                open: 10.0,
                high: 15.0,
                low: 8.0,
                close: 9.0,
                volume: 1.0,
            },
        ];

        let levels = identify_levels(&candles, 0.0, 2);

        assert!(levels.iter().any(|level| level.price == 15.0));
    }
}
