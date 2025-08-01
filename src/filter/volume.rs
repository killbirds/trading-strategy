use super::VolumeParams;
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
    )
}

/// Volume 필터 유형
#[derive(Debug, Clone)]
pub enum VolumeFilterType {
    /// 0: 볼륨이 평균 이상
    VolumeAboveAverage,
    /// 1: 볼륨이 평균 이하
    VolumeBelowAverage,
    /// 2: 볼륨 급등
    VolumeSurge,
    /// 3: 볼륨 감소
    VolumeDecline,
    /// 4: 볼륨이 현저히 높음
    VolumeSignificantlyAbove,
    /// 5: 상승과 함께 볼륨 증가
    BullishWithIncreasedVolume,
    /// 6: 하락과 함께 볼륨 증가
    BearishWithIncreasedVolume,
    /// 7: 상승 추세에서 볼륨 증가
    IncreasingVolumeInUptrend,
    /// 8: 하락 추세에서 볼륨 감소
    DecreasingVolumeInDowntrend,
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
        filter_type: i32,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        if candles.len() < period || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => VolumeFilterType::VolumeAboveAverage,
            1 => VolumeFilterType::VolumeBelowAverage,
            2 => VolumeFilterType::VolumeSurge,
            3 => VolumeFilterType::VolumeDecline,
            4 => VolumeFilterType::VolumeSignificantlyAbove,
            5 => VolumeFilterType::BullishWithIncreasedVolume,
            6 => VolumeFilterType::BearishWithIncreasedVolume,
            7 => VolumeFilterType::IncreasingVolumeInUptrend,
            8 => VolumeFilterType::DecreasingVolumeInDowntrend,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid Volume filter type: {}",
                    filter_type
                ));
            }
        };

        // Volume 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = VolumeAnalyzer::new(&[period], &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for _ in 0..analyzer.items.len() {
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
            };

            if result {
                consecutive_count += 1;
                if consecutive_count >= consecutive_n {
                    return Ok(true);
                }
            } else {
                consecutive_count = 0;
            }
        }

        Ok(false)
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

        let result = VolumeFilter::check_filter("TEST", &candles, 3, 1.5, 0, 1, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_volume_filter_invalid_type() {
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

        let result = VolumeFilter::check_filter("TEST", &candles, 3, 1.5, 99, 1, 0);
        assert!(result.is_err());
    }
}
