use super::{
    FilterError, ParabolicSARFilterType, ParabolicSARParams, Result,
    standalone_indicator as helper, utils,
};
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::parabolic_sar_analyzer::{ParabolicSARAnalyzer, ParabolicSARAnalyzerParams};
use crate::candle_store::CandleStore;
use trading_chart::Candle;

#[derive(Clone, Copy)]
enum CrossDirection {
    Above,
    Below,
}

/// `consecutive_n`개의 연속 봉에서 가격이 SAR을 위/아래로 돌파했는지 확인한다.
///
/// items 는 newest-first 이고 `p == 0` 일 때 items[0] 이 "현재 시점"이다.
/// 이 경우에 한해 외부에서 전달된 실시간 `current_price`로 cross 를 판정하고,
/// 그 외(과거 시점 오프셋이거나 윈도우 안의 오래된 봉)에는 해당 봉의 close 를 쓴다.
/// 모든 봉에 외부 가격을 적용하면 `consecutive_n > 1` 일 때 의미가 깨진다.
fn price_cross<C: Candle + 'static>(
    analyzer: &ParabolicSARAnalyzer<C>,
    consecutive_n: usize,
    p: usize,
    current_price: f64,
    direction: CrossDirection,
) -> bool {
    let items = analyzer.items();
    if items.len() < p + consecutive_n + 1 {
        return false;
    }

    for offset in 0..consecutive_n {
        let index = p + offset;
        let current = match items.get(index) {
            Some(d) => d,
            None => return false,
        };
        let previous = match items.get(index + 1) {
            Some(d) => d,
            None => return false,
        };

        // index == 0 (= p == 0 && offset == 0) 인 봉만 실시간 current_price를 사용한다.
        let current_close = if index == 0 {
            current_price
        } else {
            current.candle.close_price()
        };
        let prev_close = previous.candle.close_price();
        let crossed = match direction {
            CrossDirection::Above => {
                current_close > current.parabolic_sar.value()
                    && prev_close <= previous.parabolic_sar.value()
            }
            CrossDirection::Below => {
                current_close < current.parabolic_sar.value()
                    && prev_close >= previous.parabolic_sar.value()
            }
        };
        if !crossed {
            return false;
        }
    }
    true
}

pub(crate) fn filter_parabolic_sar<C: Candle + 'static>(
    symbol: &str,
    params: &ParabolicSARParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    validate_params(params)?;
    let needs_previous = matches!(
        params.filter_type,
        ParabolicSARFilterType::Reversal
            | ParabolicSARFilterType::PriceCrossAbove
            | ParabolicSARFilterType::PriceCrossBelow
    );
    let required = helper::required_with_offsets(
        2,
        params.consecutive_n,
        params.p,
        needs_previous,
        "ParabolicSAR required candles",
    )?;
    if !utils::check_sufficient_candles(candle_store.len(), required, symbol) {
        return Ok(false);
    }
    let analyzer = ParabolicSARAnalyzer::new(
        candle_store,
        ParabolicSARAnalyzerParams {
            step: params.step,
            max_step: params.max_step,
        },
    );
    Ok(match params.filter_type {
        ParabolicSARFilterType::Bullish => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                data.parabolic_sar.is_long()
            })
        }
        ParabolicSARFilterType::Bearish => {
            helper::matches_all(&analyzer, params.consecutive_n, params.p, |data| {
                !data.parabolic_sar.is_long()
            })
        }
        ParabolicSARFilterType::Reversal => helper::matches_previous(
            &analyzer,
            params.consecutive_n,
            params.p,
            |current, previous| current.parabolic_sar.is_long() != previous.parabolic_sar.is_long(),
        ),
        ParabolicSARFilterType::PriceCrossAbove => price_cross(
            &analyzer,
            params.consecutive_n,
            params.p,
            current_price,
            CrossDirection::Above,
        ),
        ParabolicSARFilterType::PriceCrossBelow => price_cross(
            &analyzer,
            params.consecutive_n,
            params.p,
            current_price,
            CrossDirection::Below,
        ),
    })
}

pub(crate) fn validate_params(params: &ParabolicSARParams) -> Result<()> {
    utils::validate_positive_number(params.step, "ParabolicSAR step")?;
    utils::validate_positive_number(params.max_step, "ParabolicSAR max_step")?;
    if params.step > params.max_step {
        return Err(FilterError::InvalidPeriodOrder {
            param_name: "ParabolicSAR".to_string(),
            left_name: "step".to_string(),
            left: (params.step * 10_000.0) as usize,
            right_name: "max_step".to_string(),
            right: (params.max_step * 10_000.0) as usize,
        });
    }
    helper::validate_common(params.consecutive_n, "ParabolicSAR consecutive_n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    fn candle_store(candles: Vec<TestCandle>) -> CandleStore<TestCandle> {
        CandleStore::new(candles, 1000, false)
    }

    fn params(filter_type: ParabolicSARFilterType, consecutive_n: usize) -> ParabolicSARParams {
        ParabolicSARParams {
            filter_type,
            consecutive_n,
            ..ParabolicSARParams::default()
        }
    }

    #[test]
    fn price_cross_above_uses_external_price_only_for_latest_bar() {
        let store = candle_store(vec![
            candle(1, 20.5, 19.5, 20.0),
            candle(2, 18.5, 17.5, 18.0),
            candle(3, 16.5, 15.5, 16.0),
            candle(4, 14.5, 13.5, 14.0),
        ]);

        assert!(
            filter_parabolic_sar(
                "TEST/USDT",
                &params(ParabolicSARFilterType::PriceCrossAbove, 1),
                &store,
                100.0,
            )
            .unwrap()
        );
        assert!(
            !filter_parabolic_sar(
                "TEST/USDT",
                &params(ParabolicSARFilterType::PriceCrossAbove, 2),
                &store,
                100.0,
            )
            .unwrap(),
            "historical bars must use their candle close, not the external current_price"
        );
    }

    #[test]
    fn price_cross_below_uses_external_price_only_for_latest_bar() {
        let store = candle_store(vec![
            candle(1, 10.5, 9.5, 10.0),
            candle(2, 12.5, 11.5, 12.0),
            candle(3, 14.5, 13.5, 14.0),
            candle(4, 16.5, 15.5, 16.0),
        ]);

        assert!(
            filter_parabolic_sar(
                "TEST/USDT",
                &params(ParabolicSARFilterType::PriceCrossBelow, 1),
                &store,
                -100.0,
            )
            .unwrap()
        );
        assert!(
            !filter_parabolic_sar(
                "TEST/USDT",
                &params(ParabolicSARFilterType::PriceCrossBelow, 2),
                &store,
                -100.0,
            )
            .unwrap(),
            "historical bars must use their candle close, not the external current_price"
        );
    }
}
