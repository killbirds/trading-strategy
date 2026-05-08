use crate::model::PositionType;
use crate::strategy::box_range_short_strategy::BoxRangeShortStrategy;
use crate::strategy::box_range_strategy::BoxRangeStrategy;
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

fn box_range_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 101.0, 99.0, 100.0),
        candle(2, 102.0, 98.5, 101.0),
        candle(3, 101.5, 99.5, 100.5),
        candle(4, 102.0, 98.0, 99.5),
        candle(5, 101.0, 99.0, 100.0),
        candle(6, 101.0, 99.0, 100.0),
    ]
}

fn create_box_range_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("count".to_string(), "1".to_string());
    config.insert("period".to_string(), "5".to_string());
    config.insert("max_width_ratio".to_string(), "0.05".to_string());
    config
}

#[test]
fn test_box_range_strategy_creation() {
    let storage = create_test_storage(box_range_candles());
    let strategy =
        BoxRangeStrategy::new_with_config(&storage, Some(create_box_range_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Long);
    assert_eq!(strategy.name(), StrategyType::BoxRange);
}

#[test]
fn test_box_range_short_strategy_creation() {
    let storage = create_test_storage(box_range_candles());
    let strategy =
        BoxRangeShortStrategy::new_with_config(&storage, Some(create_box_range_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Short);
    assert_eq!(strategy.name(), StrategyType::BoxRangeShort);
}

#[test]
fn test_box_range_strategy_uses_current_price_for_long_breakout() {
    let storage = create_test_storage(box_range_candles());
    let strategy =
        BoxRangeStrategy::new_with_config(&storage, Some(create_box_range_config())).unwrap();

    assert!(strategy.should_enter(103.0));
    assert!(!strategy.should_enter(101.0));
    assert!(strategy.should_exit(99.0));
    assert!(!strategy.should_exit(101.0));
}

#[test]
fn test_box_range_short_strategy_uses_current_price_for_downside_breakout() {
    let storage = create_test_storage(box_range_candles());
    let strategy =
        BoxRangeShortStrategy::new_with_config(&storage, Some(create_box_range_config())).unwrap();

    assert!(strategy.should_enter(97.0));
    assert!(!strategy.should_enter(100.0));
    assert!(strategy.should_exit(101.0));
    assert!(!strategy.should_exit(99.0));
}

#[test]
fn test_box_range_strategy_factory_wiring() {
    let storage = create_test_storage(box_range_candles());

    let long_strategy = StrategyFactory::build(
        StrategyType::BoxRange,
        &storage,
        Some(create_box_range_config()),
    )
    .unwrap();
    assert_eq!(long_strategy.position(), PositionType::Long);
    assert_eq!(long_strategy.name(), StrategyType::BoxRange);

    let short_strategy = StrategyFactory::build(
        StrategyType::BoxRangeShort,
        &storage,
        Some(create_box_range_config()),
    )
    .unwrap();
    assert_eq!(short_strategy.position(), PositionType::Short);
    assert_eq!(short_strategy.name(), StrategyType::BoxRangeShort);

    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::BoxRange),
        PositionType::Long
    );
    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::BoxRangeShort),
        PositionType::Short
    );
    assert_eq!(StrategyType::BoxRange.to_string(), "box_range");
    assert_eq!(StrategyType::BoxRangeShort.to_string(), "box_range_short");
}

#[test]
fn test_box_range_strategy_rejects_invalid_config() {
    let storage = create_test_storage(box_range_candles());
    let mut invalid_config = create_box_range_config();
    invalid_config.insert("max_width_ratio".to_string(), "0".to_string());

    assert!(BoxRangeStrategy::new_with_config(&storage, Some(invalid_config)).is_err());
}
