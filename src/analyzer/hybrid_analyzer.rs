use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::macd::{MACD, MACDBuilder};
use crate::indicator::rsi::{RSI, RSIBuilder};
use std::fmt::Display;
use trading_chart::Candle;

// RSI threshold constants
const RSI_OVERSOLD_THRESHOLD: f64 = 30.0;
const RSI_OVERBOUGHT_THRESHOLD: f64 = 70.0;
const RSI_NEUTRAL_LOWER: f64 = 30.0;
const RSI_NEUTRAL_UPPER: f64 = 70.0;

// Signal strength constants
const SIGNAL_STRENGTH_WEAK: f64 = 0.5;
const SIGNAL_STRENGTH_MODERATE: f64 = 0.6;
const SIGNAL_STRENGTH_STRONG: f64 = 0.7;
const SIGNAL_STRENGTH_HALF: f64 = 0.5;

// Volume factor adjustment threshold
const VOLUME_FACTOR_ADJUSTMENT_THRESHOLD: f64 = 1.5;

// Score range constants
const SCORE_RANGE_MIN: f64 = 0.5;

/// 하이브리드 분석기 데이터
#[derive(Debug)]
pub struct HybridAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 이동평균 데이터
    pub ma: Box<dyn MA>,
    /// MACD 데이터
    pub macd: MACD,
    /// RSI 데이터
    pub rsi: RSI,
}

impl<C: Candle + Clone> HybridAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, ma: Box<dyn MA>, macd: MACD, rsi: RSI) -> HybridAnalyzerData<C> {
        HybridAnalyzerData {
            candle,
            ma,
            macd,
            rsi,
        }
    }

    /// 저장된 값으로 데이터 복제
    pub fn clone_with_stored_values(&self) -> HybridAnalyzerData<C> {
        // Box<dyn MA>는 클론할 수 없으므로, MA 구현체의 값을 저장하고 새 객체 생성
        let ma_period = self.ma.period();
        let ma_value = self.ma.get();

        // 값을 가지고 있는 간단한 MA 구현체
        struct SimpleMA {
            period: usize,
            value: f64,
        }

        impl MA for SimpleMA {
            fn period(&self) -> usize {
                self.period
            }

            fn get(&self) -> f64 {
                self.value
            }
        }

        impl Display for SimpleMA {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MA({}: {:.2})", self.period, self.value)
            }
        }

        impl std::fmt::Debug for SimpleMA {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "SimpleMA({}: {:.2})", self.period, self.value)
            }
        }

        let simple_ma = SimpleMA {
            period: ma_period,
            value: ma_value,
        };

        HybridAnalyzerData {
            candle: self.candle.clone(),
            ma: Box::new(simple_ma),
            macd: self.macd.clone(),
            rsi: self.rsi.clone(),
        }
    }
}

impl<C: Candle> GetCandle<C> for HybridAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for HybridAnalyzerData<C> {}

/// 하이브리드 분석기 컨텍스트
#[derive(Debug)]
pub struct HybridAnalyzer<C: Candle + Clone> {
    /// 이동평균 빌더
    pub mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    /// MACD 빌더
    pub macdbuilder: MACDBuilder<C>,
    /// RSI 빌더
    pub rsibuilder: RSIBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<HybridAnalyzerData<C>>,
}

impl<C: Candle + Clone> Display for HybridAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "candle: {}, ma: {:.2}, macd: {}, rsi: {:.2}",
                first.candle,
                first.ma.get(),
                first.macd,
                first.rsi.value()
            )
        } else {
            write!(f, "데이터 없음")
        }
    }
}

impl<C: Candle + Clone + 'static> HybridAnalyzer<C> {
    /// 새 분석기 컨텍스트 생성
    pub fn new(
        ma_type: &MAType,
        ma_period: usize,
        macd_fast_period: usize,
        macd_slow_period: usize,
        macd_signal_period: usize,
        rsi_period: usize,
        storage: &CandleStore<C>,
    ) -> HybridAnalyzer<C> {
        let mabuilder = MABuilderFactory::build(ma_type, ma_period);
        let macdbuilder = MACDBuilder::new(macd_fast_period, macd_slow_period, macd_signal_period);
        let rsibuilder = RSIBuilder::new(rsi_period);

        let mut ctx = HybridAnalyzer {
            mabuilder,
            macdbuilder,
            rsibuilder,
            items: vec![],
        };

        ctx.init_from_storage(storage);
        ctx
    }

    /// 매수 신호 강도 계산
    ///
    /// # Arguments
    /// * `rsi_lower` - RSI 과매도 기준값 (예: 30)
    ///
    /// # Returns
    /// * `f64` - 0.0(신호 없음)에서 1.0(강한 신호) 사이의 매수 신호 강도
    pub fn calculate_buy_signal_strength(&self, rsi_lower: f64) -> f64 {
        if self.items.len() < 3 {
            return 0.0;
        }

        let current = match self.items.first() {
            Some(item) => item,
            None => return 0.0,
        };
        let previous = match self.items.get(1) {
            Some(item) => item,
            None => return 0.0,
        };
        let before_previous = match self.items.get(2) {
            Some(item) => item,
            None => return 0.0,
        };

        // 가중치 정의
        const MA_WEIGHT: f64 = 0.25; // 이동평균 기준 신호 가중치
        const PRICE_MOMENTUM_WEIGHT: f64 = 0.1; // 가격 모멘텀 가중치 
        const MACD_CROSS_WEIGHT: f64 = 0.3; // MACD 골든크로스 가중치
        const MACD_HIST_WEIGHT: f64 = 0.15; // MACD 히스토그램 가중치
        const RSI_WEIGHT: f64 = 0.2; // RSI 가중치

        let mut signal_strength = 0.0;

        // 1. 이동평균선 기반 신호 (가격이 이동평균선 위에 있는지, 상승추세인지)
        if current.candle.close_price() > current.ma.get() {
            // 가격이 이동평균 위에 있음 (상승추세 가능성)
            signal_strength += MA_WEIGHT * SIGNAL_STRENGTH_MODERATE;

            // 이동평균선 자체가 상승 중인지 확인
            if current.ma.get() > previous.ma.get() {
                signal_strength += MA_WEIGHT * 0.4;
            }
        }

        // 2. 가격 모멘텀 확인 (최근 캔들들의 연속적인 상승)
        if current.candle.close_price() > previous.candle.close_price()
            && previous.candle.close_price() > before_previous.candle.close_price()
        {
            signal_strength += PRICE_MOMENTUM_WEIGHT;
        }

        // 3. MACD 기반 신호
        if current.macd.macd_line > current.macd.signal_line
            && previous.macd.macd_line <= previous.macd.signal_line
        {
            // 골든 크로스 (강한 매수 신호)
            signal_strength += MACD_CROSS_WEIGHT;
        }

        // MACD 히스토그램 분석
        if current.macd.histogram > 0.0 {
            // 히스토그램이 양수 (상승 추세)
            let histogram_factor =
                (current.macd.histogram / current.candle.close_price().abs()).min(0.05) * 20.0;
            signal_strength += MACD_HIST_WEIGHT * histogram_factor.min(1.0);

            // 히스토그램이 증가 중인지 확인 (모멘텀 가속)
            if current.macd.histogram > previous.macd.histogram {
                signal_strength += MACD_HIST_WEIGHT * SIGNAL_STRENGTH_HALF;
            }
        }

        // 4. RSI 기반 신호
        let rsi = current.rsi.value();

        if rsi < rsi_lower {
            // 과매도 상태 (강한 매수 신호)
            signal_strength += RSI_WEIGHT * (1.0 - rsi / rsi_lower);
        } else if rsi < 45.0 && rsi > previous.rsi.value() {
            // RSI가 낮은 상태에서 반등 중 (적절한 매수 신호)
            signal_strength += RSI_WEIGHT * SIGNAL_STRENGTH_HALF * (45.0 - rsi) / 15.0;
        }

        // 최종 신호 강도 (0.0~1.0 범위로 클램핑)
        signal_strength.clamp(0.0, 1.0)
    }

    /// 매도 신호 강도 계산
    ///
    /// # Arguments
    /// * `rsi_upper` - RSI 과매수 기준값 (예: 70)
    /// * `profit_percentage` - 현재 포지션의 수익률 (%)
    ///
    /// # Returns
    /// * `f64` - 0.0(신호 없음)에서 1.0(강한 신호) 사이의 매도 신호 강도
    pub fn calculate_sell_signal_strength(&self, rsi_upper: f64, profit_percentage: f64) -> f64 {
        if self.items.len() < 3 {
            return 0.0;
        }

        let current = match self.items.first() {
            Some(item) => item,
            None => return 0.0,
        };
        let previous = match self.items.get(1) {
            Some(item) => item,
            None => return 0.0,
        };
        let before_previous = match self.items.get(2) {
            Some(item) => item,
            None => return 0.0,
        };

        // 가중치 정의
        const MA_WEIGHT: f64 = 0.2; // 이동평균 기준 신호 가중치
        const PRICE_MOMENTUM_WEIGHT: f64 = 0.1; // 가격 모멘텀 가중치
        const MACD_CROSS_WEIGHT: f64 = 0.25; // MACD 데드크로스 가중치
        const MACD_HIST_WEIGHT: f64 = 0.15; // MACD 히스토그램 가중치
        const RSI_WEIGHT: f64 = 0.2; // RSI 가중치
        const PROFIT_WEIGHT: f64 = 0.1; // 수익률 기반 가중치

        let mut signal_strength = 0.0;

        // 1. 이동평균선 기반 신호 (가격이 이동평균선 아래에 있는지, 하락추세인지)
        if current.candle.close_price() < current.ma.get() {
            // 가격이 이동평균 아래에 있음 (하락추세 가능성)
            signal_strength += MA_WEIGHT * SIGNAL_STRENGTH_MODERATE;

            // 이동평균선 자체가 하락 중인지 확인
            if current.ma.get() < previous.ma.get() {
                signal_strength += MA_WEIGHT * 0.4;
            }
        }

        // 2. 가격 모멘텀 확인 (최근 캔들들의 연속적인 하락)
        if current.candle.close_price() < previous.candle.close_price()
            && previous.candle.close_price() < before_previous.candle.close_price()
        {
            signal_strength += PRICE_MOMENTUM_WEIGHT;
        }

        // 3. MACD 기반 신호
        if current.macd.macd_line < current.macd.signal_line
            && previous.macd.macd_line >= previous.macd.signal_line
        {
            // 데드 크로스 (강한 매도 신호)
            signal_strength += MACD_CROSS_WEIGHT;
        }

        // MACD 히스토그램 분석
        if current.macd.histogram < 0.0 {
            // 히스토그램이 음수 (하락 추세)
            let histogram_factor =
                (current.macd.histogram.abs() / current.candle.close_price().abs()).min(0.05)
                    * 20.0;
            signal_strength += MACD_HIST_WEIGHT * histogram_factor.min(1.0);

            // 히스토그램이 감소 중인지 확인 (모멘텀 가속)
            if current.macd.histogram < previous.macd.histogram {
                signal_strength += MACD_HIST_WEIGHT * SIGNAL_STRENGTH_HALF;
            }
        }

        // 4. RSI 기반 신호
        let rsi = current.rsi.value();

        if rsi > rsi_upper {
            // 과매수 상태 (강한 매도 신호)
            signal_strength += RSI_WEIGHT * ((rsi - rsi_upper) / (100.0 - rsi_upper));
        } else if rsi > 55.0 && rsi < previous.rsi.value() {
            // RSI가 높은 상태에서 하락 중 (적절한 매도 신호)
            signal_strength += RSI_WEIGHT * SIGNAL_STRENGTH_HALF * (rsi - 55.0) / 15.0;
        }

        // 5. 수익률 기반 신호
        if profit_percentage > 7.0 {
            // 높은 수익 실현 (강한 매도 신호)
            signal_strength += PROFIT_WEIGHT;
        } else if profit_percentage > 3.0 {
            // 적정 수익 실현 (중간 매도 신호)
            signal_strength += PROFIT_WEIGHT * SIGNAL_STRENGTH_STRONG;
        } else if profit_percentage < -5.0 {
            // 큰 손실 발생 (손절 매도 신호)
            signal_strength += PROFIT_WEIGHT * 0.8;
        }

        // 최종 신호 강도 (0.0~1.0 범위로 클램핑)
        signal_strength.clamp(0.0, 1.0)
    }

    /// 향상된 매수 신호 강도 계산 (변동성 및 시장 상황 고려)
    ///
    /// # Arguments
    /// * `rsi_lower` - RSI 과매도 기준값
    /// * `volatility` - 현재 변동성 (ATR 기반)
    /// * `volume_factor` - 볼륨 증가 배수
    ///
    /// # Returns
    /// * `f64` - 향상된 매수 신호 강도 (0.0~1.0)
    pub fn calculate_enhanced_buy_signal_strength(
        &self,
        rsi_lower: f64,
        volatility: f64,
        volume_factor: f64,
    ) -> f64 {
        if self.items.len() < 5 {
            return 0.0;
        }

        let base_strength = self.calculate_buy_signal_strength(rsi_lower);

        // 시장 상황별 가중치 조정
        let market_condition_factor = self.calculate_market_condition_factor();
        let momentum_factor = self.calculate_momentum_factor();
        let volatility_factor = self.calculate_volatility_adjustment_factor(volatility);
        let volume_factor_adj = if volume_factor > VOLUME_FACTOR_ADJUSTMENT_THRESHOLD {
            1.2
        } else if volume_factor < 0.8 {
            0.8
        } else {
            1.0
        };
        let consensus_factor = self.calculate_indicators_consensus_factor();

        // 최종 조정된 신호 강도
        let enhanced_strength = base_strength
            * market_condition_factor
            * momentum_factor
            * volatility_factor
            * volume_factor_adj
            * consensus_factor;

        enhanced_strength.clamp(0.0, 1.0)
    }

    /// 향상된 매도 신호 강도 계산 (변동성 및 시장 상황 고려)
    ///
    /// # Arguments
    /// * `rsi_upper` - RSI 과매수 기준값
    /// * `profit_percentage` - 현재 수익률
    /// * `volatility` - 현재 변동성
    /// * `volume_factor` - 볼륨 증가 배수
    ///
    /// # Returns
    /// * `f64` - 향상된 매도 신호 강도 (0.0~1.0)
    pub fn calculate_enhanced_sell_signal_strength(
        &self,
        rsi_upper: f64,
        profit_percentage: f64,
        volatility: f64,
        volume_factor: f64,
    ) -> f64 {
        if self.items.len() < 5 {
            return 0.0;
        }

        let base_strength = self.calculate_sell_signal_strength(rsi_upper, profit_percentage);

        // 시장 상황별 가중치 조정
        let market_condition_factor = 2.0 - self.calculate_market_condition_factor(); // 역방향 적용
        let momentum_factor = 2.0 - self.calculate_momentum_factor(); // 역방향 적용
        let volatility_factor = self.calculate_volatility_adjustment_factor(volatility);
        let volume_factor_adj = if volume_factor > VOLUME_FACTOR_ADJUSTMENT_THRESHOLD {
            1.2
        } else if volume_factor < 0.8 {
            0.8
        } else {
            1.0
        };
        let consensus_factor = 2.0 - self.calculate_indicators_consensus_factor(); // 역방향 적용

        // 최종 조정된 신호 강도
        let enhanced_strength = base_strength
            * market_condition_factor
            * momentum_factor
            * volatility_factor
            * volume_factor_adj
            * consensus_factor;

        enhanced_strength.clamp(0.0, 1.0)
    }

    /// 시장 상황 요인 계산
    fn calculate_market_condition_factor(&self) -> f64 {
        if self.items.len() < 10 {
            return 1.0;
        }

        let recent_items = &self.items[..10];

        // 트렌드 강도 계산
        let price_trend = self.calculate_price_trend_strength(recent_items);
        let ma_trend = self.calculate_ma_trend_strength(recent_items);

        // 시장 안정성 계산
        let stability = self.calculate_market_stability(recent_items);

        // 종합 시장 상황 점수
        let market_score = (price_trend + ma_trend + stability) / 3.0;

        // SCORE_RANGE_MIN ~ SCORE_RANGE_MAX 범위로 조정
        SCORE_RANGE_MIN + market_score
    }

    /// 가격 트렌드 강도 계산
    fn calculate_price_trend_strength(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 5 {
            return 0.0;
        }

        let mut trend_score = 0.0;
        for i in 1..items.len() {
            let current_price = items[i - 1].candle.close_price();
            let previous_price = items[i].candle.close_price();

            if current_price > previous_price {
                trend_score += 1.0;
            } else if current_price < previous_price {
                trend_score -= 1.0;
            }
        }

        (trend_score / (items.len() - 1) as f64).abs()
    }

    /// 이동평균 트렌드 강도 계산
    fn calculate_ma_trend_strength(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 3 {
            return 0.0;
        }

        let current_ma = match items.first() {
            Some(item) => item.ma.get(),
            None => return 0.0,
        };
        let previous_ma = match items.get(1) {
            Some(item) => item.ma.get(),
            None => return 0.0,
        };
        let before_ma = match items.get(2) {
            Some(item) => item.ma.get(),
            None => return 0.0,
        };

        if (current_ma > previous_ma && previous_ma > before_ma)
            || (current_ma < previous_ma && previous_ma < before_ma)
        {
            1.0 // 강한 상승/하락 트렌드
        } else if current_ma > previous_ma || previous_ma > before_ma {
            SIGNAL_STRENGTH_WEAK // 약한 상승 트렌드
        } else {
            0.0 // 횡보
        }
    }

    /// 시장 안정성 계산
    fn calculate_market_stability(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 5 {
            return 0.0;
        }

        // RSI 변동성 계산
        let rsi_values: Vec<f64> = items.iter().map(|item| item.rsi.value()).collect();
        let rsi_volatility = self.calculate_values_volatility(&rsi_values);

        // MACD 변동성 계산
        let macd_values: Vec<f64> = items.iter().map(|item| item.macd.histogram).collect();
        let macd_volatility = self.calculate_values_volatility(&macd_values);

        // 안정성 점수 (변동성이 낮을수록 높은 점수)
        1.0 - (rsi_volatility + macd_volatility) / 2.0
    }

    /// 값들의 변동성 계산
    fn calculate_values_volatility(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt() / mean.abs().max(1.0) // 정규화된 변동성
    }

    /// 모멘텀 요인 계산
    fn calculate_momentum_factor(&self) -> f64 {
        if self.items.len() < 5 {
            return 1.0;
        }

        let recent_items = &self.items[..5];

        // 가격 모멘텀
        let price_momentum = self.calculate_price_momentum(recent_items);

        // MACD 모멘텀
        let macd_momentum = self.calculate_macd_momentum(recent_items);

        // RSI 모멘텀
        let rsi_momentum = self.calculate_rsi_momentum(recent_items);

        // 종합 모멘텀 점수
        let momentum_score = (price_momentum + macd_momentum + rsi_momentum) / 3.0;

        // SCORE_RANGE_MIN ~ SCORE_RANGE_MAX 범위로 조정
        SCORE_RANGE_MIN + momentum_score
    }

    /// 가격 모멘텀 계산
    fn calculate_price_momentum(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 3 {
            return 0.0;
        }

        let current_price = match items.first() {
            Some(item) => item.candle.close_price(),
            None => return 0.0,
        };
        let previous_price = match items.get(1) {
            Some(item) => item.candle.close_price(),
            None => return 0.0,
        };
        let before_price = match items.get(2) {
            Some(item) => item.candle.close_price(),
            None => return 0.0,
        };

        if previous_price == 0.0 || before_price == 0.0 {
            return 0.0;
        }

        let recent_change = (current_price - previous_price) / previous_price;
        let previous_change = (previous_price - before_price) / before_price;

        if recent_change > previous_change {
            (recent_change - previous_change).abs().min(1.0)
        } else {
            0.0
        }
    }

    /// MACD 모멘텀 계산
    fn calculate_macd_momentum(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 3 {
            return 0.0;
        }

        let current_hist = match items.first() {
            Some(item) => item.macd.histogram,
            None => return 0.0,
        };
        let previous_hist = match items.get(1) {
            Some(item) => item.macd.histogram,
            None => return 0.0,
        };
        let before_hist = match items.get(2) {
            Some(item) => item.macd.histogram,
            None => return 0.0,
        };

        if current_hist > previous_hist && previous_hist > before_hist {
            1.0 // 강한 모멘텀
        } else if current_hist > previous_hist {
            SIGNAL_STRENGTH_WEAK // 약한 모멘텀
        } else {
            0.0 // 모멘텀 없음
        }
    }

    /// RSI 모멘텀 계산
    fn calculate_rsi_momentum(&self, items: &[HybridAnalyzerData<C>]) -> f64 {
        if items.len() < 3 {
            return 0.0;
        }

        let current_rsi = match items.first() {
            Some(item) => item.rsi.value(),
            None => return 0.0,
        };
        let previous_rsi = match items.get(1) {
            Some(item) => item.rsi.value(),
            None => return 0.0,
        };

        if current_rsi > previous_rsi
            && current_rsi < RSI_NEUTRAL_UPPER
            && current_rsi > RSI_NEUTRAL_LOWER
        {
            ((current_rsi - previous_rsi) / 10.0).min(1.0)
        } else {
            0.0
        }
    }

    /// 변동성 조정 요인 계산
    fn calculate_volatility_adjustment_factor(&self, volatility: f64) -> f64 {
        // 높은 변동성일 때는 신호 강도를 낮추고, 낮은 변동성일 때는 높임
        if volatility > 0.05 {
            SIGNAL_STRENGTH_STRONG // 높은 변동성
        } else if volatility > 0.03 {
            0.85 // 중간 변동성
        } else if volatility > 0.01 {
            1.0 // 정상 변동성
        } else {
            1.2 // 낮은 변동성
        }
    }

    /// 지표 합의 요인 계산
    fn calculate_indicators_consensus_factor(&self) -> f64 {
        let current = match self.items.first() {
            Some(item) => item,
            None => return 1.0,
        };
        let mut consensus_score = 0.0;
        let mut total_indicators = 0.0;

        // MA 신호
        if current.candle.close_price() > current.ma.get() {
            consensus_score += 1.0;
        }
        total_indicators += 1.0;

        // MACD 신호
        if current.macd.macd_line > current.macd.signal_line {
            consensus_score += 1.0;
        }
        total_indicators += 1.0;

        // RSI 신호 (중립 영역에서 상승)
        let rsi = current.rsi.value();
        if let Some(previous) = self.items.get(1)
            && rsi > RSI_NEUTRAL_LOWER
            && rsi < RSI_NEUTRAL_UPPER
            && rsi > previous.rsi.value()
        {
            consensus_score += 1.0;
        }
        total_indicators += 1.0;

        // 합의 점수 (SCORE_RANGE_MIN ~ SCORE_RANGE_MAX 범위)
        SCORE_RANGE_MIN + (consensus_score / total_indicators)
    }

    /// 시장 상황 평가
    pub fn evaluate_market_condition(&self) -> String {
        if self.items.len() < 10 {
            return "데이터 부족".to_string();
        }

        let market_factor = self.calculate_market_condition_factor();
        let momentum_factor = self.calculate_momentum_factor();
        let consensus_factor = self.calculate_indicators_consensus_factor();

        let overall_score = (market_factor + momentum_factor + consensus_factor) / 3.0;

        if overall_score > 1.3 {
            "매우 좋은 시장 상황".to_string()
        } else if overall_score > 1.1 {
            "좋은 시장 상황".to_string()
        } else if overall_score > 0.9 {
            "보통 시장 상황".to_string()
        } else if overall_score > SIGNAL_STRENGTH_STRONG {
            "주의 필요한 시장 상황".to_string()
        } else {
            "위험한 시장 상황".to_string()
        }
    }

    /// 리스크 조정된 신호 강도 계산
    pub fn calculate_risk_adjusted_signal_strength(
        &self,
        signal_type: &str, // "buy" or "sell"
        base_strength: f64,
        risk_factor: f64, // 0.0~1.0, 높을수록 위험
    ) -> f64 {
        if base_strength == 0.0 {
            return 0.0;
        }

        // 리스크 조정 계수 (리스크가 높을수록 신호 강도 감소)
        let risk_adjustment = 1.0 - (risk_factor * 0.5);

        // 시장 상황 고려
        let market_adjustment = if signal_type == "buy" {
            self.calculate_market_condition_factor()
        } else {
            2.0 - self.calculate_market_condition_factor()
        };

        let adjusted_strength = base_strength * risk_adjustment * market_adjustment;
        adjusted_strength.clamp(0.0, 1.0)
    }

    /// 강한 매수 신호 돌파 확인 (n개 연속 강한 매수 신호, 이전 m개는 아님)
    pub fn is_strong_buy_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        rsi_lower: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.calculate_buy_signal_strength(rsi_lower) > threshold,
            n,
            m,
            p,
        )
    }

    /// 강한 매도 신호 돌파 확인 (n개 연속 강한 매도 신호, 이전 m개는 아님)
    pub fn is_strong_sell_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        rsi_upper: f64,
        profit_percentage: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.calculate_sell_signal_strength(rsi_upper, profit_percentage) > threshold,
            n,
            m,
            p,
        )
    }

    /// 강화된 매수 신호 돌파 확인 (n개 연속 강화된 매수 신호, 이전 m개는 아님)
    pub fn is_enhanced_buy_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        rsi_lower: f64,
        volatility: f64,
        volume_factor: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                self.calculate_enhanced_buy_signal_strength(rsi_lower, volatility, volume_factor)
                    > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 강화된 매도 신호 돌파 확인 (n개 연속 강화된 매도 신호, 이전 m개는 아님)
    pub fn is_enhanced_sell_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        rsi_upper: f64,
        profit_percentage: f64,
        volatility: f64,
        volume_factor: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                self.calculate_enhanced_sell_signal_strength(
                    rsi_upper,
                    profit_percentage,
                    volatility,
                    volume_factor,
                ) > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 시장 상황 개선 신호 확인 (n개 연속 시장 상황 개선, 이전 m개는 아님)
    pub fn is_market_condition_improving_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.calculate_market_condition_factor() > threshold,
            n,
            m,
            p,
        )
    }

    /// 모멘텀 강화 신호 확인 (n개 연속 모멘텀 강화, 이전 m개는 아님)
    pub fn is_momentum_strengthening_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.calculate_momentum_factor() > threshold,
            n,
            m,
            p,
        )
    }

    /// 지표 합의 신호 확인 (n개 연속 지표 합의, 이전 m개는 아님)
    pub fn is_indicators_consensus_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.calculate_indicators_consensus_factor() > threshold,
            n,
            m,
            p,
        )
    }

    /// 리스크 조정 매수 신호 확인 (n개 연속 리스크 조정 매수 신호, 이전 m개는 아님)
    pub fn is_risk_adjusted_buy_signal(
        &self,
        n: usize,
        m: usize,
        base_strength: f64,
        risk_factor: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                self.calculate_risk_adjusted_signal_strength("buy", base_strength, risk_factor)
                    > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 리스크 조정 매도 신호 확인 (n개 연속 리스크 조정 매도 신호, 이전 m개는 아님)
    pub fn is_risk_adjusted_sell_signal(
        &self,
        n: usize,
        m: usize,
        base_strength: f64,
        risk_factor: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                self.calculate_risk_adjusted_signal_strength("sell", base_strength, risk_factor)
                    > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 복합 신호 강도 임계값 돌파 확인 (n개 연속 임계값 초과, 이전 m개는 아님)
    pub fn is_composite_signal_strength_breakthrough(
        &self,
        n: usize,
        m: usize,
        signal_type: &str,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                // MA, MACD, RSI의 조합 신호 강도 계산
                let ma_signal = if signal_type == "buy" {
                    data.candle.close_price() > data.ma.get()
                } else {
                    data.candle.close_price() < data.ma.get()
                };

                let macd_signal = if signal_type == "buy" {
                    data.macd.macd_line > data.macd.signal_line && data.macd.histogram > 0.0
                } else {
                    data.macd.macd_line < data.macd.signal_line && data.macd.histogram < 0.0
                };

                let rsi_signal = if signal_type == "buy" {
                    data.rsi.value() < RSI_OVERSOLD_THRESHOLD // 과매도에서 반등 신호
                } else {
                    data.rsi.value() > RSI_OVERBOUGHT_THRESHOLD // 과매수에서 하락 신호
                };

                let signal_count = [ma_signal, macd_signal, rsi_signal]
                    .iter()
                    .filter(|&&x| x)
                    .count();
                (signal_count as f64 / 3.0) > threshold
            },
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 강한 매수 신호인지 확인
    pub fn is_strong_buy_signal(&self, n: usize, rsi_lower: f64, threshold: f64, p: usize) -> bool {
        self.is_all(
            |_| self.calculate_buy_signal_strength(rsi_lower) > threshold,
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 강한 매도 신호인지 확인
    pub fn is_strong_sell_signal(
        &self,
        n: usize,
        rsi_upper: f64,
        profit_percentage: f64,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_all(
            |_| self.calculate_sell_signal_strength(rsi_upper, profit_percentage) > threshold,
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 시장 상황이 좋은지 확인
    pub fn is_good_market_condition(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(
            |_| self.calculate_market_condition_factor() > threshold,
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 모멘텀이 강한지 확인
    pub fn is_strong_momentum(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(|_| self.calculate_momentum_factor() > threshold, n, p)
    }
}

impl<C: Candle + Clone> AnalyzerOps<HybridAnalyzerData<C>, C> for HybridAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> HybridAnalyzerData<C> {
        let ma = self.mabuilder.next(&candle);
        let macd = self.macdbuilder.next(&candle);
        let rsi = self.rsibuilder.next(&candle);

        let data = HybridAnalyzerData::new(candle, ma, macd, rsi);
        data.clone_with_stored_values()
    }

    fn datum(&self) -> &Vec<HybridAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<HybridAnalyzerData<C>> {
        &mut self.items
    }
}
