use super::Result;
use super::{VolumeFilterType, VolumeParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::volume_analyzer::VolumeAnalyzer;
use crate::candle_store::CandleStore;
use trading_chart::Candle;

/// Volume 필터 함수
pub(crate) fn filter_volume<C: Candle + 'static>(
    symbol: &str,
    params: &VolumeParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    VolumeFilter::matches_filter(symbol, candle_store, params, current_price)
}

/// Volume 필터 구조체
pub struct VolumeFilter;

impl VolumeFilter {
    /// Volume 필터 확인 (내부 헬퍼 함수, CandleStore 재사용)
    pub(crate) fn matches_filter<C: Candle + 'static>(
        _symbol: &str,
        candle_store: &CandleStore<C>,
        params: &VolumeParams,
        current_price: f64,
    ) -> Result<bool> {
        let period = params.period;
        let threshold = params.threshold;
        let filter_type = params.filter_type;
        let consecutive_n = params.consecutive_n;
        let p = params.p;
        let stable_min_threshold = params.stable_min_threshold;
        // 파라미터 검증
        utils::validate_period(period, "Volume")?;

        // 경계 조건 체크
        let required_length = period.max(consecutive_n);
        if !utils::check_sufficient_candles(candle_store.len(), required_length, _symbol) {
            return Ok(false);
        }
        // analyzer는 이미 init_from_storage로 초기화되었으므로 추가 처리 불필요
        let analyzer = VolumeAnalyzer::new(&[period], candle_store);

        // analyzer 메서드들이 이미 consecutive_n을 처리하므로 직접 호출
        let result = match filter_type {
            VolumeFilterType::VolumeAboveAverage => {
                analyzer.is_volume_above_average(consecutive_n, p)
            }
            VolumeFilterType::VolumeBelowAverage => {
                analyzer.is_volume_below_average(consecutive_n, p)
            }
            VolumeFilterType::VolumeSurge => analyzer.is_volume_surge(period, threshold),
            VolumeFilterType::VolumeDecline => analyzer.is_volume_decline(period, threshold),
            VolumeFilterType::VolumeSignificantlyAbove => {
                analyzer.is_volume_significantly_above(consecutive_n, threshold, p)
            }
            VolumeFilterType::BullishWithIncreasedVolume => is_directional_volume_above_average(
                &analyzer,
                period,
                consecutive_n,
                p,
                current_price,
                Direction::Bullish,
            ),
            VolumeFilterType::BearishWithIncreasedVolume => is_directional_volume_above_average(
                &analyzer,
                period,
                consecutive_n,
                p,
                current_price,
                Direction::Bearish,
            ),
            VolumeFilterType::IncreasingVolumeInUptrend => is_volume_trending_with_price_direction(
                &analyzer,
                period,
                consecutive_n,
                p,
                current_price,
                Direction::Bullish,
                VolumeTrend::Increasing,
            ),
            VolumeFilterType::DecreasingVolumeInDowntrend => {
                is_volume_trending_with_price_direction(
                    &analyzer,
                    period,
                    consecutive_n,
                    p,
                    current_price,
                    Direction::Bearish,
                    VolumeTrend::Decreasing,
                )
            }
            VolumeFilterType::VolumeSharpDecline => analyzer.is_volume_decline(period, threshold),
            VolumeFilterType::VolumeStable => {
                // VolumeStable은 threshold와 stable_min_threshold 중 큰 값을 사용
                let effective_threshold = threshold.max(stable_min_threshold);
                analyzer.is_sideways(
                    |data| data.get_volume_ratio(period),
                    consecutive_n,
                    p,
                    effective_threshold,
                )
            }
            VolumeFilterType::VolumeVolatile => analyzer.is_volume_surge(period, threshold),
            VolumeFilterType::BullishWithDecreasedVolume => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    let is_bullish = current_price > analyzer.items[p].candle.open_price();
                    let is_decreased = analyzer.is_volume_below_average(consecutive_n, p);
                    is_bullish && is_decreased
                }
            }
            VolumeFilterType::BearishWithDecreasedVolume => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    let is_bearish = current_price < analyzer.items[p].candle.open_price();
                    let is_decreased = analyzer.is_volume_below_average(consecutive_n, p);
                    is_bearish && is_decreased
                }
            }
            VolumeFilterType::VolumeDoubleAverage => {
                // volume_ratio >= 2.0 means current volume is at least 2x the average
                // If threshold is provided, use threshold * 2.0, otherwise use 2.0
                let double_threshold = if threshold > 0.0 {
                    2.0 * threshold
                } else {
                    2.0
                };
                analyzer.is_all(
                    |data| data.get_volume_ratio(period) >= double_threshold,
                    consecutive_n,
                    p,
                )
            }
            VolumeFilterType::VolumeHalfAverage => {
                // volume_ratio <= 0.5 means current volume is at most half of the average
                // If threshold is provided, use threshold * 0.5, otherwise use 0.5
                let half_threshold = if threshold > 0.0 {
                    0.5 * threshold
                } else {
                    0.5
                };
                analyzer.is_all(
                    |data| data.get_volume_ratio(period) <= half_threshold,
                    consecutive_n,
                    p,
                )
            }
            VolumeFilterType::VolumeConsecutiveIncrease => {
                if analyzer.items.len() < p + consecutive_n + 1 {
                    false
                } else {
                    (0..consecutive_n).all(|i| {
                        if let (Some(current), Some(next)) =
                            (analyzer.items.get(p + i), analyzer.items.get(p + i + 1))
                        {
                            current.get_volume_ratio(period) < next.get_volume_ratio(period)
                        } else {
                            false
                        }
                    })
                }
            }
            VolumeFilterType::VolumeConsecutiveDecrease => {
                if analyzer.items.len() < p + consecutive_n + 1 {
                    false
                } else {
                    (0..consecutive_n).all(|i| {
                        if let (Some(current), Some(next)) =
                            (analyzer.items.get(p + i), analyzer.items.get(p + i + 1))
                        {
                            current.get_volume_ratio(period) > next.get_volume_ratio(period)
                        } else {
                            false
                        }
                    })
                }
            }
            VolumeFilterType::VolumeSideways => analyzer.is_sideways(
                |data| data.get_volume_ratio(period),
                consecutive_n,
                p,
                threshold,
            ),
            VolumeFilterType::VolumeExtremelyHigh => analyzer.is_all(
                |data| data.get_volume_ratio(period) >= threshold,
                consecutive_n,
                p,
            ),
            VolumeFilterType::VolumeExtremelyLow => analyzer.is_all(
                |data| data.get_volume_ratio(period) <= threshold,
                consecutive_n,
                p,
            ),
        };

        Ok(result)
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Bullish,
    Bearish,
}

#[derive(Clone, Copy)]
enum VolumeTrend {
    Increasing,
    Decreasing,
}

fn is_directional_volume_above_average<C: Candle + 'static>(
    analyzer: &VolumeAnalyzer<C>,
    period: usize,
    consecutive_n: usize,
    p: usize,
    current_price: f64,
    direction: Direction,
) -> bool {
    if analyzer.items.len() < p + consecutive_n {
        return false;
    }

    (0..consecutive_n).all(|i| {
        analyzer.items.get(p + i).is_some_and(|item| {
            let price_matches = if i == 0 {
                price_matches_direction(current_price, item.candle.open_price(), direction)
            } else {
                price_matches_direction(
                    item.candle.close_price(),
                    item.candle.open_price(),
                    direction,
                )
            };

            price_matches && item.is_current_volume_above_average(period)
        })
    })
}

fn is_volume_trending_with_price_direction<C: Candle + 'static>(
    analyzer: &VolumeAnalyzer<C>,
    period: usize,
    consecutive_n: usize,
    p: usize,
    current_price: f64,
    direction: Direction,
    volume_trend: VolumeTrend,
) -> bool {
    if analyzer.items.len() < p + consecutive_n || consecutive_n < 2 {
        return false;
    }

    let prices_match = (0..consecutive_n).all(|i| {
        analyzer.items.get(p + i).is_some_and(|item| {
            if i == 0 {
                price_matches_direction(current_price, item.candle.open_price(), direction)
            } else {
                price_matches_direction(
                    item.candle.close_price(),
                    item.candle.open_price(),
                    direction,
                )
            }
        })
    });

    if !prices_match {
        return false;
    }

    (0..consecutive_n - 1).all(|i| {
        let Some(current) = analyzer.items.get(p + i) else {
            return false;
        };
        let Some(next) = analyzer.items.get(p + i + 1) else {
            return false;
        };

        let current_ratio = current.get_volume_ratio(period);
        let next_ratio = next.get_volume_ratio(period);
        match volume_trend {
            VolumeTrend::Increasing => current_ratio > next_ratio,
            VolumeTrend::Decreasing => current_ratio < next_ratio,
        }
    })
}

fn price_matches_direction(price: f64, open_price: f64, direction: Direction) -> bool {
    match direction {
        Direction::Bullish => price > open_price,
        Direction::Bearish => price < open_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    // use trading_chart::BasicCandle;

    #[test]
    fn test_volume_filter() {
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
        let params = VolumeParams {
            period: 3,
            threshold: 1.5,
            filter_type: VolumeFilterType::VolumeAboveAverage,
            consecutive_n: 1,
            p: 0,
            stable_min_threshold: 0.1,
        };
        let result = VolumeFilter::matches_filter("TEST", &candle_store, &params, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_volume_bullish_with_increased_volume_uses_external_current_price() {
        let candles = vec![
            TestCandle {
                timestamp: 1,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 100.0,
            },
            TestCandle {
                timestamp: 2,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 100.0,
            },
            TestCandle {
                timestamp: 3,
                open: 100.0,
                high: 101.0,
                low: 89.0,
                close: 90.0,
                volume: 1_000.0,
            },
        ];

        let candle_store = utils::create_candle_store(&candles);
        let params = VolumeParams {
            period: 2,
            threshold: 1.5,
            filter_type: VolumeFilterType::BullishWithIncreasedVolume,
            consecutive_n: 1,
            p: 0,
            stable_min_threshold: 0.1,
        };

        assert!(VolumeFilter::matches_filter("TEST", &candle_store, &params, 120.0).unwrap());
        assert!(!VolumeFilter::matches_filter("TEST", &candle_store, &params, 90.0).unwrap());
    }
}
