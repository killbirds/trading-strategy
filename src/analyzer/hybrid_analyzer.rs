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

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 매수 신호 강도 계산
    pub fn calculate_buy_signal_strength(&self, rsi_lower: f64) -> f64 {
        if self.items.len() < 2 {
            return 0.0;
        }

        let current = self.items.last().unwrap();
        let previous = &self.items[self.items.len() - 2];

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() > current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호
        if current.macd.histogram > 0.0 && previous.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선을 상향 돌파 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선 위에 있음 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi < rsi_lower && rsi > previous.rsi.value() {
            // RSI가 과매도 상태에서 반등 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi > rsi_lower && rsi < 50.0 {
            // RSI가 과매도 상태를 벗어나 상승 중 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / (count * 2.0) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        }
    }

    /// 매도 신호 강도 계산
    pub fn calculate_sell_signal_strength(&self, rsi_upper: f64, profit_percentage: f64) -> f64 {
        if self.items.len() < 2 {
            return 0.0;
        }

        let current = self.items.last().unwrap();
        let previous = &self.items[self.items.len() - 2];

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() < current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호
        if current.macd.histogram < 0.0 && previous.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선을 하향 돌파 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선 아래에 있음 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi > rsi_upper && rsi < previous.rsi.value() {
            // RSI가 과매수 상태에서 하락 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi < rsi_upper && rsi > 50.0 {
            // RSI가 과매수 상태로 접근 중 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 4. 수익률 기반 신호
        if profit_percentage > 5.0 {
            // 5% 이상 수익 (적절한 매도 신호)
            strength += 1.0;
            count += 1.0;
        } else if profit_percentage < -3.0 {
            // 3% 이상 손실 (손절 매도 신호)
            strength += 1.5;
            count += 1.0;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / (count * 2.0) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        }
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
