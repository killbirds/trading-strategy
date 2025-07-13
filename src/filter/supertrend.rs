use super::SuperTrendParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::supertrend_analyzer::SuperTrendAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// SuperTrend 필터 함수
pub fn filter_supertrend<C: Candle + 'static>(
    symbol: &str,
    params: &SuperTrendParams,
    candles: &[C],
) -> Result<bool> {
    SuperTrendFilter::check_filter(
        symbol,
        candles,
        params.period,
        params.multiplier,
        params.filter_type,
        params.consecutive_n,
    )
}

/// SuperTrend 필터 유형
#[derive(Debug, Clone)]
pub enum SuperTrendFilterType {
    /// 0: 모든 설정에서 상승 추세
    AllUptrend,
    /// 1: 모든 설정에서 하락 추세
    AllDowntrend,
    /// 2: 가격이 슈퍼트렌드 위에 있음
    PriceAboveSupertrend,
    /// 3: 가격이 슈퍼트렌드 아래에 있음
    PriceBelowSupertrend,
    /// 4: 가격이 슈퍼트렌드를 상향 돌파
    PriceCrossingAbove,
    /// 5: 가격이 슈퍼트렌드를 하향 돌파
    PriceCrossingBelow,
    /// 6: 추세 변경
    TrendChanged,
    /// 7: 특정 설정에서 상승 추세
    Uptrend,
    /// 8: 특정 설정에서 하락 추세
    Downtrend,
}

/// SuperTrend 필터 구조체
pub struct SuperTrendFilter;

impl SuperTrendFilter {
    /// SuperTrend 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        symbol: &str,
        candles: &[C],
        period: usize,
        multiplier: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> Result<bool> {
        if candles.len() < period || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => SuperTrendFilterType::AllUptrend,
            1 => SuperTrendFilterType::AllDowntrend,
            2 => SuperTrendFilterType::PriceAboveSupertrend,
            3 => SuperTrendFilterType::PriceBelowSupertrend,
            4 => SuperTrendFilterType::PriceCrossingAbove,
            5 => SuperTrendFilterType::PriceCrossingBelow,
            6 => SuperTrendFilterType::TrendChanged,
            7 => SuperTrendFilterType::Uptrend,
            8 => SuperTrendFilterType::Downtrend,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid SuperTrend filter type: {}",
                    filter_type
                ));
            }
        };

        // SuperTrend 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = SuperTrendAnalyzer::new(&[(period, multiplier)], &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for i in 0..analyzer.items.len() {
            let result = match filter_type {
                SuperTrendFilterType::AllUptrend => analyzer.is_all_uptrend(),
                SuperTrendFilterType::AllDowntrend => analyzer.is_all_downtrend(),
                SuperTrendFilterType::PriceAboveSupertrend => {
                    analyzer.is_price_above_supertrend(&period, &multiplier)
                }
                SuperTrendFilterType::PriceBelowSupertrend => {
                    analyzer.is_price_below_supertrend(&period, &multiplier)
                }
                SuperTrendFilterType::PriceCrossingAbove => {
                    analyzer.is_price_crossing_above_supertrend(&period, &multiplier)
                }
                SuperTrendFilterType::PriceCrossingBelow => {
                    analyzer.is_price_crossing_below_supertrend(&period, &multiplier)
                }
                SuperTrendFilterType::TrendChanged => {
                    analyzer.is_trend_changed(&period, &multiplier, consecutive_n)
                }
                SuperTrendFilterType::Uptrend => {
                    analyzer.is_uptrend(consecutive_n, period, multiplier)
                }
                SuperTrendFilterType::Downtrend => {
                    analyzer.is_downtrend(consecutive_n, period, multiplier)
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
    fn test_supertrend_filter() {
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

        let result = SuperTrendFilter::check_filter("TEST", &candles, 2, 2.0, 0, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_supertrend_filter_invalid_type() {
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

        let result = SuperTrendFilter::check_filter("TEST", &candles, 2, 2.0, 99, 1);
        assert!(result.is_err());
    }
}
