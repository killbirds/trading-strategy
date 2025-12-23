use super::{MACDFilterType, MACDParams, utils};
use crate::analyzer::AnalyzerOps;
use crate::analyzer::macd_analyzer::MACDAnalyzer;
use anyhow::Result;
use trading_chart::Candle;

/// 개별 코인에 대한 MACD 필터 적용
pub fn filter_macd<C: Candle + 'static>(
    coin: &str,
    params: &MACDParams,
    candles: &[C],
) -> Result<bool> {
    log::debug!(
        "MACD 필터 적용 - 빠른 기간: {}, 느린 기간: {}, 시그널 기간: {}, 타입: {:?}, 연속성: {}",
        params.fast_period,
        params.slow_period,
        params.signal_period,
        params.filter_type,
        params.consecutive_n
    );

    // 파라미터 검증
    utils::validate_period(params.fast_period, "MACD fast_period")?;
    utils::validate_period(params.slow_period, "MACD slow_period")?;
    utils::validate_period(params.signal_period, "MACD signal_period")?;

    // 필터링 로직
    let required_length = params.slow_period + params.signal_period + params.consecutive_n; // 최소 필요 캔들 수
    if !utils::check_sufficient_candles(candles.len(), required_length, coin) {
        return Ok(false);
    }

    // 캔들 데이터로 CandleStore 생성
    let candle_store = utils::create_candle_store(candles);

    // MACDAnalyzer 생성
    let analyzer = MACDAnalyzer::new(
        params.fast_period,
        params.slow_period,
        params.signal_period,
        &candle_store,
    );

    log::debug!("코인 {coin} MACD 분석기 생성 완료");

    let result = match params.filter_type {
        MACDFilterType::MacdAboveSignal => analyzer.is_all(
            |data| data.is_macd_above_signal(),
            params.consecutive_n,
            params.p,
        ),
        MACDFilterType::MacdBelowSignal => analyzer.is_all(
            |data| data.is_macd_below_signal(),
            params.consecutive_n,
            params.p,
        ),
        MACDFilterType::SignalCrossAbove => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                let current_macd_above_signal = current.macd.macd_line > current.macd.signal_line;
                let previous_macd_below_signal =
                    previous.macd.macd_line < previous.macd.signal_line;
                current_macd_above_signal && previous_macd_below_signal
            }
        }
        MACDFilterType::SignalCrossBelow => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current = &analyzer.items[params.p];
                let previous = &analyzer.items[params.p + 1];
                let current_macd_below_signal = current.macd.macd_line < current.macd.signal_line;
                let previous_macd_above_signal =
                    previous.macd.macd_line > previous.macd.signal_line;
                current_macd_below_signal && previous_macd_above_signal
            }
        }
        MACDFilterType::HistogramAboveThreshold => {
            analyzer.is_histogram_above_threshold(params.threshold, params.consecutive_n, params.p)
        }
        MACDFilterType::HistogramBelowThreshold => {
            analyzer.is_histogram_below_threshold(params.threshold, params.consecutive_n, params.p)
        }
        MACDFilterType::ZeroLineCrossAbove => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let previous_macd = analyzer.items[params.p + 1].macd.macd_line;
                current_macd > 0.0 && previous_macd <= 0.0
            }
        }
        MACDFilterType::ZeroLineCrossBelow => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let previous_macd = analyzer.items[params.p + 1].macd.macd_line;
                current_macd < 0.0 && previous_macd >= 0.0
            }
        }
        MACDFilterType::HistogramNegativeTurn => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_hist = analyzer.items[params.p].macd.histogram;
                let previous_hist = analyzer.items[params.p + 1].macd.histogram;
                current_hist < 0.0 && previous_hist >= 0.0
            }
        }
        MACDFilterType::HistogramPositiveTurn => {
            if analyzer.items.len() < params.p + 2 {
                false
            } else {
                let current_hist = analyzer.items[params.p].macd.histogram;
                let previous_hist = analyzer.items[params.p + 1].macd.histogram;
                current_hist > 0.0 && previous_hist <= 0.0
            }
        }
        MACDFilterType::StrongUptrend => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_data = &analyzer.items[params.p];
                current_data.macd.macd_line > 0.0 && current_data.macd.signal_line > 0.0
            }
        }
        MACDFilterType::StrongDowntrend => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_data = &analyzer.items[params.p];
                current_data.macd.macd_line < 0.0 && current_data.macd.signal_line < 0.0
            }
        }
        MACDFilterType::MacdRising => {
            // Note: Uses strict comparison (<=) to ensure each consecutive value is strictly greater
            // This means MACD must be strictly increasing, not just non-decreasing
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut ascending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[params.p + i].macd.macd_line
                        <= analyzer.items[params.p + i + 1].macd.macd_line
                    {
                        ascending = false;
                        break;
                    }
                }
                ascending
            }
        }
        MACDFilterType::MacdFalling => {
            // Note: Uses strict comparison (>=) to ensure each consecutive value is strictly less
            // This means MACD must be strictly decreasing, not just non-increasing
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut descending = true;
                for i in 0..params.consecutive_n {
                    if analyzer.items[params.p + i].macd.macd_line
                        >= analyzer.items[params.p + i + 1].macd.macd_line
                    {
                        descending = false;
                        break;
                    }
                }
                descending
            }
        }
        MACDFilterType::HistogramExpanding => {
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut expanding = true;
                for i in 0..params.consecutive_n {
                    let current_hist = analyzer.items[params.p + i].macd.histogram.abs();
                    let previous_hist = analyzer.items[params.p + i + 1].macd.histogram.abs();
                    if current_hist <= previous_hist {
                        expanding = false;
                        break;
                    }
                }
                expanding
            }
        }
        MACDFilterType::HistogramContracting => {
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut contracting = true;
                for i in 0..params.consecutive_n {
                    let current_hist = analyzer.items[params.p + i].macd.histogram.abs();
                    let previous_hist = analyzer.items[params.p + i + 1].macd.histogram.abs();
                    if current_hist >= previous_hist {
                        contracting = false;
                        break;
                    }
                }
                contracting
            }
        }
        MACDFilterType::Divergence => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_price = analyzer.items[params.p].candle.close_price();
                let previous_price = analyzer.items[params.p + 2].candle.close_price();
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let previous_macd = analyzer.items[params.p + 2].macd.macd_line;
                current_price > previous_price && current_macd < previous_macd
            }
        }
        MACDFilterType::Convergence => {
            if analyzer.items.len() < params.p + 3 {
                false
            } else {
                let current_price = analyzer.items[params.p].candle.close_price();
                let previous_price = analyzer.items[params.p + 2].candle.close_price();
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let previous_macd = analyzer.items[params.p + 2].macd.macd_line;
                current_price < previous_price && current_macd > previous_macd
            }
        }
        MACDFilterType::Overbought => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let avg_price = analyzer.items[params.p].candle.close_price();
                if avg_price == 0.0 {
                    false
                } else {
                    let macd_ratio = current_macd / avg_price;
                    macd_ratio >= params.overbought_threshold
                }
            }
        }
        MACDFilterType::Oversold => {
            if analyzer.items.len() <= params.p {
                false
            } else {
                let current_macd = analyzer.items[params.p].macd.macd_line;
                let avg_price = analyzer.items[params.p].candle.close_price();
                if avg_price == 0.0 {
                    false
                } else {
                    let macd_ratio = current_macd / avg_price;
                    macd_ratio <= -params.oversold_threshold
                }
            }
        }
        MACDFilterType::Sideways => {
            if analyzer.items.len() < params.p + params.consecutive_n + 1 {
                false
            } else {
                let mut sideways = true;
                for i in 0..params.consecutive_n {
                    let current_macd = analyzer.items[params.p + i].macd.macd_line;
                    let previous_macd = analyzer.items[params.p + i + 1].macd.macd_line;
                    let prev_abs = previous_macd.abs();
                    if prev_abs == 0.0 {
                        sideways = false;
                        break;
                    }
                    let change_rate = (current_macd - previous_macd).abs() / prev_abs;
                    if change_rate > params.sideways_threshold {
                        sideways = false;
                        break;
                    }
                }
                sideways
            }
        }
    };

    Ok(result)
}
