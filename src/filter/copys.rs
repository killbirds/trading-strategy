use anyhow::Result;
use std::fmt;
use trading_chart::Candle;

use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use crate::strategy::copys_common::{
    CopysStrategyCommon, CopysStrategyContext, create_strategy_context_for_filter,
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
    fn new(ctx: CopysStrategyContext<C>, params: CopysParams, candles: &[C]) -> Self {
        // 캔들 데이터로 CandleStore 생성하여 볼린저밴드 분석기 초기화
        let candles_vec = candles.to_vec();
        let storage = CandleStore::<C>::new(candles_vec, candles.len() * 2, false);
        let bband_analyzer =
            BBandAnalyzer::new(params.bband_period, params.bband_multiplier, &storage);

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

    fn should_enter(&self, _candle: &C) -> bool {
        // 전략 트레이트 구현 요구사항
        false
    }

    fn should_exit(&self, _candle: &C) -> bool {
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
}

/// CopyS 전략 필터를 적용합니다.
pub fn filter_copys<C: Candle + 'static>(
    symbol: &str,
    params: &CopysParams,
    candles: &[C],
) -> Result<bool> {
    if candles.len() < 60 {
        log::warn!(
            "코인 {} CopyS 필터에 필요한 캔들 데이터가 부족합니다. 필요: {} >= 60",
            symbol,
            candles.len()
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

    // 전략 컨텍스트 생성
    let ctx = match create_strategy_context_for_filter(
        symbol,
        params.rsi_period,
        &ma_type,
        &ma_periods,
        candles,
    ) {
        Ok(context) => context,
        Err(e) => {
            log::warn!("코인 {symbol} CopyS 필터 컨텍스트 생성 실패: {e}");
            return Ok(false);
        }
    };

    // 모의 전략 객체 생성 (캔들 데이터로 초기화)
    let mut filter = CopysFilter::new(ctx, params.clone(), candles);

    // 캔들 데이터로 분석기 업데이트
    for candle in candles {
        filter.next(candle.clone());
    }

    // 전략 신호 체크
    let result = match params.filter_type {
        CopysFilterType::BasicBuySignal => filter.check_buy_signal(params.consecutive_n),
        CopysFilterType::BasicSellSignal => filter.check_sell_signal(params.consecutive_n),
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
        CopysFilterType::BBandLowerTouch => {
            filter.bband_analyzer().is_below_lower_band(1, params.p)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1, params.p)
        }
        CopysFilterType::BBandUpperTouch => {
            filter.bband_analyzer().is_above_upper_band(1, params.p)
        }
        CopysFilterType::MASupport => filter.check_ma_support(),
        CopysFilterType::MAResistance => filter.check_ma_resistance(),
        CopysFilterType::StrongBuySignal => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
                params.p,
            );
            let bband_support = filter.bband_analyzer().is_below_lower_band(1, params.p)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1, params.p);
            let ma_support = filter.check_ma_support();
            rsi_oversold && bband_support && ma_support
        }
        CopysFilterType::StrongSellSignal => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
                params.p,
            );
            let bband_resistance = filter.bband_analyzer().is_above_upper_band(1, params.p);
            let ma_resistance = filter.check_ma_resistance();
            rsi_overbought && bband_resistance && ma_resistance
        }
        CopysFilterType::WeakBuySignal => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
                params.p,
            );
            let bband_support = filter.bband_analyzer().is_below_lower_band(1, params.p)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1, params.p);
            let ma_support = filter.check_ma_support();
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
            let bband_resistance = filter.bband_analyzer().is_above_upper_band(1, params.p);
            let ma_resistance = filter.check_ma_resistance();
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
            !filter.bband_analyzer().is_above_upper_band(1, params.p)
                && !filter.bband_analyzer().is_below_lower_band(1, params.p)
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
