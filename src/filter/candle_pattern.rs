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
        params.p,
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
    /// 11: 해머 패턴 (망치형)
    HammerPattern,
    /// 12: 샷팅 스타 패턴 (유성형)
    ShootingStarPattern,
    /// 13: 도지 패턴 (십자형)
    DojiPattern,
    /// 14: 스피닝 탑 패턴 (회전형)
    SpinningTopPattern,
    /// 15: 마루보즈 패턴 (장대형)
    MarubozuPattern,
    /// 16: 모닝 스타 패턴 (새벽별형)
    MorningStarPattern,
    /// 17: 이브닝 스타 패턴 (저녁별형)
    EveningStarPattern,
    /// 18: 인걸핑 패턴 (감싸기형)
    EngulfingPattern,
    /// 19: 피어싱 패턴 (관통형)
    PiercingPattern,
    /// 20: 다크 클라우드 패턴 (어두운 구름형)
    DarkCloudPattern,
    /// 21: 해러미 패턴 (해리어미형)
    HaramiPattern,
    /// 22: 터스터드 패턴 (신뢰형)
    TweezerPattern,
    /// 23: 트라이 스타 패턴 (삼성형)
    TriStarPattern,
    /// 24: 어드밴스 블록 패턴 (진행 블록형)
    AdvanceBlockPattern,
    /// 25: 딜리버런스 블록 패턴 (전달 블록형)
    DeliberanceBlockPattern,
    /// 26: 브레이크어웨이 패턴 (탈출형)
    BreakawayPattern,
    /// 27: 컨실먼트 패턴 (숨김형)
    ConcealmentPattern,
    /// 28: 카운터어택 패턴 (반격형)
    CounterattackPattern,
    /// 29: 다크 클라우드 커버 패턴 (어두운 구름 덮기형)
    DarkCloudCoverPattern,
    /// 30: 라이징 윈도우 패턴 (상승 창형)
    RisingWindowPattern,
    /// 31: 폴링 윈도우 패턴 (하락 창형)
    FallingWindowPattern,
    /// 32: 고가 돌파 패턴
    HighBreakoutPattern,
    /// 33: 저가 돌파 패턴
    LowBreakoutPattern,
    /// 34: 갭 패턴
    GapPattern,
    /// 35: 갭 필 패턴
    GapFillPattern,
    /// 36: 이중 바닥 패턴
    DoubleBottomPattern,
    /// 37: 이중 천정 패턴
    DoubleTopPattern,
    /// 38: 삼각형 패턴
    TrianglePattern,
    /// 39: 플래그 패턴
    FlagPattern,
    /// 40: 페넌트 패턴
    PennantPattern,
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
        filter_type: i32,
        consecutive_n: usize,
        p: usize,
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
            11 => CandlePatternFilterType::HammerPattern,
            12 => CandlePatternFilterType::ShootingStarPattern,
            13 => CandlePatternFilterType::DojiPattern,
            14 => CandlePatternFilterType::SpinningTopPattern,
            15 => CandlePatternFilterType::MarubozuPattern,
            16 => CandlePatternFilterType::MorningStarPattern,
            17 => CandlePatternFilterType::EveningStarPattern,
            18 => CandlePatternFilterType::EngulfingPattern,
            19 => CandlePatternFilterType::PiercingPattern,
            20 => CandlePatternFilterType::DarkCloudPattern,
            21 => CandlePatternFilterType::HaramiPattern,
            22 => CandlePatternFilterType::TweezerPattern,
            23 => CandlePatternFilterType::TriStarPattern,
            24 => CandlePatternFilterType::AdvanceBlockPattern,
            25 => CandlePatternFilterType::DeliberanceBlockPattern,
            26 => CandlePatternFilterType::BreakawayPattern,
            27 => CandlePatternFilterType::ConcealmentPattern,
            28 => CandlePatternFilterType::CounterattackPattern,
            29 => CandlePatternFilterType::DarkCloudCoverPattern,
            30 => CandlePatternFilterType::RisingWindowPattern,
            31 => CandlePatternFilterType::FallingWindowPattern,
            32 => CandlePatternFilterType::HighBreakoutPattern,
            33 => CandlePatternFilterType::LowBreakoutPattern,
            34 => CandlePatternFilterType::GapPattern,
            35 => CandlePatternFilterType::GapFillPattern,
            36 => CandlePatternFilterType::DoubleBottomPattern,
            37 => CandlePatternFilterType::DoubleTopPattern,
            38 => CandlePatternFilterType::TrianglePattern,
            39 => CandlePatternFilterType::FlagPattern,
            40 => CandlePatternFilterType::PennantPattern,
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
            analyzer.next(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for _ in 0..analyzer.items.len() {
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
                CandlePatternFilterType::HammerPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::ShootingStarPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DojiPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::SpinningTopPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::MarubozuPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::MorningStarPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::EveningStarPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::EngulfingPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::PiercingPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DarkCloudPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::HaramiPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::TweezerPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::TriStarPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::AdvanceBlockPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DeliberanceBlockPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::BreakawayPattern => analyzer.is_strong_reversal_signal(),
                CandlePatternFilterType::ConcealmentPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::CounterattackPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DarkCloudCoverPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
                }
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
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::GapFillPattern => {
                    analyzer.is_reversal_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DoubleBottomPattern => {
                    analyzer.is_strong_bullish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::DoubleTopPattern => {
                    analyzer.is_strong_bearish_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::TrianglePattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::FlagPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
                }
                CandlePatternFilterType::PennantPattern => {
                    analyzer.is_continuation_pattern_signal(consecutive_n, 1, p)
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

        let result = CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 0, 1, 0);
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

        let result =
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 99, 1, 0);
        assert!(result.is_err());
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 11, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 12, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 13, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 16, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 17, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 18, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 36, 1, 0);
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
            CandlePatternFilter::check_filter("TEST", &candles, 0.3, 0.3, 5, 0.5, 37, 1, 0);
        assert!(result.is_ok());
        let is_double_top = result.unwrap();
        println!("이중 천정 패턴 테스트 결과: {is_double_top}");
    }
}
