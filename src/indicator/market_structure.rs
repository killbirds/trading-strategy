use trading_chart::Candle;

/// 시장 구조 타입
#[derive(Debug, Clone, PartialEq)]
pub enum MarketStructure {
    Uptrend,
    Downtrend,
    Sideways,
    Uncertain,
}

/// 구조 변화 타입
#[derive(Debug, Clone, PartialEq)]
pub enum StructureChange {
    None,
    BullishBOS,
    BearishBOS,
    BullishCHoCH,
    BearishCHoCH,
}

/// Fair Value Gap (FVG) 타입
#[derive(Debug, Clone)]
pub struct FairValueGap {
    pub start_price: f64,
    pub end_price: f64,
    pub gap_type: FVGType,
    pub index: usize,
    pub size: f64,
}

/// Fair Value Gap 타입
#[derive(Debug, Clone, PartialEq)]
pub enum FVGType {
    Bullish,
    Bearish,
}

/// 오더 블록 타입
#[derive(Debug, Clone)]
pub struct OrderBlock {
    pub start_price: f64,
    pub end_price: f64,
    pub block_type: OrderBlockType,
    pub index: usize,
    pub strength: usize,
}

/// 오더 블록 타입
#[derive(Debug, Clone, PartialEq)]
pub enum OrderBlockType {
    Bullish,
    Bearish,
}

/// 유동성 풀 타입
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    pub price: f64,
    pub pool_type: LiquidityPoolType,
    pub index: usize,
    pub liquidity_amount: f64,
}

/// 유동성 풀 타입
#[derive(Debug, Clone, PartialEq)]
pub enum LiquidityPoolType {
    BuyLiquidity,
    SellLiquidity,
}

pub fn analyze_market_structure<C: Candle>(
    candles: &[C],
    swing_strength: usize,
    structure_period: usize,
) -> MarketStructure {
    if candles.len() < structure_period {
        return MarketStructure::Uncertain;
    }

    let swing_points = identify_swing_points(candles, swing_strength);
    if swing_points.len() < 4 {
        return MarketStructure::Uncertain;
    }

    let highs: Vec<f64> = swing_points
        .iter()
        .filter_map(|(_, price, is_high)| if *is_high { Some(*price) } else { None })
        .collect();
    let lows: Vec<f64> = swing_points
        .iter()
        .filter_map(|(_, price, is_high)| if !*is_high { Some(*price) } else { None })
        .collect();

    if highs.len() < 2 || lows.len() < 2 {
        return MarketStructure::Uncertain;
    }

    let higher_highs = highs.windows(2).all(|w| w[0] > w[1]);
    let higher_lows = lows.windows(2).all(|w| w[0] > w[1]);
    let lower_highs = highs.windows(2).all(|w| w[0] < w[1]);
    let lower_lows = lows.windows(2).all(|w| w[0] < w[1]);

    if higher_highs && higher_lows {
        MarketStructure::Uptrend
    } else if lower_highs && lower_lows {
        MarketStructure::Downtrend
    } else {
        MarketStructure::Sideways
    }
}

pub fn detect_structure_change<C: Candle>(
    candles: &[C],
    previous_structure: Option<&MarketStructure>,
    current_structure: MarketStructure,
) -> StructureChange {
    let Some(previous_structure) = previous_structure else {
        return StructureChange::None;
    };

    match (previous_structure, current_structure) {
        (MarketStructure::Uptrend, MarketStructure::Downtrend) => {
            if is_strong_reversal(candles) {
                StructureChange::BullishBOS
            } else {
                StructureChange::BearishCHoCH
            }
        }
        (MarketStructure::Downtrend, MarketStructure::Uptrend) => {
            if is_strong_reversal(candles) {
                StructureChange::BearishBOS
            } else {
                StructureChange::BullishCHoCH
            }
        }
        _ => StructureChange::None,
    }
}

pub fn is_strong_reversal<C: Candle>(candles: &[C]) -> bool {
    if candles.len() < 10 {
        return false;
    }

    let recent_volume: f64 = candles.iter().take(5).map(|c| c.volume()).sum();
    let avg_volume: f64 = candles
        .iter()
        .skip(5)
        .take(10)
        .map(|c| c.volume())
        .sum::<f64>()
        / 10.0;

    recent_volume > avg_volume * 1.5
}

pub fn identify_swing_points<C: Candle>(candles: &[C], strength: usize) -> Vec<(usize, f64, bool)> {
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
            swing_points.push((i, current.high_price(), true));
        }
        if is_swing_low {
            swing_points.push((i, current.low_price(), false));
        }
    }

    swing_points
}

pub fn identify_fair_value_gaps<C: Candle>(candles: &[C], min_fvg_size: f64) -> Vec<FairValueGap> {
    let mut fvgs = Vec::new();

    if candles.len() < 3 {
        return fvgs;
    }

    for i in 0..candles.len() - 2 {
        let candle1 = &candles[i + 2];
        let candle3 = &candles[i];

        if candle1.high_price() < candle3.low_price() {
            let gap_size = candle3.low_price() - candle1.high_price();
            if gap_size >= min_fvg_size {
                fvgs.push(FairValueGap {
                    start_price: candle1.high_price(),
                    end_price: candle3.low_price(),
                    gap_type: FVGType::Bullish,
                    index: i,
                    size: gap_size,
                });
            }
        }

        if candle1.low_price() > candle3.high_price() {
            let gap_size = candle1.low_price() - candle3.high_price();
            if gap_size >= min_fvg_size {
                fvgs.push(FairValueGap {
                    start_price: candle1.low_price(),
                    end_price: candle3.high_price(),
                    gap_type: FVGType::Bearish,
                    index: i,
                    size: gap_size,
                });
            }
        }
    }

    fvgs
}

pub fn identify_order_blocks<C: Candle>(
    candles: &[C],
    swing_strength: usize,
    min_order_block_size: f64,
) -> Vec<OrderBlock> {
    let mut order_blocks = Vec::new();
    let swing_points = identify_swing_points(candles, swing_strength);

    for (index, _price, is_high) in swing_points {
        if index >= candles.len() {
            continue;
        }

        let candle = &candles[index];
        let block_size = candle.high_price() - candle.low_price();

        if block_size >= min_order_block_size {
            let block_type = if is_high {
                OrderBlockType::Bearish
            } else {
                OrderBlockType::Bullish
            };

            order_blocks.push(OrderBlock {
                start_price: candle.low_price(),
                end_price: candle.high_price(),
                block_type,
                index,
                strength: 1,
            });
        }
    }

    order_blocks
}

pub fn identify_liquidity_pools<C: Candle>(
    candles: &[C],
    swing_strength: usize,
) -> Vec<LiquidityPool> {
    let mut liquidity_pools = Vec::new();
    let swing_points = identify_swing_points(candles, swing_strength);

    for (index, price, is_high) in swing_points {
        if index >= candles.len() {
            continue;
        }

        let candle = &candles[index];
        let pool_type = if is_high {
            LiquidityPoolType::SellLiquidity
        } else {
            LiquidityPoolType::BuyLiquidity
        };

        liquidity_pools.push(LiquidityPool {
            price,
            pool_type,
            index,
            liquidity_amount: candle.volume(),
        });
    }

    liquidity_pools
}

pub fn calculate_market_flow_strength<C: Candle>(candles: &[C]) -> f64 {
    if candles.len() < 10 {
        return 0.0;
    }

    let recent_candles = &candles[..10];
    let bullish_count = recent_candles
        .iter()
        .filter(|c| c.close_price() > c.open_price())
        .count();

    let volume_trend = calculate_volume_trend(recent_candles);
    let price_momentum = calculate_price_momentum(recent_candles);

    let bullish_ratio = bullish_count as f64 / recent_candles.len() as f64;

    (bullish_ratio + volume_trend + price_momentum) / 3.0
}

pub fn calculate_volume_trend<C: Candle>(candles: &[C]) -> f64 {
    if candles.len() < 2 {
        return 0.0;
    }

    let recent_volume: f64 = candles.iter().take(5).map(|c| c.volume()).sum();
    let past_volume: f64 = candles.iter().skip(5).map(|c| c.volume()).sum();

    if past_volume == 0.0 {
        return 0.0;
    }

    ((recent_volume - past_volume) / past_volume).clamp(-1.0, 1.0)
}

pub fn calculate_price_momentum<C: Candle>(candles: &[C]) -> f64 {
    if candles.len() < 2 {
        return 0.0;
    }

    let current_price = match candles.first() {
        Some(c) => c.close_price(),
        None => return 0.0,
    };
    let past_price = match candles.last() {
        Some(c) => c.close_price(),
        None => return 0.0,
    };

    if past_price == 0.0 {
        return 0.0;
    }

    ((current_price - past_price) / past_price).clamp(-1.0, 1.0)
}

pub fn calculate_imbalance_degree<C: Candle>(candles: &[C]) -> f64 {
    if candles.len() < 5 {
        return 0.0;
    }

    let recent_candles = &candles[..5];
    let mut imbalance_score = 0.0;

    for candle in recent_candles {
        let body_size = (candle.close_price() - candle.open_price()).abs();
        let total_size = candle.high_price() - candle.low_price();
        let upper_shadow = candle.high_price() - candle.close_price().max(candle.open_price());
        let lower_shadow = candle.close_price().min(candle.open_price()) - candle.low_price();

        if total_size > 0.0 {
            let body_ratio = body_size / total_size;
            let shadow_imbalance = (upper_shadow - lower_shadow).abs() / total_size;
            imbalance_score += body_ratio + shadow_imbalance;
        }
    }

    (imbalance_score / recent_candles.len() as f64).clamp(0.0, 1.0)
}

pub fn get_recent_swing_points<C: Candle>(
    candles: &[C],
    swing_strength: usize,
) -> (Option<f64>, Option<f64>) {
    let swing_points = identify_swing_points(candles, swing_strength);

    let recent_high = swing_points
        .iter()
        .filter(|(_, _, is_high)| *is_high)
        .map(|(_, price, _)| *price)
        .next();

    let recent_low = swing_points
        .iter()
        .filter(|(_, _, is_high)| !*is_high)
        .map(|(_, price, _)| *price)
        .next();

    (recent_high, recent_low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn identifies_bullish_fair_value_gap() {
        let candles = vec![
            TestCandle {
                timestamp: 3,
                open: 120.0,
                high: 125.0,
                low: 115.0,
                close: 122.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 2,
                open: 110.0,
                high: 112.0,
                low: 108.0,
                close: 111.0,
                volume: 1.0,
            },
            TestCandle {
                timestamp: 1,
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1.0,
            },
        ];

        let gaps = identify_fair_value_gaps(&candles, 1.0);

        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, FVGType::Bullish);
        assert_eq!(gaps[0].start_price, 105.0);
        assert_eq!(gaps[0].end_price, 115.0);
    }

    #[test]
    fn calculates_market_flow_strength_from_recent_ten_candles() {
        let candles: Vec<TestCandle> = (0..10)
            .map(|i| TestCandle {
                timestamp: i,
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 101.0 + i as f64,
                volume: 10.0,
            })
            .collect();

        assert!(calculate_market_flow_strength(&candles) > 0.0);
    }
}
