use super::MomentumParams;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::momentum_analyzer::MomentumAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// Momentum 필터 함수
pub fn filter_momentum<C: Candle + 'static>(
    symbol: &str,
    params: &MomentumParams,
    candles: &[C],
) -> Result<bool> {
    MomentumFilter::check_filter(
        symbol,
        candles,
        params.rsi_period,
        params.stoch_period,
        params.williams_period,
        params.roc_period,
        params.cci_period,
        params.momentum_period,
        params.history_length,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
        params.p,
    )
}

/// Momentum 필터 유형
#[derive(Debug, Clone)]
pub enum MomentumFilterType {
    /// 0: 강한 양의 모멘텀
    StrongPositiveMomentum,
    /// 1: 강한 음의 모멘텀
    StrongNegativeMomentum,
    /// 2: 가속하는 모멘텀
    AcceleratingMomentum,
    /// 3: 감속하는 모멘텀
    DeceleratingMomentum,
    /// 4: 과매수 상태
    Overbought,
    /// 5: 과매도 상태
    Oversold,
    /// 6: 모멘텀 다이버전스
    MomentumDivergence,
    /// 7: 불리시 다이버전스
    BullishDivergence,
    /// 8: 베어리시 다이버전스
    BearishDivergence,
    /// 9: 지속적인 모멘텀
    PersistentMomentum,
    /// 10: 안정적인 모멘텀
    StableMomentum,
    /// 11: 모멘텀 반전 신호
    MomentumReversalSignal,
}

/// Momentum 필터 구조체
pub struct MomentumFilter;

impl MomentumFilter {
    /// Momentum 필터 확인
    pub fn check_filter<C: Candle + Clone + 'static>(
        _symbol: &str,
        candles: &[C],
        rsi_period: usize,
        stoch_period: usize,
        williams_period: usize,
        roc_period: usize,
        cci_period: usize,
        momentum_period: usize,
        history_length: usize,
        _threshold: f64,
        filter_type: i32,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        if candles.len() < history_length || candles.len() < consecutive_n {
            return Ok(false);
        }

        let filter_type = match filter_type {
            0 => MomentumFilterType::StrongPositiveMomentum,
            1 => MomentumFilterType::StrongNegativeMomentum,
            2 => MomentumFilterType::AcceleratingMomentum,
            3 => MomentumFilterType::DeceleratingMomentum,
            4 => MomentumFilterType::Overbought,
            5 => MomentumFilterType::Oversold,
            6 => MomentumFilterType::MomentumDivergence,
            7 => MomentumFilterType::BullishDivergence,
            8 => MomentumFilterType::BearishDivergence,
            9 => MomentumFilterType::PersistentMomentum,
            10 => MomentumFilterType::StableMomentum,
            11 => MomentumFilterType::MomentumReversalSignal,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid Momentum filter type: {}",
                    filter_type
                ));
            }
        };

        // Momentum 분석기 생성
        let candle_store =
            crate::candle_store::CandleStore::new(candles.to_vec(), candles.len() * 2, false);
        let mut analyzer = MomentumAnalyzer::new(
            &candle_store,
            rsi_period,
            stoch_period,
            williams_period,
            roc_period,
            cci_period,
            momentum_period,
            history_length,
        );

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next_data(candle.clone());
        }

        // 연속적인 조건 확인
        let mut consecutive_count = 0;
        for _ in 0..analyzer.items.len() {
            let result = match filter_type {
                MomentumFilterType::StrongPositiveMomentum => {
                    analyzer.is_strong_positive_momentum_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::StrongNegativeMomentum => {
                    analyzer.is_strong_negative_momentum_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::AcceleratingMomentum => {
                    analyzer.is_accelerating_momentum_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::DeceleratingMomentum => {
                    analyzer.is_decelerating_momentum_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::Overbought => {
                    analyzer.is_overbought_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::Oversold => analyzer.is_oversold_signal(consecutive_n, 1, p),
                MomentumFilterType::MomentumDivergence => {
                    analyzer.is_momentum_divergence_breakthrough(consecutive_n, 1, p)
                }
                MomentumFilterType::BullishDivergence => {
                    analyzer.is_bullish_divergence_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::BearishDivergence => {
                    analyzer.is_bearish_divergence_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::PersistentMomentum => {
                    analyzer.is_persistent_momentum_breakthrough(consecutive_n, 1, p)
                }
                MomentumFilterType::StableMomentum => {
                    analyzer.is_stable_momentum_signal(consecutive_n, 1, p)
                }
                MomentumFilterType::MomentumReversalSignal => {
                    analyzer.is_momentum_reversal_breakthrough(consecutive_n, 1, p)
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
    fn test_momentum_filter() {
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
            MomentumFilter::check_filter("TEST", &candles, 14, 14, 14, 10, 20, 10, 5, 0.5, 0, 1, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_momentum_filter_invalid_type() {
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

        let result = MomentumFilter::check_filter(
            "TEST", &candles, 14, 14, 14, 10, 20, 10, 5, 0.5, 99, 1, 0,
        );
        assert!(result.is_err());
    }
}
