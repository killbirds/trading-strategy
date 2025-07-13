use super::SupportResistanceParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::support_resistance_analyzer::SupportResistanceAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// SupportResistance 필터 함수
pub fn filter_support_resistance<C: Candle + 'static>(
    symbol: &str,
    params: &SupportResistanceParams,
    candles: &[C],
) -> Result<bool> {
    SupportResistanceFilter::check_filter(
        symbol,
        candles,
        params.lookback_period,
        params.touch_threshold,
        params.min_touch_count,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
    )
}

/// SupportResistance 필터 유형
#[derive(Debug, Clone)]
pub enum SupportResistanceFilterType {
    /// 0: 지지선 하향 돌파
    SupportBreakdown,
    /// 1: 저항선 상향 돌파
    ResistanceBreakout,
    /// 2: 지지선 반등
    SupportBounce,
    /// 3: 저항선 거부
    ResistanceRejection,
    /// 4: 강한 지지선 근처
    NearStrongSupport,
    /// 5: 강한 저항선 근처
    NearStrongResistance,
    /// 6: 지지선 위에 있음
    AboveSupport,
    /// 7: 저항선 아래에 있음
    BelowResistance,
    /// 8: 지지선 근처
    NearSupport,
    /// 9: 저항선 근처
    NearResistance,
}

/// SupportResistance 필터 구조체
pub struct SupportResistanceFilter;

impl SupportResistanceFilter {
    /// SupportResistance 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        symbol: &str,
        candles: &[C],
        lookback_period: usize,
        touch_threshold: f64,
        min_touch_count: usize,
        threshold: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> Result<bool> {
        if candles.len() < lookback_period || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => SupportResistanceFilterType::SupportBreakdown,
            1 => SupportResistanceFilterType::ResistanceBreakout,
            2 => SupportResistanceFilterType::SupportBounce,
            3 => SupportResistanceFilterType::ResistanceRejection,
            4 => SupportResistanceFilterType::NearStrongSupport,
            5 => SupportResistanceFilterType::NearStrongResistance,
            6 => SupportResistanceFilterType::AboveSupport,
            7 => SupportResistanceFilterType::BelowResistance,
            8 => SupportResistanceFilterType::NearSupport,
            9 => SupportResistanceFilterType::NearResistance,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid SupportResistance filter type: {}",
                    filter_type
                ));
            }
        };

        // SupportResistance 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = SupportResistanceAnalyzer::new(
            &candle_store,
            lookback_period,
            touch_threshold,
            min_touch_count,
        );

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for i in 0..analyzer.items.len() {
            let result = match filter_type {
                SupportResistanceFilterType::SupportBreakdown => {
                    analyzer.is_support_breakdown_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::ResistanceBreakout => {
                    analyzer.is_resistance_breakout_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::SupportBounce => {
                    analyzer.is_support_bounce_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::ResistanceRejection => {
                    analyzer.is_resistance_rejection_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::NearStrongSupport => {
                    analyzer.is_near_strong_support_signal(consecutive_n, 1, threshold)
                }
                SupportResistanceFilterType::NearStrongResistance => {
                    analyzer.is_near_strong_resistance_signal(consecutive_n, 1, threshold)
                }
                SupportResistanceFilterType::AboveSupport => {
                    analyzer.is_above_support_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::BelowResistance => {
                    analyzer.is_below_resistance_signal(consecutive_n, 1)
                }
                SupportResistanceFilterType::NearSupport => {
                    analyzer.is_near_support_signal(consecutive_n, 1, threshold)
                }
                SupportResistanceFilterType::NearResistance => {
                    analyzer.is_near_resistance_signal(consecutive_n, 1, threshold)
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

        let result =
            SupportResistanceFilter::check_filter("TEST", &candles, 3, 0.01, 2, 0.05, 0, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_support_resistance_filter_invalid_type() {
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

        let result =
            SupportResistanceFilter::check_filter("TEST", &candles, 3, 0.01, 2, 0.05, 99, 1);
        assert!(result.is_err());
    }
}
