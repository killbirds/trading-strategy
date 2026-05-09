use crate::model::PositionType;
use crate::strategy::keltner_short_strategy::KeltnerShortStrategy;
use crate::strategy::keltner_strategy::KeltnerStrategy;
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

fn keltner_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 11.0, 9.0, 10.0),
        candle(2, 12.0, 10.0, 11.0),
        candle(3, 13.0, 11.0, 12.0),
        candle(4, 14.0, 12.0, 13.0),
    ]
}

fn create_keltner_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("count".to_string(), "1".to_string());
    config.insert("period".to_string(), "3".to_string());
    config.insert("multiplier".to_string(), "2.0".to_string());
    config
}

#[test]
fn test_keltner_strategy_creation() {
    let storage = create_test_storage(keltner_candles());
    let strategy =
        KeltnerStrategy::new_with_config(&storage, Some(create_keltner_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Long);
    assert_eq!(strategy.name(), StrategyType::Keltner);
}

#[test]
fn test_keltner_short_strategy_creation() {
    let storage = create_test_storage(keltner_candles());
    let strategy =
        KeltnerShortStrategy::new_with_config(&storage, Some(create_keltner_config())).unwrap();

    assert!(!strategy.to_string().is_empty());
    assert_eq!(strategy.position(), PositionType::Short);
    assert_eq!(strategy.name(), StrategyType::KeltnerShort);
}

#[test]
fn test_keltner_strategy_uses_current_price_for_long_breakout() {
    let storage = create_test_storage(keltner_candles());
    let strategy =
        KeltnerStrategy::new_with_config(&storage, Some(create_keltner_config())).unwrap();

    assert!(strategy.should_enter(16.0));
    assert!(!strategy.should_enter(15.0));
    assert!(strategy.should_exit(10.0));
    assert!(!strategy.should_exit(11.0));
}

#[test]
fn test_keltner_short_strategy_uses_current_price_for_downside_breakout() {
    let storage = create_test_storage(keltner_candles());
    let strategy =
        KeltnerShortStrategy::new_with_config(&storage, Some(create_keltner_config())).unwrap();

    assert!(strategy.should_enter(6.0));
    assert!(!strategy.should_enter(7.0));
    assert!(strategy.should_exit(12.0));
    assert!(!strategy.should_exit(11.0));
}

#[test]
fn test_keltner_strategy_factory_wiring() {
    let storage = create_test_storage(keltner_candles());

    let long_strategy = StrategyFactory::build(
        StrategyType::Keltner,
        &storage,
        Some(create_keltner_config()),
    )
    .unwrap();
    assert_eq!(long_strategy.position(), PositionType::Long);
    assert_eq!(long_strategy.name(), StrategyType::Keltner);

    let short_strategy = StrategyFactory::build(
        StrategyType::KeltnerShort,
        &storage,
        Some(create_keltner_config()),
    )
    .unwrap();
    assert_eq!(short_strategy.position(), PositionType::Short);
    assert_eq!(short_strategy.name(), StrategyType::KeltnerShort);

    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::Keltner),
        PositionType::Long
    );
    assert_eq!(
        StrategyFactory::position_from_strategy_type(StrategyType::KeltnerShort),
        PositionType::Short
    );
    assert_eq!(StrategyType::Keltner.to_string(), "keltner");
    assert_eq!(StrategyType::KeltnerShort.to_string(), "keltner_short");
}

#[test]
fn test_keltner_strategy_rejects_invalid_config() {
    let storage = create_test_storage(keltner_candles());
    let mut invalid_config = create_keltner_config();
    invalid_config.insert("multiplier".to_string(), "0".to_string());

    assert!(KeltnerStrategy::new_with_config(&storage, Some(invalid_config)).is_err());

    let mut huge_period_config = create_keltner_config();
    huge_period_config.insert("period".to_string(), usize::MAX.to_string());
    assert!(KeltnerStrategy::new_with_config(&storage, Some(huge_period_config)).is_err());

    let mut huge_count_config = create_keltner_config();
    huge_count_config.insert("count".to_string(), usize::MAX.to_string());
    assert!(KeltnerStrategy::new_with_config(&storage, Some(huge_count_config)).is_err());
}
