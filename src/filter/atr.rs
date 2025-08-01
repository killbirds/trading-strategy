use super::ATRParams;
use crate::analyzer::atr_analyzer::ATRAnalyzer;
use crate::analyzer::base::AnalyzerOps;
use anyhow::Result;
use trading_chart::Candle;

/// ATR 필터 함수
pub fn filter_atr<C: Candle + 'static>(
    symbol: &str,
    params: &ATRParams,
    candles: &[C],
) -> Result<bool> {
    ATRFilter::check_filter(
        symbol,
        candles,
        params.period,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
        params.p,
    )
}

/// ATR 필터 유형
#[derive(Debug, Clone)]
pub enum ATRFilterType {
    /// 0: ATR이 임계값 이상
    AboveThreshold,
    /// 1: 변동성 확장
    VolatilityExpanding,
    /// 2: 변동성 수축
    VolatilityContracting,
    /// 3: 높은 변동성
    HighVolatility,
    /// 4: 낮은 변동성
    LowVolatility,
    /// 5: 변동성 증가
    VolatilityIncreasing,
    /// 6: 변동성 감소
    VolatilityDecreasing,
}

/// ATR 필터 구조체
pub struct ATRFilter;

impl ATRFilter {
    /// ATR 필터 확인
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
            0 => ATRFilterType::AboveThreshold,
            1 => ATRFilterType::VolatilityExpanding,
            2 => ATRFilterType::VolatilityContracting,
            3 => ATRFilterType::HighVolatility,
            4 => ATRFilterType::LowVolatility,
            5 => ATRFilterType::VolatilityIncreasing,
            6 => ATRFilterType::VolatilityDecreasing,
            _ => return Err(anyhow::anyhow!("Invalid ATR filter type: {}", filter_type)),
        };

        // ATR 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = ATRAnalyzer::new(&[period], &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for i in 0..analyzer.items.len() {
            let result = match filter_type {
                ATRFilterType::AboveThreshold => analyzer.items[i].get_atr(period) > threshold,
                ATRFilterType::VolatilityExpanding => {
                    analyzer.is_volatility_expanding(period, consecutive_n)
                }
                ATRFilterType::VolatilityContracting => {
                    analyzer.is_volatility_contracting(period, consecutive_n)
                }
                ATRFilterType::HighVolatility => {
                    analyzer.is_high_volatility(consecutive_n, period, threshold, p)
                }
                ATRFilterType::LowVolatility => {
                    analyzer.is_low_volatility(consecutive_n, period, threshold, p)
                }
                ATRFilterType::VolatilityIncreasing => {
                    analyzer.is_volatility_increasing(consecutive_n, period)
                }
                ATRFilterType::VolatilityDecreasing => {
                    analyzer.is_volatility_decreasing(consecutive_n, period)
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

        let result = ATRFilter::check_filter("TEST", &candles, 2, 5.0, 0, 1, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_atr_filter_invalid_type() {
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

        let result = ATRFilter::check_filter("TEST", &candles, 2, 5.0, 99, 1, 0);
        assert!(result.is_err());
    }
}
