use super::{FilterError, Result, SupportResistanceFilterType, SupportResistanceParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::support_resistance_analyzer::SupportResistanceAnalyzer;
use crate::candle_store::CandleStore;
use trading_chart::Candle;

/// SupportResistance 필터 함수
pub(crate) fn filter_support_resistance<C: Candle + 'static>(
    symbol: &str,
    params: &SupportResistanceParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    SupportResistanceFilter::matches_filter(symbol, candle_store, params, current_price)
}

/// SupportResistance 필터 구조체
pub struct SupportResistanceFilter;

impl SupportResistanceFilter {
    /// SupportResistance 필터 확인 (내부 헬퍼 함수, CandleStore 재사용)
    pub(crate) fn matches_filter<C: Candle + 'static>(
        _symbol: &str,
        candle_store: &CandleStore<C>,
        params: &SupportResistanceParams,
        current_price: f64,
    ) -> Result<bool> {
        let lookback_period = params.lookback_period;
        let touch_threshold = params.touch_threshold;
        let min_touch_count = params.min_touch_count;
        let threshold = params.threshold;
        let filter_type = params.filter_type;
        let consecutive_n = params.consecutive_n;
        let p = params.p;
        // 파라미터 검증
        utils::validate_period(lookback_period, "SupportResistance lookback_period")?;
        if min_touch_count == 0 {
            return Err(FilterError::InvalidSupportResistanceMinTouchCount);
        }

        // 경계 조건 체크
        let required_length = lookback_period.max(consecutive_n);
        if !utils::check_sufficient_candles(candle_store.len(), required_length, _symbol) {
            return Ok(false);
        }
        // analyzer는 이미 init_from_storage로 초기화되었으므로 추가 처리 불필요
        let analyzer = SupportResistanceAnalyzer::new(
            candle_store,
            lookback_period,
            touch_threshold,
            min_touch_count,
        );

        // analyzer 메서드들이 이미 consecutive_n을 처리하므로 직접 호출
        let result = match filter_type {
            SupportResistanceFilterType::SupportBreakdown => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_support
                            .as_ref()
                            .is_some_and(|support| current_price < support.price)
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::ResistanceBreakout => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_resistance
                            .as_ref()
                            .is_some_and(|resistance| current_price > resistance.price)
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::SupportBounce => analyzer.is_break_through_by_satisfying(
                |data| {
                    data.nearest_support.as_ref().is_some_and(|support| {
                        (current_price - support.price).abs() <= support.price * 0.002
                    })
                },
                consecutive_n,
                1,
                p,
            ),
            SupportResistanceFilterType::ResistanceRejection => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_resistance.as_ref().is_some_and(|resistance| {
                            (current_price - resistance.price).abs() <= resistance.price * 0.002
                        })
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::NearStrongSupport => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_support.as_ref().is_some_and(|support| {
                            (current_price - support.price).abs() <= threshold
                                && support.touch_count >= 3
                        })
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::NearStrongResistance => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_resistance.as_ref().is_some_and(|resistance| {
                            (current_price - resistance.price).abs() <= threshold
                                && resistance.touch_count >= 3
                        })
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::AboveSupport => analyzer.is_break_through_by_satisfying(
                |data| {
                    data.nearest_support
                        .as_ref()
                        .is_some_and(|support| current_price > support.price)
                },
                consecutive_n,
                1,
                p,
            ),
            SupportResistanceFilterType::BelowResistance => analyzer
                .is_break_through_by_satisfying(
                    |data| {
                        data.nearest_resistance
                            .as_ref()
                            .is_some_and(|resistance| current_price < resistance.price)
                    },
                    consecutive_n,
                    1,
                    p,
                ),
            SupportResistanceFilterType::NearSupport => analyzer.is_break_through_by_satisfying(
                |data| {
                    data.nearest_support
                        .as_ref()
                        .is_some_and(|support| (current_price - support.price).abs() <= threshold)
                },
                consecutive_n,
                1,
                p,
            ),
            SupportResistanceFilterType::NearResistance => analyzer.is_break_through_by_satisfying(
                |data| {
                    data.nearest_resistance.as_ref().is_some_and(|resistance| {
                        (current_price - resistance.price).abs() <= threshold
                    })
                },
                consecutive_n,
                1,
                p,
            ),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    // use trading_chart::BasicCandle;

    #[test]
    fn test_support_resistance_filter() {
        let candles = vec![
            TestCandle {
                timestamp: 1,
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: 2,
                open: 102.0,
                high: 110.0,
                low: 98.0,
                close: 108.0,
                volume: 1200.0,
            },
            TestCandle {
                timestamp: 3,
                open: 108.0,
                high: 115.0,
                low: 105.0,
                close: 112.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: 4,
                open: 112.0,
                high: 120.0,
                low: 108.0,
                close: 118.0,
                volume: 1300.0,
            },
            TestCandle {
                timestamp: 5,
                open: 118.0,
                high: 125.0,
                low: 115.0,
                close: 122.0,
                volume: 1250.0,
            },
        ];

        let candle_store = utils::create_candle_store(&candles);
        let params = SupportResistanceParams {
            lookback_period: 3,
            touch_threshold: 0.01,
            min_touch_count: 2,
            threshold: 0.05,
            filter_type: SupportResistanceFilterType::SupportBreakdown,
            consecutive_n: 1,
            p: 0,
        };
        let result = SupportResistanceFilter::matches_filter("TEST", &candle_store, &params, 0.0);
        assert!(result.is_ok());
    }
}
