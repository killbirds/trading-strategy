mod common_test_utils;
use common_test_utils::*;

use trading_chart::Candle;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_new_empty_store() {
    let store = CandleStore::<TestCandle>::new(Vec::new(), 100, false);
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert_eq!(store.max_size, 100);
}

#[test]
fn test_new_with_items_sorted_descending() {
    let candles = vec![
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert_eq!(store.len(), 3);
    assert_eq!(store.first().unwrap().datetime().timestamp(), 3);
    assert_eq!(store.get(1).unwrap().datetime().timestamp(), 2);
    assert_eq!(store.get(2).unwrap().datetime().timestamp(), 1);
}

#[test]
fn test_new_max_size_limit() {
    let candles: Vec<TestCandle> = (0..10)
        .map(|i| TestCandle::new(i, 100.0, 105.0, 95.0, 102.0, 1000.0))
        .collect();

    let store = CandleStore::new(candles, 5, false);

    assert_eq!(store.len(), 5);
    assert_eq!(store.first().unwrap().datetime().timestamp(), 9);
}

#[test]
fn test_add_maintains_descending_order() {
    let mut store = CandleStore::<TestCandle>::new(Vec::new(), 100, false);

    store.add(TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0));
    store.add(TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0));

    assert_eq!(store.len(), 3);
    assert_eq!(store.get(0).unwrap().datetime().timestamp(), 3);
    assert_eq!(store.get(1).unwrap().datetime().timestamp(), 2);
    assert_eq!(store.get(2).unwrap().datetime().timestamp(), 1);
}

#[test]
fn test_add_with_duplicate_filter_enabled() {
    let mut store = CandleStore::new(Vec::new(), 100, true);

    let candle = TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0);
    store.add(candle.clone());
    assert_eq!(store.len(), 1);

    store.add(candle.clone());
    assert_eq!(store.len(), 1);

    let different_candle = TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0);
    store.add(different_candle);
    assert_eq!(store.len(), 2);
}

#[test]
fn test_add_with_duplicate_filter_disabled() {
    let mut store = CandleStore::<TestCandle>::new(Vec::new(), 100, false);

    let candle = TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0);
    store.add(candle.clone());
    assert_eq!(store.len(), 1);

    store.add(candle.clone());
    assert_eq!(store.len(), 2);
}

#[test]
fn test_add_removes_oldest_when_max_size_exceeded() {
    let mut store = CandleStore::new(Vec::new(), 3, false);

    for i in 0..5 {
        store.add(TestCandle::new(i, 100.0, 105.0, 95.0, 102.0, 1000.0));
    }

    assert_eq!(store.len(), 3);
    assert_eq!(store.get(0).unwrap().datetime().timestamp(), 4);
    assert_eq!(store.get(1).unwrap().datetime().timestamp(), 3);
    assert_eq!(store.get(2).unwrap().datetime().timestamp(), 2);
}

#[test]
fn test_add_binary_search_insertion_order() {
    let mut store = CandleStore::<TestCandle>::new(Vec::new(), 100, false);

    store.add(TestCandle::new(5, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(3, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(7, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(2, 100.0, 105.0, 95.0, 102.0, 1000.0));

    assert_eq!(store.len(), 5);
    let timestamps: Vec<i64> = store
        .items()
        .iter()
        .map(|c| c.datetime().timestamp())
        .collect();
    assert_eq!(timestamps, vec![7, 5, 3, 2, 1]);
}

#[test]
fn test_is_rise_true() {
    let candles = vec![
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(store.is_rise(3));
    assert!(store.is_rise(2));
}

#[test]
fn test_is_rise_false() {
    let candles = vec![
        TestCandle::new(3, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
        TestCandle::new(1, 110.0, 115.0, 105.0, 112.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_rise(3));
}

#[test]
fn test_is_rise_with_equal_prices() {
    let candles = vec![
        TestCandle::new(3, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_rise(3));
}

#[test]
fn test_is_rise_insufficient_candles() {
    let candles = vec![TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0)];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_rise(2));
    assert!(!store.is_rise(1));
}

#[test]
fn test_is_fall_true() {
    let candles = vec![
        TestCandle::new(3, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
        TestCandle::new(1, 110.0, 115.0, 105.0, 112.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(store.is_fall(3));
    assert!(store.is_fall(2));
}

#[test]
fn test_is_fall_false() {
    let candles = vec![
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_fall(3));
}

#[test]
fn test_is_fall_with_equal_prices() {
    let candles = vec![
        TestCandle::new(3, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_fall(3));
}

#[test]
fn test_is_fall_insufficient_candles() {
    let candles = vec![TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0)];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert!(!store.is_fall(2));
    assert!(!store.is_fall(1));
}

#[test]
fn test_get_time_ordered_items() {
    let candles = vec![
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);
    let time_ordered = store.get_time_ordered_items();

    assert_eq!(time_ordered.len(), 3);
    assert_eq!(time_ordered[0].datetime().timestamp(), 1);
    assert_eq!(time_ordered[1].datetime().timestamp(), 2);
    assert_eq!(time_ordered[2].datetime().timestamp(), 3);
}

#[test]
fn test_first_and_get() {
    let candles = vec![
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    assert_eq!(store.first().unwrap().datetime().timestamp(), 3);
    assert_eq!(store.get(0).unwrap().datetime().timestamp(), 3);
    assert_eq!(store.get(1).unwrap().datetime().timestamp(), 2);
    assert_eq!(store.get(2).unwrap().datetime().timestamp(), 1);
    assert_eq!(store.get(3), None);
}

#[test]
fn test_items_slice() {
    let candles = vec![
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(2, 105.0, 110.0, 100.0, 107.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);
    let items = store.items();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].datetime().timestamp(), 3);
    assert_eq!(items[1].datetime().timestamp(), 2);
    assert_eq!(items[2].datetime().timestamp(), 1);
}

#[test]
fn test_add_same_timestamp_different_data() {
    let mut store = CandleStore::<TestCandle>::new(Vec::new(), 100, false);

    store.add(TestCandle::new(1, 100.0, 105.0, 95.0, 102.0, 1000.0));
    store.add(TestCandle::new(1, 110.0, 115.0, 105.0, 112.0, 2000.0));

    assert_eq!(store.len(), 2);
}

#[test]
fn test_is_rise_partial_range() {
    let candles = vec![
        TestCandle::new(5, 120.0, 125.0, 115.0, 122.0, 1000.0),
        TestCandle::new(4, 115.0, 120.0, 110.0, 117.0, 1000.0),
        TestCandle::new(3, 110.0, 115.0, 105.0, 112.0, 1000.0),
        TestCandle::new(2, 100.0, 105.0, 95.0, 102.0, 1000.0),
        TestCandle::new(1, 90.0, 95.0, 85.0, 92.0, 1000.0),
    ];

    let store = CandleStore::<TestCandle>::new(candles, 100, false);

    // 모든 가격이 연속적으로 상승하므로 모두 true
    assert!(store.is_rise(3));
    assert!(store.is_rise(4));
    assert!(store.is_rise(5));
}
