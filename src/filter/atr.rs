use super::Result;
use super::{ATRFilterType, ATRParams, utils};
use crate::analyzer::atr_analyzer::ATRAnalyzer;
use crate::candle_store::CandleStore;
use trading_chart::Candle;

/// ATR 필터 함수
pub(crate) fn filter_atr<C: Candle + 'static>(
    symbol: &str,
    params: &ATRParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    ATRFilter::matches_filter(symbol, candle_store, params, current_price)
}

/// ATR 필터 구조체
pub struct ATRFilter;

impl ATRFilter {
    /// ATR 필터 확인
    pub(crate) fn matches_filter<C: Candle + 'static>(
        _symbol: &str,
        candle_store: &CandleStore<C>,
        params: &ATRParams,
        _current_price: f64,
    ) -> Result<bool> {
        let period = params.period;
        let threshold = params.threshold;
        let filter_type = params.filter_type;
        let consecutive_n = params.consecutive_n;
        let p = params.p;

        // 파라미터 검증
        utils::validate_period(period, "ATR")?;

        // 경계 조건 체크
        let required_length = period.max(consecutive_n);
        if !utils::check_sufficient_candles(candle_store.len(), required_length, _symbol) {
            return Ok(false);
        }
        // analyzer는 이미 init_from_storage로 초기화되었으므로 추가 처리 불필요
        let analyzer = ATRAnalyzer::new(&[period], candle_store);

        use crate::analyzer::base::AnalyzerOps;

        // analyzer 메서드들이 이미 consecutive_n을 처리하거나, is_all을 사용하여 처리
        let result = match filter_type {
            ATRFilterType::AboveThreshold => {
                analyzer.is_all(|data| data.get_atr(period) > threshold, consecutive_n, p)
            }
            ATRFilterType::VolatilityExpanding => {
                // p를 고려하여 직접 구현
                let n = consecutive_n.max(1);
                if analyzer.items.len() <= p + n {
                    false
                } else {
                    let current_atr = analyzer.items[p].get_atr(period);
                    let avg_atr: f64 = analyzer.items[p + 1..=p + n]
                        .iter()
                        .map(|item| item.get_atr(period))
                        .sum::<f64>()
                        / n as f64;
                    current_atr > avg_atr
                }
            }
            ATRFilterType::VolatilityContracting => {
                // p를 고려하여 직접 구현
                let n = consecutive_n.max(1);
                if analyzer.items.len() <= p + n {
                    false
                } else {
                    let current_atr = analyzer.items[p].get_atr(period);
                    let avg_atr: f64 = analyzer.items[p + 1..=p + n]
                        .iter()
                        .map(|item| item.get_atr(period))
                        .sum::<f64>()
                        / n as f64;
                    current_atr < avg_atr
                }
            }
            ATRFilterType::HighVolatility => {
                analyzer.is_high_volatility(consecutive_n, period, threshold, p)
            }
            ATRFilterType::LowVolatility => {
                analyzer.is_low_volatility(consecutive_n, period, threshold, p)
            }
            ATRFilterType::VolatilityIncreasing => {
                analyzer.is_volatility_increasing_signal(consecutive_n, 1, period, p)
            }
            ATRFilterType::VolatilityDecreasing => {
                analyzer.is_volatility_decreasing_signal(consecutive_n, 1, period, p)
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    #[test]
    fn test_atr_filter() {
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
        ];

        let candle_store = utils::create_candle_store(&candles);
        let params = ATRParams {
            period: 2,
            threshold: 5.0,
            filter_type: ATRFilterType::AboveThreshold,
            consecutive_n: 1,
            p: 0,
        };
        let result = ATRFilter::matches_filter("TEST", &candle_store, &params, 0.0);
        assert!(result.is_ok());
    }
}
