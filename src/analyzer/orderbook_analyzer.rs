// Orderbook Analyzer
// Provides time-series analysis of orderbook data for trading decisions

use crate::indicator::orderbook::{
    LiquidityLevel, MarketPressure, OrderBook, OrderBookAnalysis, OrderBookAnalyzer,
    SupportResistanceLevel, find_significant_levels,
};
use serde::{Deserialize, Serialize};

/// Configuration for orderbook analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookAnalyzerConfig {
    /// Depth percentage for liquidity calculation
    pub depth_percent: f64,
    /// Reference order size for slippage estimation
    pub reference_order_size: f64,
    /// Maximum acceptable spread percentage
    pub max_spread_percent: f64,
    /// Number of historical snapshots to keep
    pub history_size: usize,
    /// Minimum volume percentile for significant levels
    pub min_volume_percentile: f64,
    /// Aggression level for suggested prices (0.0 - 1.0)
    pub default_aggression: f64,
}

impl Default for OrderBookAnalyzerConfig {
    fn default() -> Self {
        Self {
            depth_percent: 1.0,
            reference_order_size: 1_000_000.0,
            max_spread_percent: 1.0,
            history_size: 100,
            min_volume_percentile: 75.0,
            default_aggression: 0.3,
        }
    }
}

/// Historical orderbook data point
#[derive(Debug, Clone)]
pub struct OrderBookDataPoint {
    /// Timestamp
    pub timestamp: i64,
    /// Analysis result
    pub analysis: OrderBookAnalysis,
    /// Market pressure
    pub pressure: MarketPressure,
    /// Liquidity level
    pub liquidity: LiquidityLevel,
    /// Significant support/resistance levels
    pub significant_levels: Vec<SupportResistanceLevel>,
}

/// Trend direction based on orderbook analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderBookTrend {
    /// Increasing buy pressure
    BuyPressureIncreasing,
    /// Decreasing buy pressure
    BuyPressureDecreasing,
    /// Increasing sell pressure
    SellPressureIncreasing,
    /// Decreasing sell pressure
    SellPressureDecreasing,
    /// No clear trend
    Neutral,
}

/// Trading signal from orderbook analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderBookSignal {
    /// Strong buy signal
    StrongBuy,
    /// Buy signal
    Buy,
    /// Neutral
    Neutral,
    /// Sell signal
    Sell,
    /// Strong sell signal
    StrongSell,
}

impl OrderBookSignal {
    pub fn is_bullish(&self) -> bool {
        matches!(self, OrderBookSignal::StrongBuy | OrderBookSignal::Buy)
    }

    pub fn is_bearish(&self) -> bool {
        matches!(self, OrderBookSignal::StrongSell | OrderBookSignal::Sell)
    }
}

/// Comprehensive orderbook analysis result
#[derive(Debug, Clone)]
pub struct OrderBookAnalysisResult {
    /// Current analysis
    pub current: OrderBookAnalysis,
    /// Current market pressure
    pub pressure: MarketPressure,
    /// Current liquidity level
    pub liquidity: LiquidityLevel,
    /// Trend based on historical data
    pub trend: OrderBookTrend,
    /// Trading signal
    pub signal: OrderBookSignal,
    /// Signal strength (0.0 - 1.0)
    pub signal_strength: f64,
    /// Is the orderbook suitable for trading
    pub is_tradeable: bool,
    /// Significant support/resistance levels
    pub significant_levels: Vec<SupportResistanceLevel>,
    /// Suggested buy price
    pub suggested_buy_price: f64,
    /// Suggested sell price
    pub suggested_sell_price: f64,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Orderbook time-series analyzer
#[derive(Debug)]
pub struct OrderBookTimeSeriesAnalyzer {
    /// Configuration
    config: OrderBookAnalyzerConfig,
    /// Internal analyzer
    analyzer: OrderBookAnalyzer,
    /// Historical data points
    history: Vec<OrderBookDataPoint>,
}

impl OrderBookTimeSeriesAnalyzer {
    /// Create new analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(OrderBookAnalyzerConfig::default())
    }

    /// Create new analyzer with custom configuration
    pub fn with_config(config: OrderBookAnalyzerConfig) -> Self {
        let analyzer = OrderBookAnalyzer::new(
            config.depth_percent,
            config.reference_order_size,
            config.max_spread_percent,
        );

        Self {
            config,
            analyzer,
            history: Vec::new(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &OrderBookAnalyzerConfig {
        &self.config
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Add new orderbook snapshot
    pub fn update(&mut self, orderbook: &OrderBook) -> OrderBookAnalysisResult {
        let analysis = self.analyzer.analyze(orderbook);
        let pressure = self.analyzer.get_market_pressure(orderbook);
        let liquidity = self.analyzer.get_liquidity_level(orderbook);
        let significant_levels =
            find_significant_levels(orderbook, self.config.min_volume_percentile);
        let is_tradeable = self.analyzer.is_tradeable(orderbook);

        // Create data point
        let data_point = OrderBookDataPoint {
            timestamp: orderbook.timestamp,
            analysis: analysis.clone(),
            pressure,
            liquidity,
            significant_levels: significant_levels.clone(),
        };

        // Add to history
        self.history.insert(0, data_point);
        if self.history.len() > self.config.history_size {
            self.history.pop();
        }

        // Calculate trend and signal
        let trend = self.calculate_trend();
        let (signal, signal_strength) = self.calculate_signal(&analysis, pressure, &trend);
        let confidence = self.calculate_confidence(&analysis, liquidity, &trend);

        // Calculate suggested prices
        let suggested_buy_price =
            self.analyzer
                .get_optimal_price(orderbook, self.config.default_aggression, true);
        let suggested_sell_price =
            self.analyzer
                .get_optimal_price(orderbook, self.config.default_aggression, false);

        OrderBookAnalysisResult {
            current: analysis,
            pressure,
            liquidity,
            trend,
            signal,
            signal_strength,
            is_tradeable,
            significant_levels,
            suggested_buy_price,
            suggested_sell_price,
            confidence,
        }
    }

    /// Calculate trend from historical data
    fn calculate_trend(&self) -> OrderBookTrend {
        if self.history.len() < 3 {
            return OrderBookTrend::Neutral;
        }

        // Calculate average imbalance for recent vs older data
        let recent_count = (self.history.len() / 3).max(1);
        let recent_avg: f64 = self
            .history
            .iter()
            .take(recent_count)
            .map(|dp| dp.analysis.imbalance_ratio)
            .sum::<f64>()
            / recent_count as f64;

        let older_avg: f64 = self
            .history
            .iter()
            .skip(recent_count)
            .take(recent_count)
            .map(|dp| dp.analysis.imbalance_ratio)
            .sum::<f64>()
            / recent_count as f64;

        let change = recent_avg - older_avg;
        let threshold = 0.05;

        if recent_avg > 0.0 && change > threshold {
            OrderBookTrend::BuyPressureIncreasing
        } else if recent_avg > 0.0 && change < -threshold {
            OrderBookTrend::BuyPressureDecreasing
        } else if recent_avg < 0.0 && change < -threshold {
            OrderBookTrend::SellPressureIncreasing
        } else if recent_avg < 0.0 && change > threshold {
            OrderBookTrend::SellPressureDecreasing
        } else {
            OrderBookTrend::Neutral
        }
    }

    /// Calculate trading signal and strength
    fn calculate_signal(
        &self,
        analysis: &OrderBookAnalysis,
        pressure: MarketPressure,
        trend: &OrderBookTrend,
    ) -> (OrderBookSignal, f64) {
        let mut score = 0.0;

        // Factor 1: Current imbalance
        score += analysis.imbalance_ratio * 0.4;

        // Factor 2: Market pressure
        score += match pressure {
            MarketPressure::StrongBuy => 0.3,
            MarketPressure::ModerateBuy => 0.15,
            MarketPressure::Neutral => 0.0,
            MarketPressure::ModerateSell => -0.15,
            MarketPressure::StrongSell => -0.3,
        };

        // Factor 3: Trend
        score += match trend {
            OrderBookTrend::BuyPressureIncreasing => 0.2,
            OrderBookTrend::BuyPressureDecreasing => -0.1,
            OrderBookTrend::SellPressureIncreasing => -0.2,
            OrderBookTrend::SellPressureDecreasing => 0.1,
            OrderBookTrend::Neutral => 0.0,
        };

        // Determine signal
        let signal = if score > 0.4 {
            OrderBookSignal::StrongBuy
        } else if score > 0.15 {
            OrderBookSignal::Buy
        } else if score < -0.4 {
            OrderBookSignal::StrongSell
        } else if score < -0.15 {
            OrderBookSignal::Sell
        } else {
            OrderBookSignal::Neutral
        };

        let strength = score.abs().min(1.0);

        (signal, strength)
    }

    /// Calculate confidence in the analysis
    fn calculate_confidence(
        &self,
        analysis: &OrderBookAnalysis,
        liquidity: LiquidityLevel,
        trend: &OrderBookTrend,
    ) -> f64 {
        let mut confidence = 0.0;

        // Factor 1: Liquidity (higher = more confident)
        confidence += match liquidity {
            LiquidityLevel::VeryHigh => 0.35,
            LiquidityLevel::High => 0.28,
            LiquidityLevel::Medium => 0.2,
            LiquidityLevel::Low => 0.1,
            LiquidityLevel::VeryLow => 0.05,
        };

        // Factor 2: Spread (lower = more confident)
        let spread_factor = if analysis.spread_percent <= 0.1 {
            0.25
        } else if analysis.spread_percent <= 0.3 {
            0.2
        } else if analysis.spread_percent <= 0.5 {
            0.15
        } else if analysis.spread_percent <= 1.0 {
            0.1
        } else {
            0.05
        };
        confidence += spread_factor;

        // Factor 3: Clear trend (clearer = more confident)
        confidence += match trend {
            OrderBookTrend::BuyPressureIncreasing | OrderBookTrend::SellPressureIncreasing => 0.25,
            OrderBookTrend::BuyPressureDecreasing | OrderBookTrend::SellPressureDecreasing => 0.15,
            OrderBookTrend::Neutral => 0.1,
        };

        // Factor 4: Data sufficiency
        let data_factor =
            (self.history.len() as f64 / self.config.history_size as f64).min(1.0) * 0.15;
        confidence += data_factor;

        confidence.min(1.0)
    }

    /// Get historical imbalance values
    pub fn get_imbalance_history(&self, count: usize) -> Vec<f64> {
        self.history
            .iter()
            .take(count)
            .map(|dp| dp.analysis.imbalance_ratio)
            .collect()
    }

    /// Get historical spread values
    pub fn get_spread_history(&self, count: usize) -> Vec<f64> {
        self.history
            .iter()
            .take(count)
            .map(|dp| dp.analysis.spread_percent)
            .collect()
    }

    /// Get average imbalance over last N snapshots
    pub fn get_average_imbalance(&self, count: usize) -> f64 {
        let values = self.get_imbalance_history(count);
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Get average spread over last N snapshots
    pub fn get_average_spread(&self, count: usize) -> f64 {
        let values = self.get_spread_history(count);
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Check if there's enough historical data for reliable analysis
    pub fn has_sufficient_data(&self) -> bool {
        self.history.len() >= 10
    }

    /// Get optimal buy price considering aggression
    pub fn get_optimal_buy_price(&self, orderbook: &OrderBook, aggression: f64) -> f64 {
        self.analyzer.get_optimal_price(orderbook, aggression, true)
    }

    /// Get optimal sell price considering aggression  
    pub fn get_optimal_sell_price(&self, orderbook: &OrderBook, aggression: f64) -> f64 {
        self.analyzer
            .get_optimal_price(orderbook, aggression, false)
    }

    /// Estimate slippage for a buy order
    pub fn estimate_buy_slippage(&self, orderbook: &OrderBook, order_size: f64) -> f64 {
        self.analyzer.estimate_slippage(orderbook, order_size, true)
    }

    /// Estimate slippage for a sell order
    pub fn estimate_sell_slippage(&self, orderbook: &OrderBook, order_size: f64) -> f64 {
        self.analyzer
            .estimate_slippage(orderbook, order_size, false)
    }

    /// Get VWAP for a buy order
    pub fn get_buy_vwap(&self, orderbook: &OrderBook, order_size: f64) -> f64 {
        self.analyzer.calculate_vwap(orderbook, order_size, true)
    }

    /// Get VWAP for a sell order
    pub fn get_sell_vwap(&self, orderbook: &OrderBook, order_size: f64) -> f64 {
        self.analyzer.calculate_vwap(orderbook, order_size, false)
    }
}

impl Default for OrderBookTimeSeriesAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator::orderbook::OrderBookLevel;

    fn create_test_orderbook(timestamp: i64, bid_base: f64, imbalance: f64) -> OrderBook {
        // Create orderbook with specified imbalance
        // imbalance > 0 means more buy pressure
        let bid_volume = 100.0 * (1.0 + imbalance);
        let ask_volume = 100.0 * (1.0 - imbalance);

        OrderBook {
            symbol: "TEST".to_string(),
            bids: vec![
                OrderBookLevel::new(bid_base, bid_volume * 0.4),
                OrderBookLevel::new(bid_base - 1.0, bid_volume * 0.3),
                OrderBookLevel::new(bid_base - 2.0, bid_volume * 0.3),
            ],
            asks: vec![
                OrderBookLevel::new(bid_base + 1.0, ask_volume * 0.4),
                OrderBookLevel::new(bid_base + 2.0, ask_volume * 0.3),
                OrderBookLevel::new(bid_base + 3.0, ask_volume * 0.3),
            ],
            timestamp,
        }
    }

    #[test]
    fn test_analyzer_initialization() {
        let analyzer = OrderBookTimeSeriesAnalyzer::new();
        assert_eq!(analyzer.history_len(), 0);
        assert!(!analyzer.has_sufficient_data());
    }

    #[test]
    fn test_single_update() {
        let mut analyzer = OrderBookTimeSeriesAnalyzer::new();
        let orderbook = create_test_orderbook(1000, 100.0, 0.2);

        let result = analyzer.update(&orderbook);

        assert_eq!(analyzer.history_len(), 1);
        assert!(result.current.imbalance_ratio > 0.0);
        assert!(result.pressure.is_bullish());
    }

    #[test]
    fn test_trend_detection() {
        let mut analyzer = OrderBookTimeSeriesAnalyzer::new();

        // Add increasing buy pressure over time
        for i in 0..20 {
            let imbalance = (i as f64) * 0.02; // Increasing imbalance
            let orderbook = create_test_orderbook(1000 + i, 100.0, imbalance);
            analyzer.update(&orderbook);
        }

        let orderbook = create_test_orderbook(1020, 100.0, 0.4);
        let result = analyzer.update(&orderbook);

        assert_eq!(result.trend, OrderBookTrend::BuyPressureIncreasing);
    }

    #[test]
    fn test_signal_generation() {
        let mut analyzer = OrderBookTimeSeriesAnalyzer::new();

        // Build up strong buy pressure
        for i in 0..15 {
            let orderbook = create_test_orderbook(1000 + i, 100.0, 0.4);
            analyzer.update(&orderbook);
        }

        let orderbook = create_test_orderbook(1015, 100.0, 0.5);
        let result = analyzer.update(&orderbook);

        assert!(result.signal.is_bullish());
        assert!(result.signal_strength > 0.0);
    }

    #[test]
    fn test_history_limit() {
        let config = OrderBookAnalyzerConfig {
            history_size: 10,
            ..Default::default()
        };
        let mut analyzer = OrderBookTimeSeriesAnalyzer::with_config(config);

        for i in 0..20 {
            let orderbook = create_test_orderbook(1000 + i, 100.0, 0.1);
            analyzer.update(&orderbook);
        }

        assert_eq!(analyzer.history_len(), 10);
    }

    #[test]
    fn test_average_calculations() {
        let mut analyzer = OrderBookTimeSeriesAnalyzer::new();

        for i in 0..10 {
            let orderbook = create_test_orderbook(1000 + i, 100.0, 0.2);
            analyzer.update(&orderbook);
        }

        let avg_imbalance = analyzer.get_average_imbalance(5);
        assert!(avg_imbalance > 0.0);

        let avg_spread = analyzer.get_average_spread(5);
        assert!(avg_spread > 0.0);
    }
}
