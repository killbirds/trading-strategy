use super::{ADXFilterType, ADXParams, utils};
use crate::analyzer::adx_analyzer::ADXAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

pub fn filter_adx<C: Candle + 'static>(
    coin: &str,
    params: &ADXParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "ADX 필터 적용 - 기간: {}, 임계값: {}, 타입: {:?}, 연속성: {}",
        params.period,
        params.threshold,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 유효성 검증
    utils::validate_period(params.period, "ADX")?;
    if !(0.0..=100.0).contains(&params.threshold) {
        return Err(anyhow::anyhow!(
            "ADX 파라미터 오류: threshold는 0에서 100 사이여야 합니다"
        ));
    }

    // 필터링할 코인 식별
    let required_length = params.period * 2 + params.consecutive_n; // ADX 계산에 필요한 최소 기간 + 연속성

    // 경계 조건 체크
    if !utils::check_sufficient_candles(candles.len(), required_length, coin) {
        return Ok(false);
    }

    // CandleStore 생성 및 ADXAnalyzer 초기화
    let candle_store = utils::create_candle_store(candles);

    // ADXAnalyzer 생성 (trading-strategy의 analyzer 사용)
    let adx_periods = vec![params.period];
    let analyzer = ADXAnalyzer::new(&adx_periods, &candle_store);

    // analyzer에서 ADX 값 가져오기
    let adx = analyzer.get_adx(params.period);

    if adx.is_nan() {
        log::debug!("코인 {coin} ADX 계산 실패");
        return Ok(false);
    }

    // 로그 출력용으로 +DI, -DI 값 가져오기
    if let Some(adx_data) = analyzer
        .items
        .first()
        .map(|data| data.adxs.get(&params.period))
    {
        log::debug!(
            "코인 {coin} ADX: {adx:.2}, +DI: {:.2}, -DI: {:.2}",
            adx_data.plus_di,
            adx_data.minus_di
        );
    }

    let result = match params.filter_type {
        ADXFilterType::BelowThreshold => {
            if params.threshold == 25.0 {
                // 기본 임계값 25인 경우 is_weak_trend 함수 사용
                analyzer.is_weak_trend(params.consecutive_n, params.p)
            } else {
                // 다른 임계값의 경우 consecutive_n과 p를 고려하여 확인
                if analyzer.items.len() < params.p + params.consecutive_n {
                    false
                } else {
                    analyzer
                        .items
                        .iter()
                        .skip(params.p)
                        .take(params.consecutive_n)
                        .all(|data| data.get_adx(params.period) <= params.threshold)
                }
            }
        }
        ADXFilterType::AboveThreshold => {
            if params.threshold == 25.0 {
                analyzer.is_strong_trend(params.consecutive_n, params.p)
            } else if params.threshold == 50.0 {
                analyzer.is_very_strong_trend(params.consecutive_n, params.p)
            } else {
                // 다른 임계값의 경우 consecutive_n과 p를 고려하여 확인
                if analyzer.items.len() < params.p + params.consecutive_n {
                    false
                } else {
                    analyzer
                        .items
                        .iter()
                        .skip(params.p)
                        .take(params.consecutive_n)
                        .all(|data| data.get_adx(params.period) > params.threshold)
                }
            }
        }
        ADXFilterType::PDIAboveMDI => {
            // consecutive_n과 p를 고려
            if analyzer.items.len() < params.p + params.consecutive_n {
                false
            } else {
                analyzer
                    .items
                    .iter()
                    .skip(params.p)
                    .take(params.consecutive_n)
                    .all(|data| {
                        let adx_data = data.adxs.get(&params.period);
                        adx_data.plus_di > adx_data.minus_di
                    })
            }
        }
        ADXFilterType::MDIAbovePDI => {
            // consecutive_n과 p를 고려
            if analyzer.items.len() < params.p + params.consecutive_n {
                false
            } else {
                analyzer
                    .items
                    .iter()
                    .skip(params.p)
                    .take(params.consecutive_n)
                    .all(|data| {
                        let adx_data = data.adxs.get(&params.period);
                        adx_data.minus_di > adx_data.plus_di
                    })
            }
        }
        ADXFilterType::StrongUptrend => {
            // consecutive_n과 p를 고려
            if analyzer.items.len() < params.p + params.consecutive_n {
                false
            } else {
                analyzer
                    .items
                    .iter()
                    .skip(params.p)
                    .take(params.consecutive_n)
                    .all(|data| {
                        let adx_data = data.adxs.get(&params.period);
                        adx_data.adx > params.threshold && adx_data.plus_di > adx_data.minus_di
                    })
            }
        }
        ADXFilterType::StrongDowntrend => {
            // consecutive_n과 p를 고려
            if analyzer.items.len() < params.p + params.consecutive_n {
                false
            } else {
                analyzer
                    .items
                    .iter()
                    .skip(params.p)
                    .take(params.consecutive_n)
                    .all(|data| {
                        let adx_data = data.adxs.get(&params.period);
                        adx_data.adx > params.threshold && adx_data.minus_di > adx_data.plus_di
                    })
            }
        }
        ADXFilterType::ADXRising => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                current_adx > prev_adx
            }
        }
        ADXFilterType::ADXFalling => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                current_adx < prev_adx
            }
        }
        ADXFilterType::DIGapExpanding => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx_data = analyzer.items[params.p].adxs.get(&params.period);
                let prev_adx_data = analyzer.items[params.p + 1].adxs.get(&params.period);
                let current_gap = (current_adx_data.plus_di - current_adx_data.minus_di).abs();
                let prev_gap = (prev_adx_data.plus_di - prev_adx_data.minus_di).abs();
                current_gap > prev_gap
            }
        }
        ADXFilterType::DIGapContracting => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx_data = analyzer.items[params.p].adxs.get(&params.period);
                let prev_adx_data = analyzer.items[params.p + 1].adxs.get(&params.period);
                let current_gap = (current_adx_data.plus_di - current_adx_data.minus_di).abs();
                let prev_gap = (prev_adx_data.plus_di - prev_adx_data.minus_di).abs();
                current_gap < prev_gap
            }
        }
        ADXFilterType::ExtremeHigh => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                analyzer.items[params.p].get_adx(params.period) >= 50.0
            }
        }
        ADXFilterType::ExtremeLow => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                analyzer.items[params.p].get_adx(params.period) <= 10.0
            }
        }
        ADXFilterType::MiddleLevel => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                (20.0..=30.0).contains(&current_adx)
            }
        }
        ADXFilterType::PDICrossAboveMDI => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx_data = analyzer.items[params.p].adxs.get(&params.period);
                let prev_adx_data = analyzer.items[params.p + 1].adxs.get(&params.period);
                prev_adx_data.plus_di <= prev_adx_data.minus_di
                    && current_adx_data.plus_di > current_adx_data.minus_di
            }
        }
        ADXFilterType::MDICrossAbovePDI => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx_data = analyzer.items[params.p].adxs.get(&params.period);
                let prev_adx_data = analyzer.items[params.p + 1].adxs.get(&params.period);
                prev_adx_data.minus_di <= prev_adx_data.plus_di
                    && current_adx_data.minus_di > current_adx_data.plus_di
            }
        }
        ADXFilterType::Sideways => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                if prev_adx == 0.0 {
                    false
                } else {
                    let change_rate = (current_adx - prev_adx).abs() / prev_adx;
                    change_rate <= 0.05
                }
            }
        }
        ADXFilterType::Surge => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                if prev_adx == 0.0 {
                    false
                } else {
                    let change_rate = (current_adx - prev_adx) / prev_adx;
                    change_rate >= 0.1
                }
            }
        }
        ADXFilterType::Crash => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                if prev_adx == 0.0 {
                    false
                } else {
                    let change_rate = (prev_adx - current_adx) / prev_adx;
                    change_rate >= 0.1
                }
            }
        }
        ADXFilterType::StrongDirectionality => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.plus_di > 25.0 && adx_data.minus_di > 25.0
            }
        }
        ADXFilterType::WeakDirectionality => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.plus_di < 15.0 && adx_data.minus_di < 15.0
            }
        }
        ADXFilterType::TrendStrengthHigherThanDirection => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.adx > adx_data.plus_di
            }
        }
        ADXFilterType::ADXHigherThanMDI => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.adx > adx_data.minus_di
            }
        }
        ADXFilterType::PDIHigherThanADX => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.plus_di > adx_data.adx
            }
        }
        ADXFilterType::MDIHigherThanADX => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.minus_di > adx_data.adx
            }
        }
        ADXFilterType::TrendReversalDown => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                let prev_prev_adx = analyzer.items[params.p + 2].get_adx(params.period);
                prev_prev_adx < prev_adx && prev_adx > current_adx
            }
        }
        ADXFilterType::TrendReversalUp => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                let prev_prev_adx = analyzer.items[params.p + 2].get_adx(params.period);
                prev_prev_adx > prev_adx && prev_adx < current_adx
            }
        }
        ADXFilterType::DICrossover => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                let gap = (adx_data.plus_di - adx_data.minus_di).abs();
                gap <= 2.0
            }
        }
        ADXFilterType::ExtremePDI => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.plus_di >= 40.0
            }
        }
        ADXFilterType::ExtremeMDI => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let adx_data = analyzer.items[params.p].adxs.get(&params.period);
                adx_data.minus_di >= 40.0
            }
        }
        ADXFilterType::Stable => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                if prev_adx == 0.0 {
                    false
                } else {
                    let change_rate = (current_adx - prev_adx).abs() / prev_adx;
                    change_rate <= 0.02
                }
            }
        }
        ADXFilterType::Unstable => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_adx = analyzer.items[params.p].get_adx(params.period);
                let prev_adx = analyzer.items[params.p + 1].get_adx(params.period);
                if prev_adx == 0.0 {
                    false
                } else {
                    let change_rate = (current_adx - prev_adx).abs() / prev_adx;
                    change_rate >= 0.15
                }
            }
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn create_test_candles() -> Vec<TestCandle> {
        vec![
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
            TestCandle {
                timestamp: 6,
                open: 122.0,
                high: 130.0,
                low: 120.0,
                close: 128.0,
                volume: 1400.0,
            },
            TestCandle {
                timestamp: 7,
                open: 128.0,
                high: 135.0,
                low: 125.0,
                close: 132.0,
                volume: 1500.0,
            },
            TestCandle {
                timestamp: 8,
                open: 132.0,
                high: 140.0,
                low: 130.0,
                close: 138.0,
                volume: 1600.0,
            },
        ]
    }

    #[test]
    fn test_adx_filter_extreme_high() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 10.into(), // ADX가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_extreme_high = result.unwrap();
        println!("ADX 극도 높음 테스트 결과: {is_extreme_high}");
    }

    #[test]
    fn test_adx_filter_extreme_low() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 11.into(), // ADX가 극도로 낮음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_extreme_low = result.unwrap();
        println!("ADX 극도 낮음 테스트 결과: {is_extreme_low}");
    }

    #[test]
    fn test_adx_filter_middle_level() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 12.into(), // ADX가 중간 수준
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_middle_level = result.unwrap();
        println!("ADX 중간 수준 테스트 결과: {is_middle_level}");
    }

    #[test]
    fn test_adx_filter_pdi_cross_above_mdi() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 13.into(), // +DI가 -DI를 상향 돌파
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_cross_above = result.unwrap();
        println!("+DI가 -DI를 상향 돌파 테스트 결과: {is_cross_above}");
    }

    #[test]
    fn test_adx_filter_mdi_cross_above_pdi() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 14.into(), // -DI가 +DI를 상향 돌파
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_cross_above = result.unwrap();
        println!("-DI가 +DI를 상향 돌파 테스트 결과: {is_cross_above}");
    }

    #[test]
    fn test_adx_filter_sideways() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 15.into(), // ADX가 횡보 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_sideways = result.unwrap();
        println!("ADX 횡보 테스트 결과: {is_sideways}");
    }

    #[test]
    fn test_adx_filter_surge() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 16.into(), // ADX가 급등 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_surge = result.unwrap();
        println!("ADX 급등 테스트 결과: {is_surge}");
    }

    #[test]
    fn test_adx_filter_crash() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 17.into(), // ADX가 급락 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_crash = result.unwrap();
        println!("ADX 급락 테스트 결과: {is_crash}");
    }

    #[test]
    fn test_adx_filter_strong_directionality() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 18.into(), // +DI와 -DI가 모두 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_strong_directionality = result.unwrap();
        println!("강한 방향성 테스트 결과: {is_strong_directionality}");
    }

    #[test]
    fn test_adx_filter_weak_directionality() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 19.into(), // +DI와 -DI가 모두 낮음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_weak_directionality = result.unwrap();
        println!("약한 방향성 테스트 결과: {is_weak_directionality}");
    }

    #[test]
    fn test_adx_filter_trend_strength_vs_direction() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 20.into(), // ADX가 +DI보다 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_trend_strength_higher = result.unwrap();
        println!("추세 강도 > 방향성 테스트 결과: {is_trend_strength_higher}");
    }

    #[test]
    fn test_adx_filter_direction_vs_trend_strength() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 22.into(), // +DI가 ADX보다 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_direction_higher = result.unwrap();
        println!("방향성 > 추세 강도 테스트 결과: {is_direction_higher}");
    }

    #[test]
    fn test_adx_filter_trend_reversal_down() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 24.into(), // ADX가 상승 추세에서 하락 전환
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_reversal_down = result.unwrap();
        println!("ADX 상승에서 하락 전환 테스트 결과: {is_reversal_down}");
    }

    #[test]
    fn test_adx_filter_trend_reversal_up() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 25.into(), // ADX가 하락 추세에서 상승 전환
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_reversal_up = result.unwrap();
        println!("ADX 하락에서 상승 전환 테스트 결과: {is_reversal_up}");
    }

    #[test]
    fn test_adx_filter_di_crossover() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 26.into(), // +DI와 -DI가 교차 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_crossover = result.unwrap();
        println!("+DI와 -DI 교차 테스트 결과: {is_crossover}");
    }

    #[test]
    fn test_adx_filter_extreme_pdi() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 27.into(), // +DI가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_extreme_pdi = result.unwrap();
        println!("+DI 극도 높음 테스트 결과: {is_extreme_pdi}");
    }

    #[test]
    fn test_adx_filter_extreme_mdi() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 28.into(), // -DI가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_extreme_mdi = result.unwrap();
        println!("-DI 극도 높음 테스트 결과: {is_extreme_mdi}");
    }

    #[test]
    fn test_adx_filter_stable() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 29.into(), // ADX가 안정적
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_stable = result.unwrap();
        println!("ADX 안정적 테스트 결과: {is_stable}");
    }

    #[test]
    fn test_adx_filter_unstable() {
        let candles = create_test_candles();
        let params = ADXParams {
            period: 14,
            threshold: 25.0,
            filter_type: 30.into(), // ADX가 불안정
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", &params, &candles);
        assert!(result.is_ok());
        let is_unstable = result.unwrap();
        println!("ADX 불안정 테스트 결과: {is_unstable}");
    }
}
