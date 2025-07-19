use super::ThreeRSIParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::three_rsi_analyzer::ThreeRSIAnalyzer;
use crate::indicator::ma::MAType;
use anyhow::Result;
use trading_chart::Candle;

/// ThreeRSI 필터 함수
pub fn filter_three_rsi<C: Candle + 'static>(
    symbol: &str,
    params: &ThreeRSIParams,
    candles: &[C],
) -> Result<bool> {
    let ma_type = match params.ma_type.as_str() {
        "EMA" => MAType::EMA,
        "WMA" => MAType::WMA,
        _ => MAType::SMA,
    };

    ThreeRSIFilter::check_filter(
        symbol,
        candles,
        &params.rsi_periods,
        ma_type,
        params.ma_period,
        params.adx_period,
        params.filter_type,
        params.consecutive_n,
    )
}

/// ThreeRSI 필터 유형
#[derive(Debug, Clone)]
pub enum ThreeRSIFilterType {
    /// 0: 모든 RSI가 50 미만
    AllRSILessThan50,
    /// 1: 모든 RSI가 50 이상
    AllRSIGreaterThan50,
    /// 2: RSI 역순 배열
    RSIReverseArrangement,
    /// 3: RSI 정상 배열
    RSIRegularArrangement,
    /// 4: 캔들 저가가 이동평균 아래
    CandleLowBelowMA,
    /// 5: 캔들 고가가 이동평균 위
    CandleHighAboveMA,
    /// 6: ADX가 20 이상
    ADXGreaterThan20,
}

/// ThreeRSI 필터 구조체
pub struct ThreeRSIFilter;

impl ThreeRSIFilter {
    /// ThreeRSI 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        _symbol: &str,
        candles: &[C],
        rsi_periods: &[usize],
        ma_type: MAType,
        ma_period: usize,
        adx_period: usize,
        filter_type: i32,
        consecutive_n: usize,
    ) -> Result<bool> {
        if candles.len() < ma_period || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => ThreeRSIFilterType::AllRSILessThan50,
            1 => ThreeRSIFilterType::AllRSIGreaterThan50,
            2 => ThreeRSIFilterType::RSIReverseArrangement,
            3 => ThreeRSIFilterType::RSIRegularArrangement,
            4 => ThreeRSIFilterType::CandleLowBelowMA,
            5 => ThreeRSIFilterType::CandleHighAboveMA,
            6 => ThreeRSIFilterType::ADXGreaterThan20,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid ThreeRSI filter type: {}",
                    filter_type
                ));
            }
        };

        // ThreeRSI 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer =
            ThreeRSIAnalyzer::new(rsi_periods, &ma_type, ma_period, adx_period, &candle_store);

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for _ in 0..analyzer.items.len() {
            let result = match filter_type {
                ThreeRSIFilterType::AllRSILessThan50 => {
                    analyzer.is_rsi_all_less_than_50(consecutive_n)
                }
                ThreeRSIFilterType::AllRSIGreaterThan50 => {
                    analyzer.is_rsi_all_greater_than_50(consecutive_n)
                }
                ThreeRSIFilterType::RSIReverseArrangement => {
                    analyzer.is_rsi_reverse_arrangement(consecutive_n)
                }
                ThreeRSIFilterType::RSIRegularArrangement => {
                    analyzer.is_rsi_regular_arrangement(consecutive_n)
                }
                ThreeRSIFilterType::CandleLowBelowMA => {
                    analyzer.is_candle_low_below_ma(consecutive_n)
                }
                ThreeRSIFilterType::CandleHighAboveMA => {
                    analyzer.is_candle_high_above_ma(consecutive_n)
                }
                ThreeRSIFilterType::ADXGreaterThan20 => {
                    analyzer.is_adx_greater_than_20(consecutive_n)
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
    fn test_three_rsi_filter() {
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
            ThreeRSIFilter::check_filter("TEST", &candles, &[7, 14, 21], MAType::SMA, 20, 14, 0, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_three_rsi_filter_invalid_type() {
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
            ThreeRSIFilter::check_filter("TEST", &candles, &[7, 14, 21], MAType::SMA, 5, 14, 99, 1);
        assert!(result.is_err());
    }
}
