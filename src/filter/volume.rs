use super::{VolumeFilterType, VolumeParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::volume_analyzer::VolumeAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// Volume 필터 함수
pub fn filter_volume<C: Candle + 'static>(
    symbol: &str,
    params: &VolumeParams,
    candles: &[C],
) -> Result<bool> {
    VolumeFilter::check_filter(
        symbol,
        candles,
        params.period,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
        params.p,
        params.stable_min_threshold,
    )
}

/// Volume 필터 구조체
pub struct VolumeFilter;

impl VolumeFilter {
    /// Volume 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        _symbol: &str,
        candles: &[C],
        period: usize,
        threshold: f64,
        filter_type: VolumeFilterType,
        consecutive_n: usize,
        p: usize,
        stable_min_threshold: f64,
    ) -> Result<bool> {
        // 파라미터 검증
        utils::validate_period(period, "Volume")?;

        // 경계 조건 체크
        let required_length = period.max(consecutive_n);
        if !utils::check_sufficient_candles(candles.len(), required_length, _symbol) {
            return Ok(false);
        }

        // Volume 분석기 생성
        let candle_store = utils::create_candle_store(candles);
        let mut analyzer = VolumeAnalyzer::new(&[period], &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next(candle.clone());
        }

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
            VolumeFilterType::BullishWithIncreasedVolume => {
                analyzer.is_bullish_with_increased_volume(consecutive_n, period, p)
            }
            VolumeFilterType::BearishWithIncreasedVolume => {
                analyzer.is_bearish_with_increased_volume(consecutive_n, period, p)
            }
            VolumeFilterType::IncreasingVolumeInUptrend => {
                analyzer.is_increasing_volume_in_uptrend(period, consecutive_n)
            }
            VolumeFilterType::DecreasingVolumeInDowntrend => {
                analyzer.is_decreasing_volume_in_downtrend(period, consecutive_n)
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
                    let is_bullish = analyzer.items[p].candle.close_price()
                        > analyzer.items[p].candle.open_price();
                    let is_decreased = analyzer.is_volume_below_average(consecutive_n, p);
                    is_bullish && is_decreased
                }
            }
            VolumeFilterType::BearishWithDecreasedVolume => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    let is_bearish = analyzer.items[p].candle.close_price()
                        < analyzer.items[p].candle.open_price();
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

        let result = VolumeFilter::check_filter("TEST", &candles, 3, 1.5, 0.into(), 1, 0, 0.1);
        assert!(result.is_ok());
    }
}
