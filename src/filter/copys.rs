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

use super::CopysParams;

/// CopyS 모의 전략 (필터 사용을 위한 임시 객체)
struct CopysFilter<C: Candle> {
    ctx: CopysStrategyContext<C>,
    bband_analyzer: BBandAnalyzer<C>,
    params: CopysParams,
}

impl<C: Candle + 'static> CopysFilter<C> {
    fn new(ctx: CopysStrategyContext<C>, params: CopysParams) -> Self {
        // 임시 저장소로 볼린저밴드 분석기 생성
        let storage = CandleStore::<C>::new(Vec::new(), 1000, false);
        let bband_analyzer = BBandAnalyzer::new(20, 2.0, &storage);

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
        20 // 기본값
    }

    fn config_bband_multiplier(&self) -> f64 {
        2.0 // 기본값
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

    // MAType 설정 - 이미지 참고로 수정된 이평선 기간 적용
    let ma_type = MAType::EMA;
    let ma_periods = vec![5, 20, 60, 120, 200, 240]; // 이미지 참고: 수정된 MA 주기

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

    // 모의 전략 객체 생성
    let mut filter = CopysFilter::new(ctx, params.clone());

    // 캔들 데이터로 분석기 업데이트
    for candle in candles {
        filter.next(candle.clone());
    }

    // 전략 신호 체크
    let result = match params.filter_type {
        // 0: 기본 매수 신호 (RSI 과매도 + 볼린저밴드 하단 + 이평선 지지 중 2개 이상)
        0 => filter.check_buy_signal(params.consecutive_n),
        // 1: 기본 매도 신호 (RSI 과매수 + 볼린저밴드 상단 + 이평선 저항 중 2개 이상)
        1 => filter.check_sell_signal(params.consecutive_n),
        // 2: RSI 과매도 조건만 확인
        2 => filter.context().is_all(
            |data| data.rsi.value() < filter.config_rsi_lower(),
            params.consecutive_n,
        ),
        // 3: RSI 과매수 조건만 확인
        3 => filter.context().is_all(
            |data| data.rsi.value() > filter.config_rsi_upper(),
            params.consecutive_n,
        ),
        // 4: 볼린저밴드 하단 터치 조건만 확인
        4 => {
            filter.bband_analyzer().is_below_lower_band(1)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1)
        }
        // 5: 볼린저밴드 상단 터치 조건만 확인
        5 => filter.bband_analyzer().is_above_upper_band(1),
        // 6: 이평선 지지선 조건만 확인
        6 => filter.check_ma_support(),
        // 7: 이평선 저항선 조건만 확인
        7 => filter.check_ma_resistance(),
        // 8: 강한 매수 신호 (3개 조건 모두 만족)
        8 => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
            );
            let bband_support = filter.bband_analyzer().is_below_lower_band(1)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1);
            let ma_support = filter.check_ma_support();

            rsi_oversold && bband_support && ma_support
        }
        // 9: 강한 매도 신호 (3개 조건 모두 만족)
        9 => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
            );
            let bband_resistance = filter.bband_analyzer().is_above_upper_band(1);
            let ma_resistance = filter.check_ma_resistance();

            rsi_overbought && bband_resistance && ma_resistance
        }
        // 10: 약한 매수 신호 (1개 조건만 만족)
        10 => {
            let rsi_oversold = filter.context().is_all(
                |data| data.rsi.value() < filter.config_rsi_lower(),
                params.consecutive_n,
            );
            let bband_support = filter.bband_analyzer().is_below_lower_band(1)
                || filter
                    .bband_analyzer()
                    .is_break_through_lower_band_from_below(1);
            let ma_support = filter.check_ma_support();

            let signal_count = [rsi_oversold, bband_support, ma_support]
                .iter()
                .filter(|&&x| x)
                .count();
            signal_count == 1
        }
        // 11: 약한 매도 신호 (1개 조건만 만족)
        11 => {
            let rsi_overbought = filter.context().is_all(
                |data| data.rsi.value() > filter.config_rsi_upper(),
                params.consecutive_n,
            );
            let bband_resistance = filter.bband_analyzer().is_above_upper_band(1);
            let ma_resistance = filter.check_ma_resistance();

            let signal_count = [rsi_overbought, bband_resistance, ma_resistance]
                .iter()
                .filter(|&&x| x)
                .count();
            signal_count == 1
        }
        // 12: RSI 중립대 (과매수/과매도 아님)
        12 => filter.context().is_all(
            |data| {
                let rsi = data.rsi.value();
                rsi >= filter.config_rsi_lower() && rsi <= filter.config_rsi_upper()
            },
            params.consecutive_n,
        ),
        // 13: 볼린저밴드 내부 (상단과 하단 사이)
        13 => {
            !filter.bband_analyzer().is_above_upper_band(1)
                && !filter.bband_analyzer().is_below_lower_band(1)
        }
        // 14: 이평선 정배열 상태 (단기 > 장기)
        14 => {
            if filter.context().items.is_empty() {
                false
            } else {
                let mas = &filter.context().items[0].mas;
                if mas.len() < 2 {
                    false
                } else {
                    let short_ma = mas.get_by_key_index(0).get(); // 5일선
                    let long_ma = mas.get_by_key_index(mas.len() - 1).get(); // 240일선
                    short_ma > long_ma
                }
            }
        }
        // 15: 이평선 역배열 상태 (단기 < 장기)
        15 => {
            if filter.context().items.is_empty() {
                false
            } else {
                let mas = &filter.context().items[0].mas;
                if mas.len() < 2 {
                    false
                } else {
                    let short_ma = mas.get_by_key_index(0).get(); // 5일선
                    let long_ma = mas.get_by_key_index(mas.len() - 1).get(); // 240일선
                    short_ma < long_ma
                }
            }
        }
        _ => false,
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
        assert_eq!(params.filter_type, 0);
        assert_eq!(params.consecutive_n, 1);
    }
}
