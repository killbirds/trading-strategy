use super::Result;
use super::{MomentumFilterType, MomentumParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::momentum_analyzer::MomentumAnalyzer;
use crate::candle_store::CandleStore;
use trading_chart::Candle;

/// Momentum 필터 함수
pub(crate) fn filter_momentum<C: Candle + Clone + 'static>(
    symbol: &str,
    params: &MomentumParams,
    candle_store: &CandleStore<C>,
) -> Result<bool> {
    MomentumFilter::matches_filter(symbol, candle_store, params)
}

/// Momentum 필터 구조체
pub struct MomentumFilter;

impl MomentumFilter {
    /// Momentum 필터 확인 (내부 헬퍼 함수, CandleStore 재사용)
    pub(crate) fn matches_filter<C: Candle + Clone + 'static>(
        _symbol: &str,
        candle_store: &CandleStore<C>,
        params: &MomentumParams,
    ) -> Result<bool> {
        let rsi_period = params.rsi_period;
        let stoch_period = params.stoch_period;
        let williams_period = params.williams_period;
        let roc_period = params.roc_period;
        let cci_period = params.cci_period;
        let momentum_period = params.momentum_period;
        let history_length = params.history_length;
        let _threshold = params.threshold;
        let filter_type = params.filter_type;
        let consecutive_n = params.consecutive_n;
        let p = params.p;
        // 파라미터 검증
        utils::validate_period(rsi_period, "Momentum rsi_period")?;
        utils::validate_period(stoch_period, "Momentum stoch_period")?;
        utils::validate_period(williams_period, "Momentum williams_period")?;
        utils::validate_period(roc_period, "Momentum roc_period")?;
        utils::validate_period(cci_period, "Momentum cci_period")?;
        utils::validate_period(momentum_period, "Momentum momentum_period")?;

        // 경계 조건 체크
        let required_length = history_length.max(consecutive_n);
        if !utils::check_sufficient_candles(candle_store.len(), required_length, _symbol) {
            return Ok(false);
        }
        // analyzer는 이미 init_from_storage로 초기화되었으므로 추가 처리 불필요
        let analyzer = MomentumAnalyzer::new(
            candle_store,
            crate::analyzer::momentum_analyzer::MomentumAnalyzerParams {
                rsi_period,
                stoch_period,
                williams_period,
                roc_period,
                cci_period,
                momentum_period,
                history_length,
            },
        );

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

        let candle_store = utils::create_candle_store(&candles);
        let params = MomentumParams {
            rsi_period: 14,
            stoch_period: 14,
            williams_period: 14,
            roc_period: 10,
            cci_period: 20,
            momentum_period: 10,
            history_length: 5,
            threshold: 0.5,
            filter_type: MomentumFilterType::StrongPositiveMomentum,
            consecutive_n: 1,
            p: 0,
        };
        let result = MomentumFilter::matches_filter("TEST", &candle_store, &params);
        assert!(result.is_ok());
    }
}
