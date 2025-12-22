// Orderbook analysis indicator
// Provides tools for analyzing orderbook data to identify trading opportunities

use serde::{Deserialize, Serialize};

/// Orderbook entry representing a price level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    /// Price at this level
    pub price: f64,
    /// Quantity available at this price
    pub quantity: f64,
}

impl OrderBookLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self { price, quantity }
    }

    /// Calculate the total value (price * quantity) at this level
    pub fn value(&self) -> f64 {
        self.price * self.quantity
    }
}

/// Orderbook data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Symbol/market identifier
    pub symbol: String,
    /// Bid orders (buy orders) - sorted by price descending
    pub bids: Vec<OrderBookLevel>,
    /// Ask orders (sell orders) - sorted by price ascending
    pub asks: Vec<OrderBookLevel>,
    /// Timestamp of the orderbook snapshot
    pub timestamp: i64,
}

impl OrderBook {
    pub fn new(
        symbol: String,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol,
            bids,
            asks,
            timestamp,
        }
    }

    /// Create from tuple vectors (price, quantity)
    pub fn from_tuples(
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol,
            bids: bids
                .into_iter()
                .map(|(p, q)| OrderBookLevel::new(p, q))
                .collect(),
            asks: asks
                .into_iter()
                .map(|(p, q)| OrderBookLevel::new(p, q))
                .collect(),
            timestamp,
        }
    }

    /// Get best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    /// Get best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    /// Calculate mid price
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Calculate bid-ask spread
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Calculate spread as percentage of mid price
    pub fn spread_percent(&self) -> Option<f64> {
        match (self.spread(), self.mid_price()) {
            (Some(spread), Some(mid)) if mid > 0.0 => Some((spread / mid) * 100.0),
            _ => None,
        }
    }

    /// Calculate total bid depth (volume)
    pub fn bid_depth(&self) -> f64 {
        self.bids
            .iter()
            .filter_map(|l| {
                if l.quantity > 0.0 {
                    Some(l.quantity)
                } else {
                    None
                }
            })
            .sum()
    }

    /// Calculate total ask depth (volume)
    pub fn ask_depth(&self) -> f64 {
        self.asks
            .iter()
            .filter_map(|l| {
                if l.quantity > 0.0 {
                    Some(l.quantity)
                } else {
                    None
                }
            })
            .sum()
    }

    /// Calculate total bid value (price * quantity)
    pub fn bid_value(&self) -> f64 {
        self.bids
            .iter()
            .filter_map(|l| {
                if l.price > 0.0 && l.quantity > 0.0 {
                    Some(l.value())
                } else {
                    None
                }
            })
            .sum()
    }

    /// Calculate total ask value (price * quantity)
    pub fn ask_value(&self) -> f64 {
        self.asks
            .iter()
            .filter_map(|l| {
                if l.price > 0.0 && l.quantity > 0.0 {
                    Some(l.value())
                } else {
                    None
                }
            })
            .sum()
    }

    /// Calculate bid depth up to a certain price percentage from best bid
    pub fn bid_depth_within_percent(&self, percent: f64) -> f64 {
        if let Some(best_bid) = self.best_bid() {
            if best_bid <= 0.0 || percent < 0.0 {
                return 0.0;
            }
            let threshold = best_bid * (1.0 - percent / 100.0).max(0.0);
            self.bids
                .iter()
                .filter(|l| l.price >= threshold && l.quantity > 0.0)
                .map(|l| l.quantity)
                .sum()
        } else {
            0.0
        }
    }

    /// Calculate ask depth up to a certain price percentage from best ask
    pub fn ask_depth_within_percent(&self, percent: f64) -> f64 {
        if let Some(best_ask) = self.best_ask() {
            if best_ask <= 0.0 || percent < 0.0 {
                return 0.0;
            }
            let threshold = best_ask * (1.0 + percent / 100.0);
            self.asks
                .iter()
                .filter(|l| l.price <= threshold && l.quantity > 0.0)
                .map(|l| l.quantity)
                .sum()
        } else {
            0.0
        }
    }
}

/// Analysis result from orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookAnalysis {
    /// Best bid price
    pub best_bid: f64,
    /// Best ask price
    pub best_ask: f64,
    /// Bid-ask spread in absolute value
    pub spread: f64,
    /// Bid-ask spread as percentage
    pub spread_percent: f64,
    /// Mid price (average of best bid and ask)
    pub mid_price: f64,
    /// Total bid depth
    pub bid_depth: f64,
    /// Total ask depth
    pub ask_depth: f64,
    /// Order imbalance ratio: (bid_depth - ask_depth) / (bid_depth + ask_depth)
    /// Positive = more buy pressure, Negative = more sell pressure
    pub imbalance_ratio: f64,
    /// Buy/sell pressure indicator (-1.0 to 1.0)
    pub pressure: f64,
    /// Liquidity score (0.0 to 1.0)
    pub liquidity_score: f64,
    /// Suggested buy price based on analysis
    pub suggested_buy_price: f64,
    /// Suggested sell price based on analysis
    pub suggested_sell_price: f64,
    /// Estimated slippage for a given order size (percentage)
    pub estimated_buy_slippage: f64,
    /// Estimated slippage for a given sell order (percentage)
    pub estimated_sell_slippage: f64,
}

impl Default for OrderBookAnalysis {
    fn default() -> Self {
        Self {
            best_bid: 0.0,
            best_ask: 0.0,
            spread: 0.0,
            spread_percent: 0.0,
            mid_price: 0.0,
            bid_depth: 0.0,
            ask_depth: 0.0,
            imbalance_ratio: 0.0,
            pressure: 0.0,
            liquidity_score: 0.0,
            suggested_buy_price: 0.0,
            suggested_sell_price: 0.0,
            estimated_buy_slippage: 0.0,
            estimated_sell_slippage: 0.0,
        }
    }
}

/// Market pressure indicator based on orderbook
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketPressure {
    /// Strong buying pressure
    StrongBuy,
    /// Moderate buying pressure
    ModerateBuy,
    /// Neutral/balanced
    Neutral,
    /// Moderate selling pressure
    ModerateSell,
    /// Strong selling pressure
    StrongSell,
}

impl MarketPressure {
    /// Create from imbalance ratio
    pub fn from_imbalance(ratio: f64) -> Self {
        if ratio > 0.3 {
            MarketPressure::StrongBuy
        } else if ratio > 0.1 {
            MarketPressure::ModerateBuy
        } else if ratio < -0.3 {
            MarketPressure::StrongSell
        } else if ratio < -0.1 {
            MarketPressure::ModerateSell
        } else {
            MarketPressure::Neutral
        }
    }

    /// Check if this indicates buying pressure
    pub fn is_bullish(&self) -> bool {
        matches!(
            self,
            MarketPressure::StrongBuy | MarketPressure::ModerateBuy
        )
    }

    /// Check if this indicates selling pressure
    pub fn is_bearish(&self) -> bool {
        matches!(
            self,
            MarketPressure::StrongSell | MarketPressure::ModerateSell
        )
    }
}

/// Liquidity level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidityLevel {
    /// Very high liquidity - tight spread, deep orderbook
    VeryHigh,
    /// High liquidity
    High,
    /// Medium liquidity
    Medium,
    /// Low liquidity
    Low,
    /// Very low liquidity - wide spread, shallow orderbook
    VeryLow,
}

impl LiquidityLevel {
    /// Create from liquidity score (0.0 to 1.0)
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            LiquidityLevel::VeryHigh
        } else if score >= 0.6 {
            LiquidityLevel::High
        } else if score >= 0.4 {
            LiquidityLevel::Medium
        } else if score >= 0.2 {
            LiquidityLevel::Low
        } else {
            LiquidityLevel::VeryLow
        }
    }
}

/// Orderbook analyzer for calculating various metrics
#[derive(Debug, Clone)]
pub struct OrderBookAnalyzer {
    /// Depth percentage for calculating near-spread liquidity
    pub depth_percent: f64,
    /// Reference order size for slippage estimation (in quote currency)
    pub reference_order_size: f64,
    /// Maximum acceptable spread percentage
    pub max_spread_percent: f64,
}

impl Default for OrderBookAnalyzer {
    fn default() -> Self {
        Self {
            depth_percent: 1.0,                // 1% from best price
            reference_order_size: 1_000_000.0, // 1M KRW
            max_spread_percent: 1.0,           // 1% max spread
        }
    }
}

impl OrderBookAnalyzer {
    pub fn new(depth_percent: f64, reference_order_size: f64, max_spread_percent: f64) -> Self {
        Self {
            depth_percent,
            reference_order_size,
            max_spread_percent,
        }
    }

    /// Perform comprehensive orderbook analysis
    pub fn analyze(&self, orderbook: &OrderBook) -> OrderBookAnalysis {
        let best_bid = orderbook.best_bid().unwrap_or(0.0);
        let best_ask = orderbook.best_ask().unwrap_or(0.0);
        let spread = orderbook.spread().unwrap_or(0.0);
        let mid_price = orderbook.mid_price().unwrap_or(0.0);
        let spread_percent = orderbook.spread_percent().unwrap_or(0.0);

        // Validate basic orderbook integrity
        if best_bid <= 0.0 || best_ask <= 0.0 || best_ask <= best_bid {
            return OrderBookAnalysis::default();
        }

        // Calculate depth within percentage
        let bid_depth = orderbook.bid_depth_within_percent(self.depth_percent);
        let ask_depth = orderbook.ask_depth_within_percent(self.depth_percent);
        let total_depth = bid_depth + ask_depth;

        // Calculate imbalance ratio
        let imbalance_ratio = if total_depth > 0.0 {
            (bid_depth - ask_depth) / total_depth
        } else {
            0.0
        };

        // Calculate pressure (-1.0 to 1.0)
        let pressure = imbalance_ratio.clamp(-1.0, 1.0);

        // Calculate liquidity score based on spread and depth
        let liquidity_score = self.calculate_liquidity_score(orderbook);

        // Calculate suggested prices
        let (suggested_buy_price, suggested_sell_price) =
            self.calculate_suggested_prices(orderbook, imbalance_ratio);

        // Estimate slippage
        let estimated_buy_slippage =
            self.estimate_slippage(orderbook, self.reference_order_size, true);
        let estimated_sell_slippage =
            self.estimate_slippage(orderbook, self.reference_order_size, false);

        OrderBookAnalysis {
            best_bid,
            best_ask,
            spread,
            spread_percent,
            mid_price,
            bid_depth,
            ask_depth,
            imbalance_ratio,
            pressure,
            liquidity_score,
            suggested_buy_price,
            suggested_sell_price,
            estimated_buy_slippage,
            estimated_sell_slippage,
        }
    }

    /// Calculate liquidity score (0.0 to 1.0)
    fn calculate_liquidity_score(&self, orderbook: &OrderBook) -> f64 {
        let spread_percent = orderbook.spread_percent().unwrap_or(100.0);
        let total_depth = orderbook.bid_depth() + orderbook.ask_depth();

        // Spread component (lower spread = higher score)
        let spread_score = if spread_percent <= 0.0 {
            1.0
        } else if spread_percent >= self.max_spread_percent {
            0.0
        } else {
            1.0 - (spread_percent / self.max_spread_percent)
        };

        // Depth component (higher depth = higher score)
        // Normalize based on reference order size
        let depth_score = (total_depth / (self.reference_order_size * 10.0)).min(1.0);

        // Combine scores (weighted average)
        (spread_score * 0.6 + depth_score * 0.4).clamp(0.0, 1.0)
    }

    /// Calculate suggested buy and sell prices based on orderbook analysis
    fn calculate_suggested_prices(
        &self,
        orderbook: &OrderBook,
        imbalance_ratio: f64,
    ) -> (f64, f64) {
        let best_bid = orderbook.best_bid().unwrap_or(0.0);
        let best_ask = orderbook.best_ask().unwrap_or(0.0);
        let spread = orderbook.spread().unwrap_or(0.0);

        if spread <= 0.0 {
            return (best_ask, best_bid);
        }

        // Adjust aggression based on imbalance
        // Positive imbalance (buy pressure) -> be more aggressive on sells
        // Negative imbalance (sell pressure) -> be more aggressive on buys
        let imbalance_adjustment = imbalance_ratio * 0.3; // Max 30% adjustment

        // For buying: passive (at bid) to aggressive (at ask)
        // Default aggression: 0.3 (slightly passive)
        let buy_aggression = (0.3 + imbalance_adjustment).clamp(0.0, 1.0);
        let suggested_buy_price = best_bid + (spread * buy_aggression);

        // For selling: passive (at ask) to aggressive (at bid)
        let sell_aggression = (0.3 - imbalance_adjustment).clamp(0.0, 1.0);
        let suggested_sell_price = best_ask - (spread * sell_aggression);

        (suggested_buy_price, suggested_sell_price)
    }

    /// Calculate fill details for a given order size
    /// Returns (total_cost, total_quantity, remaining_size)
    fn calculate_fill_details(
        &self,
        orders: &[OrderBookLevel],
        order_size: f64,
    ) -> (f64, f64, f64) {
        let mut remaining_size = order_size;
        let mut total_cost = 0.0;
        let mut total_quantity = 0.0;

        for level in orders {
            if remaining_size <= 0.0 || level.price <= 0.0 || level.quantity <= 0.0 {
                break;
            }

            let level_value = level.value();
            let fill_value = remaining_size.min(level_value);
            let fill_quantity = fill_value / level.price;

            total_cost += fill_value;
            total_quantity += fill_quantity;
            remaining_size -= fill_value;
        }

        (total_cost, total_quantity, remaining_size)
    }

    /// Estimate slippage for a given order size
    /// Returns slippage as percentage
    pub fn estimate_slippage(&self, orderbook: &OrderBook, order_size: f64, is_buy: bool) -> f64 {
        let orders = if is_buy {
            &orderbook.asks
        } else {
            &orderbook.bids
        };

        let reference_price = if is_buy {
            orderbook.best_ask()
        } else {
            orderbook.best_bid()
        };

        let Some(reference_price) = reference_price else {
            return 0.0;
        };

        if reference_price <= 0.0 || orders.is_empty() || order_size <= 0.0 {
            return 0.0;
        }

        let (total_cost, total_quantity, _) = self.calculate_fill_details(orders, order_size);

        if total_quantity <= 0.0 {
            return 0.0;
        }

        let avg_price = total_cost / total_quantity;
        let slippage = if is_buy {
            (avg_price - reference_price) / reference_price * 100.0
        } else {
            (reference_price - avg_price) / reference_price * 100.0
        };

        slippage.max(0.0)
    }

    /// Calculate Volume-Weighted Average Price for a given order
    pub fn calculate_vwap(&self, orderbook: &OrderBook, order_size: f64, is_buy: bool) -> f64 {
        let orders = if is_buy {
            &orderbook.asks
        } else {
            &orderbook.bids
        };

        if orders.is_empty() || order_size <= 0.0 {
            return if is_buy {
                orderbook.best_ask().unwrap_or(0.0)
            } else {
                orderbook.best_bid().unwrap_or(0.0)
            };
        }

        let (total_cost, total_quantity, _) = self.calculate_fill_details(orders, order_size);

        if total_quantity > 0.0 {
            total_cost / total_quantity
        } else if is_buy {
            orderbook.best_ask().unwrap_or(0.0)
        } else {
            orderbook.best_bid().unwrap_or(0.0)
        }
    }

    /// Get market pressure indicator
    pub fn get_market_pressure(&self, orderbook: &OrderBook) -> MarketPressure {
        let bid_depth = orderbook.bid_depth_within_percent(self.depth_percent);
        let ask_depth = orderbook.ask_depth_within_percent(self.depth_percent);
        let total_depth = bid_depth + ask_depth;

        let imbalance_ratio = if total_depth > 0.0 {
            (bid_depth - ask_depth) / total_depth
        } else {
            0.0
        };

        MarketPressure::from_imbalance(imbalance_ratio)
    }

    /// Get liquidity level classification
    pub fn get_liquidity_level(&self, orderbook: &OrderBook) -> LiquidityLevel {
        let score = self.calculate_liquidity_score(orderbook);
        LiquidityLevel::from_score(score)
    }

    /// Check if orderbook is suitable for trading
    pub fn is_tradeable(&self, orderbook: &OrderBook) -> bool {
        let spread_percent = orderbook.spread_percent().unwrap_or(100.0);
        let liquidity_score = self.calculate_liquidity_score(orderbook);

        spread_percent <= self.max_spread_percent && liquidity_score >= 0.3
    }

    /// Get optimal order price based on aggression level
    /// aggression: 0.0 = most passive, 1.0 = most aggressive
    pub fn get_optimal_price(&self, orderbook: &OrderBook, aggression: f64, is_buy: bool) -> f64 {
        let best_bid = orderbook.best_bid().unwrap_or(0.0);
        let best_ask = orderbook.best_ask().unwrap_or(0.0);
        let spread = orderbook.spread().unwrap_or(0.0);
        let aggression = aggression.clamp(0.0, 1.0);

        if is_buy {
            // Buy: passive = at bid, aggressive = at ask
            best_bid + (spread * aggression)
        } else {
            // Sell: passive = at ask, aggressive = at bid
            best_ask - (spread * aggression)
        }
    }
}

/// Support/Resistance level detected from orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportResistanceLevel {
    /// Price level
    pub price: f64,
    /// Total volume at this level
    pub volume: f64,
    /// Whether this is a support (bid) or resistance (ask)
    pub is_support: bool,
    /// Strength of the level (0.0 to 1.0)
    pub strength: f64,
}

/// Find significant support/resistance levels from orderbook
pub fn find_significant_levels(
    orderbook: &OrderBook,
    min_volume_percentile: f64,
) -> Vec<SupportResistanceLevel> {
    let mut levels = Vec::new();

    // Calculate volume threshold
    let all_volumes: Vec<f64> = orderbook
        .bids
        .iter()
        .chain(orderbook.asks.iter())
        .filter_map(|l| {
            if l.quantity > 0.0 {
                Some(l.quantity)
            } else {
                None
            }
        })
        .collect();

    if all_volumes.is_empty() {
        return levels;
    }

    let mut sorted_volumes = all_volumes;
    sorted_volumes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let percentile_idx =
        ((sorted_volumes.len() as f64) * (min_volume_percentile / 100.0).clamp(0.0, 1.0)) as usize;
    let threshold = sorted_volumes
        .get(percentile_idx.min(sorted_volumes.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);
    let max_volume = sorted_volumes.last().copied().unwrap_or(1.0);

    // Find support levels (large bid orders)
    for level in &orderbook.bids {
        if level.quantity >= threshold {
            let strength = (level.quantity / max_volume).min(1.0);
            levels.push(SupportResistanceLevel {
                price: level.price,
                volume: level.quantity,
                is_support: true,
                strength,
            });
        }
    }

    // Find resistance levels (large ask orders)
    for level in &orderbook.asks {
        if level.quantity >= threshold {
            let strength = (level.quantity / max_volume).min(1.0);
            levels.push(SupportResistanceLevel {
                price: level.price,
                volume: level.quantity,
                is_support: false,
                strength,
            });
        }
    }

    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_orderbook() -> OrderBook {
        OrderBook::from_tuples(
            "TEST".to_string(),
            vec![
                (100.0, 10.0), // Best bid
                (99.0, 20.0),
                (98.0, 30.0),
                (97.0, 15.0),
                (96.0, 25.0),
            ],
            vec![
                (101.0, 8.0), // Best ask
                (102.0, 18.0),
                (103.0, 28.0),
                (104.0, 12.0),
                (105.0, 22.0),
            ],
            0,
        )
    }

    #[test]
    fn test_orderbook_basics() {
        let ob = create_test_orderbook();

        assert_eq!(ob.best_bid(), Some(100.0));
        assert_eq!(ob.best_ask(), Some(101.0));
        assert_eq!(ob.spread(), Some(1.0));
        assert_eq!(ob.mid_price(), Some(100.5));
    }

    #[test]
    fn test_orderbook_depth() {
        let ob = create_test_orderbook();

        let bid_depth = ob.bid_depth();
        let ask_depth = ob.ask_depth();

        assert_eq!(bid_depth, 100.0); // 10 + 20 + 30 + 15 + 25
        assert_eq!(ask_depth, 88.0); // 8 + 18 + 28 + 12 + 22
    }

    #[test]
    fn test_orderbook_analysis() {
        let ob = create_test_orderbook();
        let analyzer = OrderBookAnalyzer::default();

        let analysis = analyzer.analyze(&ob);

        assert_eq!(analysis.best_bid, 100.0);
        assert_eq!(analysis.best_ask, 101.0);
        assert!(analysis.imbalance_ratio > 0.0); // More bids than asks
        assert!(analysis.liquidity_score > 0.0);
    }

    #[test]
    fn test_market_pressure() {
        // Test bullish
        assert!(MarketPressure::from_imbalance(0.5).is_bullish());
        assert!(MarketPressure::from_imbalance(0.15).is_bullish());

        // Test bearish
        assert!(MarketPressure::from_imbalance(-0.5).is_bearish());
        assert!(MarketPressure::from_imbalance(-0.15).is_bearish());

        // Test neutral
        assert_eq!(MarketPressure::from_imbalance(0.0), MarketPressure::Neutral);
        assert_eq!(
            MarketPressure::from_imbalance(0.05),
            MarketPressure::Neutral
        );
    }

    #[test]
    fn test_slippage_estimation() {
        let ob = create_test_orderbook();
        let analyzer = OrderBookAnalyzer::new(1.0, 1000.0, 1.0);

        // Small order should have minimal slippage
        let small_slippage = analyzer.estimate_slippage(&ob, 500.0, true);
        assert!(small_slippage < 1.0);

        // Large order should have more slippage
        let large_slippage = analyzer.estimate_slippage(&ob, 5000.0, true);
        assert!(large_slippage >= small_slippage);
    }

    #[test]
    fn test_optimal_price() {
        let ob = create_test_orderbook();
        let analyzer = OrderBookAnalyzer::default();

        // Passive buy = at bid
        let passive_buy = analyzer.get_optimal_price(&ob, 0.0, true);
        assert_eq!(passive_buy, 100.0);

        // Aggressive buy = at ask
        let aggressive_buy = analyzer.get_optimal_price(&ob, 1.0, true);
        assert_eq!(aggressive_buy, 101.0);

        // Passive sell = at ask
        let passive_sell = analyzer.get_optimal_price(&ob, 0.0, false);
        assert_eq!(passive_sell, 101.0);

        // Aggressive sell = at bid
        let aggressive_sell = analyzer.get_optimal_price(&ob, 1.0, false);
        assert_eq!(aggressive_sell, 100.0);
    }

    #[test]
    fn test_empty_orderbook() {
        let ob = OrderBook::new("TEST".to_string(), Vec::new(), Vec::new(), 0);
        let analyzer = OrderBookAnalyzer::default();

        assert_eq!(ob.best_bid(), None);
        assert_eq!(ob.best_ask(), None);
        assert_eq!(ob.bid_depth(), 0.0);
        assert_eq!(ob.ask_depth(), 0.0);

        let analysis = analyzer.analyze(&ob);
        assert_eq!(analysis.best_bid, 0.0);
        assert_eq!(analysis.best_ask, 0.0);
    }

    #[test]
    fn test_invalid_orderbook() {
        let ob = OrderBook::from_tuples(
            "TEST".to_string(),
            vec![(100.0, 10.0), (101.0, 20.0)], // Invalid: bid > ask
            vec![(99.0, 10.0)],                 // Invalid: ask < bid
            0,
        );
        let analyzer = OrderBookAnalyzer::default();
        let analysis = analyzer.analyze(&ob);

        // Should handle gracefully - returns default when invalid (best_ask <= best_bid)
        assert_eq!(analysis.best_bid, 0.0);
        assert_eq!(analysis.best_ask, 0.0);
    }

    #[test]
    fn test_calculate_vwap() {
        let ob = create_test_orderbook();
        let analyzer = OrderBookAnalyzer::default();

        let vwap = analyzer.calculate_vwap(&ob, 1000.0, true);
        assert!(vwap > 0.0);
        assert!(vwap >= ob.best_ask().unwrap());

        let vwap_sell = analyzer.calculate_vwap(&ob, 1000.0, false);
        assert!(vwap_sell > 0.0);
        assert!(vwap_sell <= ob.best_bid().unwrap());
    }

    #[test]
    fn test_find_significant_levels() {
        let ob = create_test_orderbook();
        let levels = find_significant_levels(&ob, 50.0);

        assert!(!levels.is_empty());
        assert!(levels.iter().any(|l| l.is_support));
        assert!(levels.iter().any(|l| !l.is_support));
    }
}
