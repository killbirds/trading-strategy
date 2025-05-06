use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::macd::{MACD, MACDBuilder};
use crate::indicator::rsi::{RSI, RSIBuilder};
use std::fmt::Display;
use trading_chart::Candle;

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

        let current = &self.items[0];
        let previous = &self.items[1];
        let before_previous = &self.items[2];

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
            signal_strength += MA_WEIGHT * 0.6;

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
                signal_strength += MACD_HIST_WEIGHT * 0.5;
            }
        }

        // 4. RSI 기반 신호
        let rsi = current.rsi.value();

        if rsi < rsi_lower {
            // 과매도 상태 (강한 매수 신호)
            signal_strength += RSI_WEIGHT * (1.0 - rsi / rsi_lower);
        } else if rsi < 45.0 && rsi > previous.rsi.value() {
            // RSI가 낮은 상태에서 반등 중 (적절한 매수 신호)
            signal_strength += RSI_WEIGHT * 0.5 * (45.0 - rsi) / 15.0;
        }

        // 최종 신호 강도 (0.0~1.0 범위로 클램핑)
        signal_strength.min(1.0).max(0.0)
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

        let current = &self.items[0];
        let previous = &self.items[1];
        let before_previous = &self.items[2];

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
            signal_strength += MA_WEIGHT * 0.6;

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
                signal_strength += MACD_HIST_WEIGHT * 0.5;
            }
        }

        // 4. RSI 기반 신호
        let rsi = current.rsi.value();

        if rsi > rsi_upper {
            // 과매수 상태 (강한 매도 신호)
            signal_strength += RSI_WEIGHT * ((rsi - rsi_upper) / (100.0 - rsi_upper));
        } else if rsi > 55.0 && rsi < previous.rsi.value() {
            // RSI가 높은 상태에서 하락 중 (적절한 매도 신호)
            signal_strength += RSI_WEIGHT * 0.5 * (rsi - 55.0) / 15.0;
        }

        // 5. 수익률 기반 신호
        if profit_percentage > 7.0 {
            // 높은 수익 실현 (강한 매도 신호)
            signal_strength += PROFIT_WEIGHT;
        } else if profit_percentage > 3.0 {
            // 적정 수익 실현 (중간 매도 신호)
            signal_strength += PROFIT_WEIGHT * 0.7;
        } else if profit_percentage < -5.0 {
            // 큰 손실 발생 (손절 매도 신호)
            signal_strength += PROFIT_WEIGHT * 0.8;
        }

        // 최종 신호 강도 (0.0~1.0 범위로 클램핑)
        signal_strength.min(1.0).max(0.0)
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
