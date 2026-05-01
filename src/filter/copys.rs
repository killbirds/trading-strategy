use super::Result;
use std::fmt;
use trading_chart::Candle;

use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use crate::strategy::copys_common::{
    CopysStrategyCommon, CopysStrategyContext, create_strategy_context_for_filter_with_store,
};
use crate::strategy::{Strategy, StrategyType};

use super::{CopysFilterType, CopysParams};

fn default_copys_ma_periods() -> Vec<usize> {
    vec![5, 20, 60, 120, 200, 240]
}

/// CopyS 모의 전략 (필터 사용을 위한 임시 객체)
struct CopysFilter<C: Candle> {
    ctx: CopysStrategyContext<C>,
    bband_analyzer: BBandAnalyzer<C>,
    params: CopysParams,
}

impl<C: Candle + 'static> CopysFilter<C> {
    fn new_with_store(
        ctx: CopysStrategyContext<C>,
        params: CopysParams,
        candle_store: &CandleStore<C>,
    ) -> Self {
        // CandleStore 재사용하여 볼린저밴드 분석기 초기화
        let bband_analyzer =
            BBandAnalyzer::new(params.bband_period, params.bband_multiplier, candle_store);

        Self {
            ctx,
            bband_analyzer,
            params,
        }
    }
}

// Display 트레이트 구현
impl<C: Candle + 'static> fmt::Display for CopysFilter<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CopyS Filter [RSI: {}/{}]",
            self.params.rsi_lower, self.params.rsi_upper
        )
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysFilter<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle.clone());
        self.bband_analyzer.next(candle);
    }

    fn should_enter(&self, _current_price: f64) -> bool {
        // 전략 트레이트 구현 요구사항
        false
    }

    fn should_exit(&self, _current_price: f64) -> bool {
        // 전략 트레이트 구현 요구사항
        false
    }

    fn position(&self) -> crate::model::PositionType {
        crate::model::PositionType::Long
    }

    fn name(&self) -> StrategyType {
        StrategyType::Copys
    }
}

impl<C: Candle + 'static> CopysStrategyCommon<C> for CopysFilter<C> {
    fn context(&self) -> &CopysStrategyContext<C> {
        &self.ctx
    }

    fn bband_analyzer(&self) -> &BBandAnalyzer<C> {
        &self.bband_analyzer
    }

    fn config_rsi_lower(&self) -> f64 {
        self.params.rsi_lower
    }

    fn config_rsi_upper(&self) -> f64 {
        self.params.rsi_upper
    }

    fn config_rsi_count(&self) -> usize {
        self.params.consecutive_n
    }

    fn config_bband_period(&self) -> usize {
        self.params.bband_period
    }

    fn config_bband_multiplier(&self) -> f64 {
        self.params.bband_multiplier
    }

    fn config_ma_distance_threshold(&self) -> f64 {
        // CopysParams에는 ma_distance_threshold 필드가 없으므로 기본값 반환
        0.02
    }
}

/// CopyS 전략 필터를 적용합니다
pub(crate) fn filter_copys<C: Candle + 'static>(
    symbol: &str,
    params: &CopysParams,
    candle_store: &CandleStore<C>,
    current_price: f64,
) -> Result<bool> {
    if candle_store.len() < 60 {
        log::warn!(
            "코인 {} CopyS 필터에 필요한 캔들 데이터가 부족합니다. 필요: {} >= 60",
            symbol,
            candle_store.len()
        );
        return Ok(false);
    }

    // MAType 설정
    let ma_type = MAType::EMA;
    let ma_periods = if params.ma_periods.is_empty() {
        default_copys_ma_periods()
    } else {
        params.ma_periods.clone()
    };

    // 전략 컨텍스트 생성 (CandleStore 재사용)
    let ctx = match create_strategy_context_for_filter_with_store(
        symbol,
        params.rsi_period,
        &ma_type,
        &ma_periods,
        candle_store,
    ) {
        Ok(context) => context,
        Err(e) => {
            log::warn!("코인 {symbol} CopyS 필터 컨텍스트 생성 실패: {e}");
            return Ok(false);
        }
    };

    // 모의 전략 객체 생성 (CandleStore 재사용)
    // analyzer는 이미 init_from_storage로 초기화되었으므로 추가 처리 불필요
    let filter = CopysFilter::new_with_store(ctx, params.clone(), candle_store);

    // 전략 신호 체크
    let result = match params.filter_type {
        CopysFilterType::BasicBuySignal => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
                params.p,
            );
            let bband_support = is_bband_lower_touch(&filter, current_price, params.p);
            let ma_support = is_ma_support(&filter, current_price, params.p);
            [rsi_oversold, bband_support, ma_support]
                .iter()
                .filter(|&&passed| passed)
                .count()
                >= 2
        }
        CopysFilterType::BasicSellSignal => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
                params.p,
            );
            let bband_resistance = is_bband_upper_touch(&filter, current_price, params.p);
            let ma_resistance = is_ma_resistance(&filter, current_price, params.p);
            [rsi_overbought, bband_resistance, ma_resistance]
                .iter()
                .filter(|&&passed| passed)
                .count()
                >= 2
        }
        CopysFilterType::RSIOversold => filter.context().is_all(
            |data| data.rsi.value() < filter.config_rsi_lower(),
            params.consecutive_n,
            params.p,
        ),
        CopysFilterType::RSIOverbought => filter.context().is_all(
            |data| data.rsi.value() > filter.config_rsi_upper(),
            params.consecutive_n,
            params.p,
        ),
        CopysFilterType::BBandLowerTouch => is_bband_lower_touch(&filter, current_price, params.p),
        CopysFilterType::BBandUpperTouch => is_bband_upper_touch(&filter, current_price, params.p),
        CopysFilterType::MASupport => is_ma_support(&filter, current_price, params.p),
        CopysFilterType::MAResistance => is_ma_resistance(&filter, current_price, params.p),
        CopysFilterType::StrongBuySignal => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
                params.p,
            );
            let bband_support = is_bband_lower_touch(&filter, current_price, params.p);
            let ma_support = is_ma_support(&filter, current_price, params.p);
            rsi_oversold && bband_support && ma_support
        }
        CopysFilterType::StrongSellSignal => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
                params.p,
            );
            let bband_resistance = is_bband_upper_touch(&filter, current_price, params.p);
            let ma_resistance = is_ma_resistance(&filter, current_price, params.p);
            rsi_overbought && bband_resistance && ma_resistance
        }
        CopysFilterType::WeakBuySignal => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
                params.p,
            );
            let bband_support = is_bband_lower_touch(&filter, current_price, params.p);
            let ma_support = is_ma_support(&filter, current_price, params.p);
            let signal_count = [rsi_oversold, bband_support, ma_support]
                .iter()
                .filter(|&&x| x)
                .count();
            signal_count == 1
        }
        CopysFilterType::WeakSellSignal => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
                params.p,
            );
            let bband_resistance = is_bband_upper_touch(&filter, current_price, params.p);
            let ma_resistance = is_ma_resistance(&filter, current_price, params.p);
            let signal_count = [rsi_overbought, bband_resistance, ma_resistance]
                .iter()
                .filter(|&&x| x)
                .count();
            signal_count == 1
        }
        CopysFilterType::RSINeutral => filter.context().is_all(
            |data| {
                let rsi = data.rsi.value();
                rsi >= filter.config_rsi_lower() && rsi <= filter.config_rsi_upper()
            },
            params.consecutive_n,
            params.p,
        ),
        CopysFilterType::BBandInside => {
            filter
                .bband_analyzer()
                .items
                .get(params.p)
                .is_some_and(|data| {
                    current_price <= data.bband.upper() && current_price >= data.bband.lower()
                })
        }
        CopysFilterType::MARegularArrangement => {
            if filter.context().items.len() <= params.p {
                false
            } else {
                let mas = &filter.context().items[params.p].mas;
                if mas.len() < 2 {
                    false
                } else {
                    let short_ma = mas.get_by_key_index(0).get();
                    let long_ma = mas.get_by_key_index(mas.len() - 1).get();
                    short_ma > long_ma
                }
            }
        }
        CopysFilterType::MAReverseArrangement => {
            if filter.context().items.len() <= params.p {
                false
            } else {
                let mas = &filter.context().items[params.p].mas;
                if mas.len() < 2 {
                    false
                } else {
                    let short_ma = mas.get_by_key_index(0).get();
                    let long_ma = mas.get_by_key_index(mas.len() - 1).get();
                    short_ma < long_ma
                }
            }
        }
    };

    Ok(result)
}

fn is_bband_lower_touch<C: Candle + 'static>(
    filter: &CopysFilter<C>,
    current_price: f64,
    p: usize,
) -> bool {
    filter
        .bband_analyzer()
        .items
        .get(p)
        .is_some_and(|data| current_price <= data.bband.lower())
}

fn is_bband_upper_touch<C: Candle + 'static>(
    filter: &CopysFilter<C>,
    current_price: f64,
    p: usize,
) -> bool {
    filter
        .bband_analyzer()
        .items
        .get(p)
        .is_some_and(|data| current_price >= data.bband.upper())
}

fn is_ma_support<C: Candle + 'static>(
    filter: &CopysFilter<C>,
    current_price: f64,
    p: usize,
) -> bool {
    let Some(item) = filter.context().items.get(p) else {
        return false;
    };

    let threshold = filter.config_ma_distance_threshold();
    (0..item.mas.len()).any(|index| {
        let ma_value = item.mas.get_by_key_index(index).get();
        ma_value != 0.0
            && ((current_price - ma_value) / ma_value).abs() <= threshold
            && current_price >= ma_value
    })
}

fn is_ma_resistance<C: Candle + 'static>(
    filter: &CopysFilter<C>,
    current_price: f64,
    p: usize,
) -> bool {
    let Some(item) = filter.context().items.get(p) else {
        return false;
    };

    let threshold = filter.config_ma_distance_threshold();
    (0..item.mas.len()).any(|index| {
        let ma_value = item.mas.get_by_key_index(index).get();
        ma_value != 0.0
            && ((current_price - ma_value) / ma_value).abs() <= threshold
            && current_price <= ma_value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copys_params() {
        // CopysParams 기본값 테스트
        let params = CopysParams::default();
        assert_eq!(params.rsi_period, 14);
        assert_eq!(params.rsi_upper, 70.0);
        assert_eq!(params.rsi_lower, 30.0);
        assert_eq!(params.filter_type, CopysFilterType::BasicBuySignal);
        assert_eq!(params.consecutive_n, 1);
    }
}
