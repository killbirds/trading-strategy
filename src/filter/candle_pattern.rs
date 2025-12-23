use super::{CandlePatternFilterType, CandlePatternParams, utils};
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
        params.p,
    )
}

/// CandlePattern 필터 구조체
pub struct CandlePatternFilter;

impl CandlePatternFilter {
    /// CandlePattern 필터 확인
    pub fn check_filter<C: Candle + Clone + 'static>(
        _symbol: &str,
        candles: &[C],
        min_body_ratio: f64,
        min_shadow_ratio: f64,
        pattern_history_length: usize,
        threshold: f64,
        filter_type: CandlePatternFilterType,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        // 파라미터 검증
        if pattern_history_length == 0 {
            return Err(anyhow::anyhow!(
                "CandlePattern 파라미터 오류: pattern_history_length는 0보다 커야 합니다"
            ));
        }

        // 경계 조건 체크
        let required_length = pattern_history_length.max(consecutive_n);
        if !utils::check_sufficient_candles(candles.len(), required_length, _symbol) {
            return Ok(false);
        }

        // CandlePattern 분석기 생성
        let candle_store = utils::create_candle_store(candles);
        let mut analyzer = CandlePatternAnalyzer::new(
            &candle_store,
            min_body_ratio,
            min_shadow_ratio,
            pattern_history_length,
        );

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next(candle.clone());
        }

        use crate::analyzer::candle_pattern_analyzer::{MultiCandlePattern, SingleCandlePattern};

        // analyzer 메서드들이 이미 consecutive_n을 처리하므로 직접 호출
        let result = match filter_type {
            CandlePatternFilterType::StrongBullishPattern => {
                analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::StrongBearishPattern => {
                analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::ReversalPattern => {
                analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::ContinuationPattern => {
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::VolumeConfirmedPattern => {
                analyzer.is_volume_confirmed_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::HighReliabilityPattern => {
                analyzer.is_high_reliability_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::ContextAlignedPattern => {
                analyzer.is_context_aligned_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::StrongReversalSignal => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    analyzer.items[p].is_strong_bullish_pattern()
                        || analyzer.items[p].is_strong_bearish_pattern()
                }
            }
            CandlePatternFilterType::HighConfidenceSignal => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    analyzer.items[p].is_high_reliability_pattern()
                        && analyzer.items[p].pattern_analysis.confidence_score > 0.8
                }
            }
            CandlePatternFilterType::VolumeConfirmedSignal => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    analyzer.items[p].is_volume_confirmed_pattern()
                }
            }
            CandlePatternFilterType::PatternClusteringSignal => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    analyzer.items[p].calculate_pattern_clustering_score() > threshold
                }
            }
            CandlePatternFilterType::HammerPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.single_pattern,
                        SingleCandlePattern::Hammer
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::ShootingStarPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.single_pattern,
                        SingleCandlePattern::ShootingStar
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::DojiPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.single_pattern,
                        SingleCandlePattern::Doji
                            | SingleCandlePattern::GravestoneDoji
                            | SingleCandlePattern::DragonFlyDoji
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::SpinningTopPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.single_pattern,
                        SingleCandlePattern::SpinningTop
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::MarubozuPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.single_pattern,
                        SingleCandlePattern::Marubozu
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::MorningStarPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::MorningStar
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::EveningStarPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::EveningStar
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::EngulfingPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::BullishEngulfing | MultiCandlePattern::BearishEngulfing
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::PiercingPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::PiercingPattern
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::DarkCloudPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::DarkCloudCover
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::HaramiPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::Harami
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::TweezerPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::TweezerTop | MultiCandlePattern::TweezerBottom
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::TriStarPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::ThreeWhiteSoldiers
                            | MultiCandlePattern::ThreeBlackCrows
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::AdvanceBlockPattern => {
                // Note: Uses continuation pattern signal (same as DeliberanceBlockPattern, ConcealmentPattern, GapPattern, TrianglePattern, FlagPattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::DeliberanceBlockPattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, ConcealmentPattern, GapPattern, TrianglePattern, FlagPattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::BreakawayPattern => {
                analyzer.is_strong_reversal_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::ConcealmentPattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, DeliberanceBlockPattern, GapPattern, TrianglePattern, FlagPattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::CounterattackPattern => {
                analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::DarkCloudCoverPattern => analyzer.is_all(
                |data| {
                    matches!(
                        data.pattern_analysis.multi_pattern,
                        MultiCandlePattern::DarkCloudCover
                    )
                },
                consecutive_n,
                p,
            ),
            CandlePatternFilterType::RisingWindowPattern => {
                analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::FallingWindowPattern => {
                analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::HighBreakoutPattern => {
                analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::LowBreakoutPattern => {
                analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::GapPattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, DeliberanceBlockPattern, ConcealmentPattern, TrianglePattern, FlagPattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::GapFillPattern => {
                analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::DoubleBottomPattern => {
                // Note: Uses strong bullish pattern signal (same as RisingWindowPattern, HighBreakoutPattern)
                analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::DoubleTopPattern => {
                // Note: Uses strong bearish pattern signal (same as FallingWindowPattern, LowBreakoutPattern)
                analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::TrianglePattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, DeliberanceBlockPattern, ConcealmentPattern, GapPattern, FlagPattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::FlagPattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, DeliberanceBlockPattern, ConcealmentPattern, GapPattern, TrianglePattern, PennantPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
            }
            CandlePatternFilterType::PennantPattern => {
                // Note: Uses continuation pattern signal (same as AdvanceBlockPattern, DeliberanceBlockPattern, ConcealmentPattern, GapPattern, TrianglePattern, FlagPattern)
                analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
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

        let result =
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 0.into(), 1, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_candle_pattern_filter_hammer() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 11.into(), 1, 0);
        assert!(result.is_ok());
        let is_hammer = result.unwrap();
        println!("해머 패턴 테스트 결과: {is_hammer}");
    }

    #[test]
    fn test_candle_pattern_filter_shooting_star() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 12.into(), 1, 0);
        assert!(result.is_ok());
        let is_shooting_star = result.unwrap();
        println!("샷팅 스타 패턴 테스트 결과: {is_shooting_star}");
    }

    #[test]
    fn test_candle_pattern_filter_doji() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 13.into(), 1, 0);
        assert!(result.is_ok());
        let is_doji = result.unwrap();
        println!("도지 패턴 테스트 결과: {is_doji}");
    }

    #[test]
    fn test_candle_pattern_filter_morning_star() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 16.into(), 1, 0);
        assert!(result.is_ok());
        let is_morning_star = result.unwrap();
        println!("모닝 스타 패턴 테스트 결과: {is_morning_star}");
    }

    #[test]
    fn test_candle_pattern_filter_evening_star() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 17.into(), 1, 0);
        assert!(result.is_ok());
        let is_evening_star = result.unwrap();
        println!("이브닝 스타 패턴 테스트 결과: {is_evening_star}");
    }

    #[test]
    fn test_candle_pattern_filter_engulfing() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 18.into(), 1, 0);
        assert!(result.is_ok());
        let is_engulfing = result.unwrap();
        println!("인걸핑 패턴 테스트 결과: {is_engulfing}");
    }

    #[test]
    fn test_candle_pattern_filter_double_bottom() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 36.into(), 1, 0);
        assert!(result.is_ok());
        let is_double_bottom = result.unwrap();
        println!("이중 바닥 패턴 테스트 결과: {is_double_bottom}");
    }

    #[test]
    fn test_candle_pattern_filter_double_top() {
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 37.into(), 1, 0);
        assert!(result.is_ok());
        let is_double_top = result.unwrap();
        println!("이중 천정 패턴 테스트 결과: {is_double_top}");
    }
}
