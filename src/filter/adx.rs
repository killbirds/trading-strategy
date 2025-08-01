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
    if !(0..=5).contains(&params.filter_type) {
        return Err(anyhow::anyhow!(
            "ADX 파라미터 오류: filter_type은 0에서 5 사이여야 합니다"
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
