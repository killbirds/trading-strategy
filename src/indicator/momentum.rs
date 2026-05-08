use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct MomentumInput {
    high: f64,
    low: f64,
    close: f64,
}

/// 모멘텀 복합 지표 계산 결과
#[derive(Clone, Debug)]
pub struct MomentumIndicators {
    /// RSI (Relative Strength Index)
    pub rsi: f64,
    /// 스토캐스틱 %K
    pub stoch_k: f64,
    /// 스토캐스틱 %D
    pub stoch_d: f64,
    /// 윌리엄스 %R
    pub williams_r: f64,
    /// Rate of Change (ROC)
    pub roc: f64,
    /// Commodity Channel Index (CCI)
    pub cci: f64,
    /// 단순 가격 변화 모멘텀
    pub momentum: f64,
    /// Ultimate Oscillator
    pub ultimate_oscillator: f64,
}

/// Indicator module에서 사용하는 모멘텀 결과 타입
pub type Momentum = MomentumIndicators;

impl Display for MomentumIndicators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Momentum({:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2})",
            self.rsi,
            self.stoch_k,
            self.stoch_d,
            self.williams_r,
            self.roc,
            self.cci,
            self.momentum,
            self.ultimate_oscillator
        )
    }
}

/// 모멘텀 복합 지표 빌더
#[derive(Debug)]
pub struct MomentumBuilder<C: Candle> {
    rsi_period: usize,
    stoch_period: usize,
    williams_period: usize,
    roc_period: usize,
    cci_period: usize,
    momentum_period: usize,
    values: Vec<MomentumInput>,
    stoch_k_values: Vec<f64>,
    _phantom: PhantomData<C>,
}

impl<C> MomentumBuilder<C>
where
    C: Candle,
{
    pub fn new(
        rsi_period: usize,
        stoch_period: usize,
        williams_period: usize,
        roc_period: usize,
        cci_period: usize,
        momentum_period: usize,
    ) -> Self {
        match Self::new_checked(
            rsi_period,
            stoch_period,
            williams_period,
            roc_period,
            cci_period,
            momentum_period,
        ) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn new_checked(
        rsi_period: usize,
        stoch_period: usize,
        williams_period: usize,
        roc_period: usize,
        cci_period: usize,
        momentum_period: usize,
    ) -> IndicatorResult<Self> {
        if rsi_period == 0
            || stoch_period == 0
            || williams_period == 0
            || roc_period == 0
            || cci_period == 0
            || momentum_period == 0
        {
            return Err("모멘텀 기간은 0보다 커야 합니다".to_string());
        }

        let max_period = rsi_period
            .max(stoch_period)
            .max(williams_period)
            .max(roc_period)
            .max(cci_period)
            .max(momentum_period)
            .max(28);

        Ok(Self {
            rsi_period,
            stoch_period,
            williams_period,
            roc_period,
            cci_period,
            momentum_period,
            values: Vec::with_capacity(max_period * 2),
            stoch_k_values: Vec::with_capacity(4),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Momentum {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> Momentum {
        self.values.clear();
        self.stoch_k_values.clear();

        if data.is_empty() {
            return self.empty_momentum();
        }

        let mut momentum = self.empty_momentum();
        for item in data {
            momentum = self.next(item);
        }

        momentum
    }

    pub fn next(&mut self, data: &C) -> Momentum {
        self.push(data);
        self.calculate(data)
    }

    fn push(&mut self, data: &C) {
        self.values.push(MomentumInput {
            high: data.high_price(),
            low: data.low_price(),
            close: data.close_price(),
        });

        let max_period = self
            .rsi_period
            .max(self.stoch_period)
            .max(self.williams_period)
            .max(self.roc_period)
            .max(self.cci_period)
            .max(self.momentum_period)
            .max(28);
        if self.values.len() > max_period * 2 {
            let excess = self.values.len() - max_period * 2;
            self.values.drain(0..excess);
        }
    }

    fn recent_values(&self) -> Vec<MomentumInput> {
        self.values.iter().rev().take(50).cloned().collect()
    }

    fn empty_momentum(&self) -> Momentum {
        MomentumIndicators {
            rsi: 50.0,
            stoch_k: 50.0,
            stoch_d: 50.0,
            williams_r: -50.0,
            roc: 0.0,
            cci: 0.0,
            momentum: 0.0,
            ultimate_oscillator: 50.0,
        }
    }

    fn calculate(&mut self, _data: &C) -> Momentum {
        let recent = self.recent_values();
        let rsi = calculate_rsi(&recent, self.rsi_period);
        let stoch_k = calculate_stochastic_k(&recent, self.stoch_period);

        self.stoch_k_values.insert(0, stoch_k);
        if self.stoch_k_values.len() > 3 {
            self.stoch_k_values.truncate(3);
        }
        let stoch_d = calculate_stochastic_d(&self.stoch_k_values);

        let williams_r = calculate_williams_r(&recent, self.williams_period);
        let roc = calculate_roc(&recent, self.roc_period);
        let cci = calculate_cci(&recent, self.cci_period);
        let momentum = calculate_momentum(&recent, self.momentum_period);
        let ultimate_oscillator = calculate_ultimate_oscillator(&recent);

        MomentumIndicators {
            rsi,
            stoch_k,
            stoch_d,
            williams_r,
            roc,
            cci,
            momentum,
            ultimate_oscillator,
        }
    }
}

impl<C> TABuilder<Momentum, C> for MomentumBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Momentum {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> Momentum {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> Momentum {
        self.next(data)
    }
}

fn calculate_rsi(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return 50.0;
    }

    let price_changes: Vec<f64> = candles
        .windows(2)
        .map(|w| w[0].close - w[1].close)
        .collect();
    let gains: Vec<f64> = price_changes
        .iter()
        .map(|&x| if x > 0.0 { x } else { 0.0 })
        .collect();
    let losses: Vec<f64> = price_changes
        .iter()
        .map(|&x| if x < 0.0 { -x } else { 0.0 })
        .collect();

    let avg_gain = gains.iter().sum::<f64>() / gains.len() as f64;
    let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;

    if avg_loss == 0.0 {
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn calculate_stochastic_k(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return 50.0;
    }

    let recent_candles = &candles[..period];
    let highest_high = recent_candles.iter().map(|c| c.high).fold(0.0, f64::max);
    let lowest_low = recent_candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let current_close = match candles.first() {
        Some(c) => c.close,
        None => return 50.0,
    };

    if highest_high == lowest_low {
        return 50.0;
    }

    ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
}

fn calculate_stochastic_d(k_values: &[f64]) -> f64 {
    if k_values.len() < 3 {
        return k_values.first().copied().unwrap_or(50.0);
    }

    k_values[..3].iter().sum::<f64>() / 3.0
}

fn calculate_williams_r(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return -50.0;
    }

    let recent_candles = &candles[..period];
    let highest_high = recent_candles.iter().map(|c| c.high).fold(0.0, f64::max);
    let lowest_low = recent_candles
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);
    let current_close = match candles.first() {
        Some(c) => c.close,
        None => return 50.0,
    };

    if highest_high == lowest_low {
        return -50.0;
    }

    -((highest_high - current_close) / (highest_high - lowest_low)) * 100.0
}

fn calculate_roc(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return 0.0;
    }

    let current_price = match candles.first() {
        Some(c) => c.close,
        None => return 0.0,
    };
    let past_price = match candles.get(period - 1) {
        Some(c) => c.close,
        None => return 0.0,
    };

    if past_price == 0.0 {
        return 0.0;
    }

    ((current_price - past_price) / past_price) * 100.0
}

fn calculate_cci(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return 0.0;
    }

    let recent_candles = &candles[..period];
    let typical_prices: Vec<f64> = recent_candles
        .iter()
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();
    let sma = typical_prices.iter().sum::<f64>() / typical_prices.len() as f64;
    let current_typical = match typical_prices.first() {
        Some(&tp) => tp,
        None => return 0.0,
    };
    let mad = typical_prices
        .iter()
        .map(|&tp| (tp - sma).abs())
        .sum::<f64>()
        / typical_prices.len() as f64;

    if mad == 0.0 {
        return 0.0;
    }

    (current_typical - sma) / (0.015 * mad)
}

fn calculate_momentum(candles: &[MomentumInput], period: usize) -> f64 {
    if candles.len() < period {
        return 0.0;
    }

    let current_price = match candles.first() {
        Some(c) => c.close,
        None => return 0.0,
    };
    let past_price = match candles.get(period - 1) {
        Some(c) => c.close,
        None => return 0.0,
    };

    current_price - past_price
}

fn calculate_ultimate_oscillator(candles: &[MomentumInput]) -> f64 {
    if candles.len() < 28 {
        return 50.0;
    }

    let calculate_bp_tr = |current: &MomentumInput, previous: &MomentumInput| -> (f64, f64) {
        let bp = current.close - current.low.min(previous.close);
        let tr = current.high.max(previous.close) - current.low.min(previous.close);
        (bp, tr)
    };

    let mut bp_sum_7 = 0.0;
    let mut tr_sum_7 = 0.0;
    let mut bp_sum_14 = 0.0;
    let mut tr_sum_14 = 0.0;
    let mut bp_sum_28 = 0.0;
    let mut tr_sum_28 = 0.0;

    for i in 0..28.min(candles.len() - 1) {
        let (bp, tr) = calculate_bp_tr(&candles[i], &candles[i + 1]);

        if i < 7 {
            bp_sum_7 += bp;
            tr_sum_7 += tr;
        }
        if i < 14 {
            bp_sum_14 += bp;
            tr_sum_14 += tr;
        }
        bp_sum_28 += bp;
        tr_sum_28 += tr;
    }

    let avg_7 = if tr_sum_7 != 0.0 {
        bp_sum_7 / tr_sum_7
    } else {
        0.0
    };
    let avg_14 = if tr_sum_14 != 0.0 {
        bp_sum_14 / tr_sum_14
    } else {
        0.0
    };
    let avg_28 = if tr_sum_28 != 0.0 {
        bp_sum_28 / tr_sum_28
    } else {
        0.0
    };

    ((4.0 * avg_7) + (2.0 * avg_14) + avg_28) / 7.0 * 100.0
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

    fn uptrend_candles(count: usize) -> Vec<TestCandle> {
        (0..count)
            .map(|index| {
                let close = 100.0 + index as f64;
                candle(index as i64, close + 1.0, close - 1.0, close)
            })
            .collect()
    }

    #[test]
    fn test_momentum_builder_new() {
        let builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);

        assert_eq!(builder.rsi_period, 14);
        assert_eq!(builder.momentum_period, 10);
    }

    #[test]
    #[should_panic(expected = "모멘텀 기간은 0보다 커야 합니다")]
    fn test_momentum_builder_invalid_period() {
        MomentumBuilder::<TestCandle>::new(0, 14, 14, 10, 20, 10);
    }

    #[test]
    fn test_momentum_build_empty_data() {
        let mut builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);
        let momentum = builder.build(&[]);

        assert_eq!(momentum.rsi, 50.0);
        assert_eq!(momentum.stoch_k, 50.0);
        assert_eq!(momentum.momentum, 0.0);
    }

    #[test]
    fn test_momentum_indicators_preserves_public_struct_literal_shape() {
        let indicators = MomentumIndicators {
            rsi: 50.0,
            stoch_k: 50.0,
            stoch_d: 50.0,
            williams_r: -50.0,
            roc: 0.0,
            cci: 0.0,
            momentum: 0.0,
            ultimate_oscillator: 50.0,
        };

        assert_eq!(indicators.rsi, 50.0);
        assert_eq!(indicators.ultimate_oscillator, 50.0);
    }

    #[test]
    fn test_momentum_detects_uptrend_values() {
        let candles = uptrend_candles(30);
        let mut builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);
        let momentum = builder.build(&candles);

        assert_eq!(momentum.rsi, 100.0);
        assert!(momentum.stoch_k > 90.0);
        assert!(momentum.williams_r > -10.0);
        assert!(momentum.roc > 0.0);
        assert_eq!(momentum.momentum, 9.0);
    }

    #[test]
    fn test_momentum_build_and_next_are_consistent() {
        let candles = uptrend_candles(30);
        let mut build_builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);
        let built = build_builder.build(&candles);

        let mut next_builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);
        let mut next = next_builder.build(&[]);
        for candle in &candles {
            next = next_builder.next(candle);
        }

        assert_eq!(built.rsi, next.rsi);
        assert_eq!(built.stoch_k, next.stoch_k);
        assert_eq!(built.stoch_d, next.stoch_d);
        assert_eq!(built.momentum, next.momentum);
    }

    #[test]
    fn test_momentum_display() {
        let candles = uptrend_candles(30);
        let mut builder = MomentumBuilder::<TestCandle>::new(14, 14, 14, 10, 20, 10);
        let momentum = builder.build(&candles);

        assert!(momentum.to_string().starts_with("Momentum("));
    }
}
