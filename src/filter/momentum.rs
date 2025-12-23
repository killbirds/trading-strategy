use super::{MomentumFilterType, MomentumParams, utils};
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
        filter_type: MomentumFilterType,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        // 파라미터 검증
        utils::validate_period(rsi_period, "Momentum rsi_period")?;
        utils::validate_period(stoch_period, "Momentum stoch_period")?;
        utils::validate_period(williams_period, "Momentum williams_period")?;
        utils::validate_period(roc_period, "Momentum roc_period")?;
        utils::validate_period(cci_period, "Momentum cci_period")?;
        utils::validate_period(momentum_period, "Momentum momentum_period")?;

        // 경계 조건 체크
        let required_length = history_length.max(consecutive_n);
        if !utils::check_sufficient_candles(candles.len(), required_length, _symbol) {
            return Ok(false);
        }

        // Momentum 분석기 생성
        let candle_store = utils::create_candle_store(candles);
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
            analyzer.next(candle.clone());
        }

        // analyzer 메서드들이 이미 consecutive_n을 처리하므로 직접 호출
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
            MomentumFilterType::Overbought => analyzer.is_overbought_signal(consecutive_n, 1, p),
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
            MomentumFilterType::MomentumSideways => analyzer.is_sideways(
                |data| data.momentum_analysis.momentum_strength,
                consecutive_n,
                p,
                _threshold,
            ),
            MomentumFilterType::MomentumSurge => {
                analyzer.is_strong_positive_momentum_signal(consecutive_n, 1, p)
            }
            MomentumFilterType::MomentumCrash => {
                analyzer.is_strong_negative_momentum_signal(consecutive_n, 1, p)
            }
            MomentumFilterType::MomentumConvergence => {
                // Convergence: 다이버전스가 없는 상태 (가격과 모멘텀이 같은 방향으로 움직임)
                analyzer.is_all(|data| !data.has_momentum_divergence(), consecutive_n, p)
            }
            MomentumFilterType::MomentumDivergencePattern => {
                if analyzer.items.len() <= p {
                    false
                } else {
                    analyzer.items[p].has_momentum_divergence()
                        && analyzer.items[p]
                            .momentum_analysis
                            .divergence_analysis
                            .divergence_confidence
                            > 0.7
                }
            }
            MomentumFilterType::MomentumParallel => {
                analyzer.is_persistent_momentum_breakthrough(consecutive_n, 1, p)
            }
            MomentumFilterType::MomentumCrossover => {
                analyzer.is_momentum_reversal_breakthrough(consecutive_n, 1, p)
            }
            MomentumFilterType::MomentumSupportTest => {
                analyzer.is_oversold_signal(consecutive_n, 1, p)
            }
            MomentumFilterType::MomentumResistanceTest => {
                analyzer.is_overbought_signal(consecutive_n, 1, p)
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

        let result = MomentumFilter::check_filter(
            "TEST",
            &candles,
            14,
            14,
            14,
            10,
            20,
            10,
            5,
            0.5,
            0.into(),
            1,
            0,
        );
        assert!(result.is_ok());
    }
}
