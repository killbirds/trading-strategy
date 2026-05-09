use crate::model::PositionType;
use crate::strategy::parabolic_sar_short_strategy::ParabolicSARShortStrategy;
use crate::strategy::parabolic_sar_strategy::ParabolicSARStrategy;
use crate::strategy::tests::common::create_test_storage;
use crate::strategy::{Strategy, StrategyFactory, StrategyType};
use crate::tests::TestCandle;
use std::collections::HashMap;

fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
    TestCandle {
        timestamp,
        open: close,
        high,
        low,
        close,
        volume: 1000.0,
    }
}

fn bullish_sar_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 10.0, 8.0, 10.0),
        candle(2, 11.0, 9.0, 11.0),
        candle(3, 12.0, 10.0, 12.0),
    ]
}

fn bearish_sar_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 10.0, 8.0, 10.0),
        candle(2, 11.0, 9.0, 11.0),
        candle(3, 7.0, 6.0, 7.0),
    ]
}

fn create_parabolic_sar_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("count".to_string(), "1".to_string());
    config.insert("step".to_string(), "0.02".to_string());
    config.insert("max_step".to_string(), "0.2".to_string());
    config
}

#[test]
fn test_parabolic_sar_strategy_creation() {
    let storage = create_test_storage(bullish_sar_candles());
    let strategy =
        ParabolicSARStrategy::new_with_config(&storage, Some(create_parabolic_sar_config()))
            .unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Long);
    assert_eq!(strategy.name(), StrategyType::ParabolicSAR);
}

#[test]
fn test_parabolic_sar_short_strategy_creation() {
    let storage = create_test_storage(bearish_sar_candles());
    let strategy =
        ParabolicSARShortStrategy::new_with_config(&storage, Some(create_parabolic_sar_config()))
            .unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Short);
    assert_eq!(strategy.name(), StrategyType::ParabolicSARShort);
}

#[test]
fn test_parabolic_sar_strategy_uses_current_price_for_long_signal() {
    let storage = create_test_storage(bullish_sar_candles());
    let strategy =
        ParabolicSARStrategy::new_with_config(&storage, Some(create_parabolic_sar_config()))
            .unwrap();

    assert!(strategy.should_enter(12.0));
    assert!(!strategy.should_enter(7.0));
    assert!(strategy.should_exit(7.0));
    assert!(!strategy.should_exit(12.0));
}

#[test]
fn test_parabolic_sar_short_strategy_uses_current_price_for_short_signal() {
    let storage = create_test_storage(bearish_sar_candles());
    let strategy =
        ParabolicSARShortStrategy::new_with_config(&storage, Some(create_parabolic_sar_config()))
            .unwrap();

    assert!(strategy.should_enter(6.0));
    assert!(!strategy.should_enter(12.0));
    assert!(strategy.should_exit(12.0));
    assert!(!strategy.should_exit(6.0));
}

#[test]
fn test_parabolic_sar_strategy_factory_wiring() {
    let bullish_storage = create_test_storage(bullish_sar_candles());
    let bearish_storage = create_test_storage(bearish_sar_candles());

    let long_strategy = StrategyFactory::build(
        StrategyType::ParabolicSAR,
        &bullish_storage,
        Some(create_parabolic_sar_config()),
    )
    .unwrap();
    assert_eq!(long_strategy.position(), PositionType::Long);
    assert_eq!(long_strategy.name(), StrategyType::ParabolicSAR);

    let short_strategy = StrategyFactory::build(
        StrategyType::ParabolicSARShort,
        &bearish_storage,
        Some(create_parabolic_sar_config()),
    )
    .unwrap();
    assert_eq!(short_strategy.position(), PositionType::Short);
    assert_eq!(short_strategy.name(), StrategyType::ParabolicSARShort);

    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::ParabolicSAR),
        PositionType::Long
    );
    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::ParabolicSARShort),
        PositionType::Short
    );
    assert_eq!(StrategyType::ParabolicSAR.to_string(), "parabolic_sar");
    assert_eq!(
        StrategyType::ParabolicSARShort.to_string(),
        "parabolic_sar_short"
    );
}

#[test]
fn test_parabolic_sar_strategy_rejects_invalid_config() {
    let storage = create_test_storage(bullish_sar_candles());
    let mut invalid_config = create_parabolic_sar_config();
    invalid_config.insert("step".to_string(), "0.3".to_string());
    invalid_config.insert("max_step".to_string(), "0.2".to_string());

    assert!(ParabolicSARStrategy::new_with_config(&storage, Some(invalid_config)).is_err());

    let mut huge_count_config = create_parabolic_sar_config();
    huge_count_config.insert("count".to_string(), usize::MAX.to_string());
    assert!(ParabolicSARStrategy::new_with_config(&storage, Some(huge_count_config)).is_err());
}
