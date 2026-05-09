use crate::model::PositionType;
use crate::strategy::donchian_short_strategy::DonchianShortStrategy;
use crate::strategy::donchian_strategy::DonchianStrategy;
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

fn donchian_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 10.0, 8.0, 9.0),
        candle(2, 11.0, 7.0, 10.0),
        candle(3, 12.0, 6.0, 11.0),
        candle(4, 11.0, 7.0, 10.0),
    ]
}

fn create_donchian_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("count".to_string(), "1".to_string());
    config.insert("period".to_string(), "3".to_string());
    config
}

#[test]
fn test_donchian_strategy_creation() {
    let storage = create_test_storage(donchian_candles());
    let strategy =
        DonchianStrategy::new_with_config(&storage, Some(create_donchian_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Long);
    assert_eq!(strategy.name(), StrategyType::Donchian);
}

#[test]
fn test_donchian_short_strategy_creation() {
    let storage = create_test_storage(donchian_candles());
    let strategy =
        DonchianShortStrategy::new_with_config(&storage, Some(create_donchian_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Short);
    assert_eq!(strategy.name(), StrategyType::DonchianShort);
}

#[test]
fn test_donchian_strategy_uses_current_price_for_long_breakout() {
    let storage = create_test_storage(donchian_candles());
    let strategy =
        DonchianStrategy::new_with_config(&storage, Some(create_donchian_config())).unwrap();

    assert!(strategy.should_enter(13.0));
    assert!(!strategy.should_enter(12.0));
    assert!(strategy.should_exit(8.0));
    assert!(!strategy.should_exit(10.0));
}

#[test]
fn test_donchian_short_strategy_uses_current_price_for_downside_breakout() {
    let storage = create_test_storage(donchian_candles());
    let strategy =
        DonchianShortStrategy::new_with_config(&storage, Some(create_donchian_config())).unwrap();

    assert!(strategy.should_enter(5.0));
    assert!(!strategy.should_enter(6.0));
    assert!(strategy.should_exit(10.0));
    assert!(!strategy.should_exit(8.0));
}

#[test]
fn test_donchian_strategy_factory_wiring() {
    let storage = create_test_storage(donchian_candles());

    let long_strategy = StrategyFactory::build(
        StrategyType::Donchian,
        &storage,
        Some(create_donchian_config()),
    )
    .unwrap();
    assert_eq!(long_strategy.position(), PositionType::Long);
    assert_eq!(long_strategy.name(), StrategyType::Donchian);

    let short_strategy = StrategyFactory::build(
        StrategyType::DonchianShort,
        &storage,
        Some(create_donchian_config()),
    )
    .unwrap();
    assert_eq!(short_strategy.position(), PositionType::Short);
    assert_eq!(short_strategy.name(), StrategyType::DonchianShort);

    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::Donchian),
        PositionType::Long
    );
    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::DonchianShort),
        PositionType::Short
    );
    assert_eq!(StrategyType::Donchian.to_string(), "donchian");
    assert_eq!(StrategyType::DonchianShort.to_string(), "donchian_short");
}

#[test]
fn test_donchian_strategy_rejects_invalid_config() {
    let storage = create_test_storage(donchian_candles());
    let mut invalid_config = create_donchian_config();
    invalid_config.insert("period".to_string(), "0".to_string());

    assert!(DonchianStrategy::new_with_config(&storage, Some(invalid_config)).is_err());

    let mut huge_period_config = create_donchian_config();
    huge_period_config.insert("period".to_string(), usize::MAX.to_string());
    assert!(DonchianStrategy::new_with_config(&storage, Some(huge_period_config)).is_err());

    let mut huge_count_config = create_donchian_config();
    huge_count_config.insert("count".to_string(), usize::MAX.to_string());
    assert!(DonchianStrategy::new_with_config(&storage, Some(huge_count_config)).is_err());
}
