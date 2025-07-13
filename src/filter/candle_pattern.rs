use super::CandlePatternParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::candle_pattern_analyzer::CandlePatternAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// CandlePattern 필터 함수
pub fn filter_candle_pattern<C: Candle + 'static>(
    symbol: &str,
    params: &CandlePatternParams,
    candles: &[C],
) -> Result<bool> {
    CandlePatternFilter::check_filter(
        symbol,
        candles,
        params.min_body_ratio,
        params.min_shadow_ratio,
        params.pattern_history_length,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
    )
}

/// CandlePattern 필터 유형
#[derive(Debug, Clone)]
pub enum CandlePatternFilterType {
    /// 0: 강한 상승 패턴
    StrongBullishPattern,
    /// 1: 강한 하락 패턴
    StrongBearishPattern,
    /// 2: 반전 패턴
    ReversalPattern,
    /// 3: 지속 패턴
    ContinuationPattern,
    /// 4: 볼륨으로 확인된 패턴
    VolumeConfirmedPattern,
    /// 5: 높은 신뢰도 패턴
    HighReliabilityPattern,
    /// 6: 컨텍스트에 맞는 패턴
    ContextAlignedPattern,
    /// 7: 강한 반전 신호
    StrongReversalSignal,
    /// 8: 높은 신뢰도 신호
    HighConfidenceSignal,
    /// 9: 볼륨 확인 신호
    VolumeConfirmedSignal,
    /// 10: 패턴 클러스터링 신호
    PatternClusteringSignal,
}

/// CandlePattern 필터 구조체
pub struct CandlePatternFilter;

impl CandlePatternFilter {
    /// CandlePattern 필터 확인
    pub fn check_filter<C: Candle + Clone + 'static>(
        symbol: &str,
        candles: &[C],
        min_body_ratio: f64,
        min_shadow_ratio: f64,
        pattern_history_length: usize,
        threshold: f64,
        filter_type: i32,
        consecutive_n: usize,
    ) -> Result<bool> {
        if candles.len() < pattern_history_length || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => CandlePatternFilterType::StrongBullishPattern,
            1 => CandlePatternFilterType::StrongBearishPattern,
            2 => CandlePatternFilterType::ReversalPattern,
            3 => CandlePatternFilterType::ContinuationPattern,
            4 => CandlePatternFilterType::VolumeConfirmedPattern,
            5 => CandlePatternFilterType::HighReliabilityPattern,
            6 => CandlePatternFilterType::ContextAlignedPattern,
            7 => CandlePatternFilterType::StrongReversalSignal,
            8 => CandlePatternFilterType::HighConfidenceSignal,
            9 => CandlePatternFilterType::VolumeConfirmedSignal,
            10 => CandlePatternFilterType::PatternClusteringSignal,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid CandlePattern filter type: {}",
                    filter_type
                ));
            }
        };

        // CandlePattern 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = CandlePatternAnalyzer::new(
            &candle_store,
            min_body_ratio,
            min_shadow_ratio,
            pattern_history_length,
        );

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for i in 0..analyzer.items.len() {
            let result = match filter_type {
                CandlePatternFilterType::StrongBullishPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::StrongBearishPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::ReversalPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::ContinuationPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::VolumeConfirmedPattern => {
                    analyzer.is_volume_confirmed_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::HighReliabilityPattern => {
                    analyzer.is_high_reliability_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::ContextAlignedPattern => {
                    analyzer.is_context_aligned_pattern_signal(consecutive_n, 1)
                }
                CandlePatternFilterType::StrongReversalSignal => {
                    analyzer.is_strong_reversal_signal()
                }
                CandlePatternFilterType::HighConfidenceSignal => {
                    analyzer.is_high_confidence_signal()
                }
                CandlePatternFilterType::VolumeConfirmedSignal => {
                    analyzer.is_volume_confirmed_signal()
                }
                CandlePatternFilterType::PatternClusteringSignal => {
                    analyzer.is_pattern_clustering_signal(threshold)
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
    fn test_candle_pattern_filter() {
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

        let result = CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 0, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_candle_pattern_filter_invalid_type() {
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

        let result = CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 99, 1);
        assert!(result.is_err());
    }
}
