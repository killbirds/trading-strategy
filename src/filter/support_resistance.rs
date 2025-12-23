use super::{SupportResistanceFilterType, SupportResistanceParams, utils};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::support_resistance_analyzer::SupportResistanceAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// SupportResistance 필터 함수
pub fn filter_support_resistance<C: Candle + 'static>(
    symbol: &str,
    params: &SupportResistanceParams,
    candles: &[C],
) -> Result<bool> {
    SupportResistanceFilter::check_filter(
        symbol,
        candles,
        params.lookback_period,
        params.touch_threshold,
        params.min_touch_count,
        params.threshold,
        params.filter_type,
        params.consecutive_n,
        params.p,
    )
}

/// SupportResistance 필터 구조체
pub struct SupportResistanceFilter;

impl SupportResistanceFilter {
    /// SupportResistance 필터 확인
    pub fn check_filter<C: Candle + 'static>(
        _symbol: &str,
        candles: &[C],
        lookback_period: usize,
        touch_threshold: f64,
        min_touch_count: usize,
        threshold: f64,
        filter_type: SupportResistanceFilterType,
        consecutive_n: usize,
        p: usize,
    ) -> Result<bool> {
        // 파라미터 검증
        utils::validate_period(lookback_period, "SupportResistance lookback_period")?;
        if min_touch_count == 0 {
            return Err(anyhow::anyhow!(
                "SupportResistance 파라미터 오류: min_touch_count는 0보다 커야 합니다"
            ));
        }

        // 경계 조건 체크
        let required_length = lookback_period.max(consecutive_n);
        if !utils::check_sufficient_candles(candles.len(), required_length, _symbol) {
            return Ok(false);
        }

        // SupportResistance 분석기 생성
        let candle_store = utils::create_candle_store(candles);
        let mut analyzer = SupportResistanceAnalyzer::new(
            &candle_store,
            lookback_period,
            touch_threshold,
            min_touch_count,
        );

        // 캔들 데이터 처리
        for candle in candles {
            analyzer.next(candle.clone());
        }

        // analyzer 메서드들이 이미 consecutive_n을 처리하므로 직접 호출
        let result = match filter_type {
            SupportResistanceFilterType::SupportBreakdown => {
                analyzer.is_support_breakdown_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::ResistanceBreakout => {
                analyzer.is_resistance_breakout_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::SupportBounce => {
                analyzer.is_support_bounce_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::ResistanceRejection => {
                analyzer.is_resistance_rejection_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::NearStrongSupport => {
                analyzer.is_near_strong_support_signal(consecutive_n, 1, threshold, p)
            }
            SupportResistanceFilterType::NearStrongResistance => {
                analyzer.is_near_strong_resistance_signal(consecutive_n, 1, threshold, p)
            }
            SupportResistanceFilterType::AboveSupport => {
                analyzer.is_above_support_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::BelowResistance => {
                analyzer.is_below_resistance_signal(consecutive_n, 1, p)
            }
            SupportResistanceFilterType::NearSupport => {
                analyzer.is_near_support_signal(consecutive_n, 1, threshold, p)
            }
            SupportResistanceFilterType::NearResistance => {
                analyzer.is_near_resistance_signal(consecutive_n, 1, threshold, p)
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
    fn test_support_resistance_filter() {
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

        let result = SupportResistanceFilter::check_filter(
            "TEST",
            &candles,
            3,
            0.01,
            2,
            0.05,
            0.into(),
            1,
            0,
        );
        assert!(result.is_ok());
    }
}
