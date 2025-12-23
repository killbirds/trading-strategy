use super::{SuperTrendFilterType, SuperTrendParams, utils};
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
        params.p,
    )
}

/// SuperTrend 필터 구조체
pub struct SuperTrendFilter;

impl SuperTrendFilter {
    /// SuperTrend 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        _symbol: &str,
        candles: &[C],
        period: usize,
        multiplier: f64,
        filter_type: SuperTrendFilterType,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        // 파라미터 검증
        utils::validate_period(period, "SuperTrend")?;

        // 경계 조건 체크
        let required_length = period.max(consecutive_n);
        if !utils::check_sufficient_candles(candles.len(), required_length, _symbol) {
            return Ok(false);
        }

        // SuperTrend 분석기 생성
        let candle_store = utils::create_candle_store(candles);
        let mut analyzer = SuperTrendAnalyzer::new(&[(period, multiplier)], &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next(candle.clone());
        }

        use crate::analyzer::base::AnalyzerOps;

        // analyzer 메서드들이 이미 consecutive_n을 처리하거나, 직접 호출
        let result = match filter_type {
            SuperTrendFilterType::AllUptrend => analyzer.is_all_uptrend_signal(consecutive_n, 1, p),
            SuperTrendFilterType::AllDowntrend => {
                analyzer.is_all_downtrend_signal(consecutive_n, 1, p)
            }
            SuperTrendFilterType::PriceAboveSupertrend => {
                analyzer.is_price_above_supertrend_continuous(consecutive_n, period, multiplier, p)
            }
            SuperTrendFilterType::PriceBelowSupertrend => {
                analyzer.is_price_below_supertrend_continuous(consecutive_n, period, multiplier, p)
            }
            SuperTrendFilterType::PriceCrossingAbove => analyzer
                .is_price_crossing_above_supertrend_signal(consecutive_n, 1, period, multiplier, p),
            SuperTrendFilterType::PriceCrossingBelow => analyzer
                .is_price_crossing_below_supertrend_signal(consecutive_n, 1, period, multiplier, p),
            SuperTrendFilterType::TrendChanged => analyzer.is_trend_changed_signal(
                consecutive_n,
                1,
                period,
                multiplier,
                consecutive_n,
                p,
            ),
            SuperTrendFilterType::Uptrend => {
                analyzer.is_uptrend(consecutive_n, period, multiplier, p)
            }
            SuperTrendFilterType::Downtrend => {
                analyzer.is_downtrend(consecutive_n, period, multiplier, p)
            }
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

        let result = SuperTrendFilter::check_filter("TEST", &candles, 2, 2.0, 0.into(), 1, 0);
        assert!(result.is_ok());
    }
}
