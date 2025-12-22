use super::ADXParams;
use crate::analyzer::adx_analyzer::ADXAnalyzer;
use crate::candle_store::CandleStore;
use anyhow::Result;
use trading_chart::Candle;

pub fn filter_adx<C: Candle + 'static>(
    coin: &str,
    params: ADXParams,
    candles: &[C],
) -> Result<bool> {
    // 파라미터 유효성 검증
    if params.period == 0 {
        return Err(anyhow::anyhow!(
            "ADX 파라미터 오류: period는 0보다 커야 합니다"
        ));
    }
    if !(0.0..=100.0).contains(&params.threshold) {
        return Err(anyhow::anyhow!(
            "ADX 파라미터 오류: threshold는 0에서 100 사이여야 합니다"
        ));
    }
    if !(0..=30).contains(&params.filter_type) {
        return Err(anyhow::anyhow!(
            "ADX 파라미터 오류: filter_type은 0에서 30 사이여야 합니다"
        ));
    }

    log::debug!(
        "ADX 필터 적용 - 기간: {}, 임계값: {}, 타입: {}, 연속성: {}",
        params.period,
        params.threshold,
        params.filter_type,
        params.consecutive_n
    );

    // 필터링할 코인 식별
    let required_length = params.period * 2 + params.consecutive_n; // ADX 계산에 필요한 최소 기간 + 연속성

    if candles.len() < required_length {
        log::debug!(
            "코인 {} 캔들 데이터 부족: {} < {}",
            coin,
            candles.len(),
            required_length
        );
        return Ok(false);
    }

    // Vec<C>로 캔들 데이터 복사 (CandleStore 생성용)
    let candles_vec = candles.to_vec();

    // CandleStore 생성 및 ADXAnalyzer 초기화
    let candle_store = CandleStore::new(candles_vec, candles.len() * 2, false);

    // ADXAnalyzer 생성 (trading-strategy의 analyzer 사용)
    let adx_periods = vec![params.period];
    let analyzer = ADXAnalyzer::new(&adx_periods, &candle_store);

    // analyzer에서 ADX 값 가져오기
    let adx = analyzer.get_adx(params.period);

    // +DI, -DI 값은 ADXAnalyzer에서 직접 제공하지 않으므로 계산
    // 여기서는 기존 함수를 사용하거나 필터링 로직을 수정해야 함
    let (_, pdi, mdi) = calculate_directional_indicators(candles, params.period);

    if adx.is_nan() {
        log::debug!("코인 {coin} ADX 계산 실패");
        return Ok(false);
    }

    log::debug!("코인 {coin} ADX: {adx:.2}, +DI: {pdi:.2}, -DI: {mdi:.2}");

    let result = match params.filter_type {
        // 0: ADX가 임계값보다 낮은 경우 (약한 추세)
        0 => {
            if params.threshold == 25.0 {
                // 기본 임계값 25인 경우 is_weak_trend 함수 사용
                analyzer.is_weak_trend(params.consecutive_n, params.p)
            } else {
                adx <= params.threshold
            }
        }
        // 1: ADX가 임계값보다 높은 경우 (강한 추세)
        1 => {
            if params.threshold == 25.0 {
                // 기본 임계값 25인 경우 is_strong_trend 함수 사용
                analyzer.is_strong_trend(params.consecutive_n, params.p)
            } else if params.threshold == 50.0 {
                // 임계값 50인 경우 is_very_strong_trend 함수 사용
                analyzer.is_very_strong_trend(params.consecutive_n, params.p)
            } else {
                adx > params.threshold
            }
        }
        // 2: +DI가 -DI보다 높은 경우 (상승 추세)
        2 => pdi > mdi,
        // 3: -DI가 +DI보다 높은 경우 (하락 추세)
        3 => mdi > pdi,
        // 4: ADX가 임계값보다 높고 +DI가 -DI보다 높은 경우 (강한 상승 추세)
        4 => adx > params.threshold && pdi > mdi,
        // 5: ADX가 임계값보다 높고 -DI가 +DI보다 높은 경우 (강한 하락 추세)
        5 => adx > params.threshold && mdi > pdi,
        // 6: ADX가 상승하는 중 (추세 강화)
        6 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                !prev_adx.is_nan() && adx > prev_adx
            }
        }
        // 7: ADX가 하락하는 중 (추세 약화)
        7 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                !prev_adx.is_nan() && adx < prev_adx
            }
        }
        // 8: +DI와 -DI의 간격이 넓어지는 중 (추세 명확화)
        8 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (_, prev_pdi, prev_mdi) = calculate_adx_values(prev_candles, params.period);
                let current_gap = (pdi - mdi).abs();
                let prev_gap = (prev_pdi - prev_mdi).abs();
                current_gap > prev_gap
            }
        }
        // 9: +DI와 -DI의 간격이 좁아지는 중 (추세 불명확)
        9 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (_, prev_pdi, prev_mdi) = calculate_adx_values(prev_candles, params.period);
                let current_gap = (pdi - mdi).abs();
                let prev_gap = (prev_pdi - prev_mdi).abs();
                current_gap < prev_gap
            }
        }
        // 10: ADX가 극도로 높음 (50 이상)
        10 => adx >= 50.0,
        // 11: ADX가 극도로 낮음 (10 이하)
        11 => adx <= 10.0,
        // 12: ADX가 중간 수준 (20-30)
        12 => (20.0..=30.0).contains(&adx),
        // 13: +DI가 -DI를 상향 돌파
        13 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (_, prev_pdi, prev_mdi) = calculate_adx_values(prev_candles, params.period);
                prev_pdi <= prev_mdi && pdi > mdi
            }
        }
        // 14: -DI가 +DI를 상향 돌파
        14 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (_, prev_pdi, prev_mdi) = calculate_adx_values(prev_candles, params.period);
                prev_mdi <= prev_pdi && mdi > pdi
            }
        }
        // 15: ADX가 횡보 중 (변화율이 임계값 이하)
        15 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let change_rate = (adx - prev_adx).abs() / prev_adx;
                change_rate <= 0.05 // 5% 이하 변화
            }
        }
        // 16: ADX가 급등 중 (변화율이 임계값 이상)
        16 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let change_rate = (adx - prev_adx) / prev_adx;
                change_rate >= 0.1 // 10% 이상 증가
            }
        }
        // 17: ADX가 급락 중 (변화율이 임계값 이상)
        17 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let change_rate = (prev_adx - adx) / prev_adx;
                change_rate >= 0.1 // 10% 이상 감소
            }
        }
        // 18: +DI와 -DI가 모두 높음 (강한 방향성)
        18 => pdi > 25.0 && mdi > 25.0,
        // 19: +DI와 -DI가 모두 낮음 (약한 방향성)
        19 => pdi < 15.0 && mdi < 15.0,
        // 20: ADX가 +DI보다 높음 (추세 강도 > 방향성)
        20 => adx > pdi,
        // 21: ADX가 -DI보다 높음 (추세 강도 > 방향성)
        21 => adx > mdi,
        // 22: +DI가 ADX보다 높음 (방향성 > 추세 강도)
        22 => pdi > adx,
        // 23: -DI가 ADX보다 높음 (방향성 > 추세 강도)
        23 => mdi > adx,
        // 24: ADX가 상승 추세에서 하락 전환
        24 => {
            if candles.len() < params.period * 2 + 3 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let prev_prev_candles = &candles[..candles.len() - 2];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let (prev_prev_adx, _, _) = calculate_adx_values(prev_prev_candles, params.period);
                prev_prev_adx < prev_adx && prev_adx > adx
            }
        }
        // 25: ADX가 하락 추세에서 상승 전환
        25 => {
            if candles.len() < params.period * 2 + 3 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let prev_prev_candles = &candles[..candles.len() - 2];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let (prev_prev_adx, _, _) = calculate_adx_values(prev_prev_candles, params.period);
                prev_prev_adx > prev_adx && prev_adx < adx
            }
        }
        // 26: +DI와 -DI가 교차 중 (방향성 불명확)
        26 => {
            let gap = (pdi - mdi).abs();
            gap <= 2.0 // 2% 이하 차이
        }
        // 27: +DI가 극도로 높음 (40 이상)
        27 => pdi >= 40.0,
        // 28: -DI가 극도로 높음 (40 이상)
        28 => mdi >= 40.0,
        // 29: ADX가 안정적 (변화율이 매우 낮음)
        29 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let change_rate = (adx - prev_adx).abs() / prev_adx;
                change_rate <= 0.02 // 2% 이하 변화
            }
        }
        // 30: ADX가 불안정 (변화율이 매우 높음)
        30 => {
            if candles.len() < params.period * 2 + 2 {
                false
            } else {
                let prev_candles = &candles[..candles.len() - 1];
                let (prev_adx, _, _) = calculate_adx_values(prev_candles, params.period);
                let change_rate = (adx - prev_adx).abs() / prev_adx;
                change_rate >= 0.15 // 15% 이상 변화
            }
        }
        _ => false,
    };

    Ok(result)
}

// ADX 값들을 계산하는 헬퍼 함수
fn calculate_adx_values<C: Candle>(candles: &[C], period: usize) -> (f64, f64, f64) {
    let (dx, pdi, mdi) = calculate_directional_indicators(candles, period);
    (dx, pdi, mdi)
}

// 방향 지표(+DI, -DI) 계산 함수
fn calculate_directional_indicators<C: Candle>(candles: &[C], period: usize) -> (f64, f64, f64) {
    if candles.len() < period * 2 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    // 상승 및 하락 이동 값 계산
    let mut tr_values = Vec::with_capacity(candles.len() - 1);
    let mut plus_dm = Vec::with_capacity(candles.len() - 1);
    let mut minus_dm = Vec::with_capacity(candles.len() - 1);

    for i in 1..candles.len() {
        let high = candles[i].high_price();
        let low = candles[i].low_price();
        let prev_high = candles[i - 1].high_price();
        let prev_low = candles[i - 1].low_price();
        let prev_close = candles[i - 1].close_price();

        // True Range 계산
        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        tr_values.push(tr);

        // Plus DM
        let up_move = high - prev_high;
        let down_move = prev_low - low;

        if up_move > down_move && up_move > 0.0 {
            plus_dm.push(up_move);
        } else {
            plus_dm.push(0.0);
        }

        // Minus DM
        if down_move > up_move && down_move > 0.0 {
            minus_dm.push(down_move);
        } else {
            minus_dm.push(0.0);
        }
    }

    // 평균 TR, +DI, -DI 계산
    if tr_values.len() < period {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    let mut smoothed_tr = tr_values.iter().take(period).sum::<f64>();
    let mut smoothed_plus_dm = plus_dm.iter().take(period).sum::<f64>();
    let mut smoothed_minus_dm = minus_dm.iter().take(period).sum::<f64>();

    for i in period..tr_values.len() {
        smoothed_tr = smoothed_tr - (smoothed_tr / period as f64) + tr_values[i];
        smoothed_plus_dm = smoothed_plus_dm - (smoothed_plus_dm / period as f64) + plus_dm[i];
        smoothed_minus_dm = smoothed_minus_dm - (smoothed_minus_dm / period as f64) + minus_dm[i];
    }

    // +DI, -DI 계산
    if smoothed_tr == 0.0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    let plus_di = 100.0 * (smoothed_plus_dm / smoothed_tr);
    let minus_di = 100.0 * (smoothed_minus_dm / smoothed_tr);

    // DX 계산
    let dx = if plus_di + minus_di == 0.0 {
        0.0
    } else {
        let diff = (plus_di - minus_di).abs();
        100.0 * (diff / (plus_di + minus_di))
    };

    (dx, plus_di, minus_di)
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
            filter_type: 10, // ADX가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 11, // ADX가 극도로 낮음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 12, // ADX가 중간 수준
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 13, // +DI가 -DI를 상향 돌파
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 14, // -DI가 +DI를 상향 돌파
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 15, // ADX가 횡보 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 16, // ADX가 급등 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 17, // ADX가 급락 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 18, // +DI와 -DI가 모두 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 19, // +DI와 -DI가 모두 낮음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 20, // ADX가 +DI보다 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 22, // +DI가 ADX보다 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 24, // ADX가 상승 추세에서 하락 전환
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 25, // ADX가 하락 추세에서 상승 전환
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 26, // +DI와 -DI가 교차 중
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 27, // +DI가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 28, // -DI가 극도로 높음
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 29, // ADX가 안정적
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
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
            filter_type: 30, // ADX가 불안정
            consecutive_n: 1,
            p: 0,
        };

        let result = filter_adx("TEST", params, &candles);
        assert!(result.is_ok());
        let is_unstable = result.unwrap();
        println!("ADX 불안정 테스트 결과: {is_unstable}");
    }
}
