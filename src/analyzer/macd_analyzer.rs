use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::macd::{MACD, MACDBuilder};
use std::fmt::Display;
use trading_chart::Candle;

/// MACD 분석기 데이터
#[derive(Debug)]
pub struct MACDAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// MACD 지표
    pub macd: MACD,
}

impl<C: Candle> MACDAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, macd: MACD) -> MACDAnalyzerData<C> {
        MACDAnalyzerData { candle, macd }
    }

    /// MACD 히스토그램이 임계값보다 큰지 확인 (상승 추세)
    pub fn is_histogram_above_threshold(&self, threshold: f64) -> bool {
        self.macd.histogram > threshold
    }

    /// MACD 히스토그램이 임계값보다 작은지 확인 (하락 추세)
    pub fn is_histogram_below_threshold(&self, threshold: f64) -> bool {
        self.macd.histogram < threshold
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    pub fn is_macd_above_signal(&self) -> bool {
        self.macd.macd_line > self.macd.signal_line
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    pub fn is_macd_below_signal(&self) -> bool {
        self.macd.macd_line < self.macd.signal_line
    }
}

impl<C: Candle> GetCandle<C> for MACDAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for MACDAnalyzerData<C> {}

/// MACD 분석기 컨텍스트
#[derive(Debug)]
pub struct MACDAnalyzer<C: Candle> {
    /// MACD 빌더
    pub macdbuilder: MACDBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<MACDAnalyzerData<C>>,
}

impl<C: Candle> Display for MACDAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MACD: {}", first.candle, first.macd),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> MACDAnalyzer<C> {
    /// 새 분석기 컨텍스트 생성
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        storage: &CandleStore<C>,
    ) -> MACDAnalyzer<C> {
        let macdbuilder = MACDBuilder::new(fast_period, slow_period, signal_period);
        let mut ctx = MACDAnalyzer {
            macdbuilder,
            items: vec![],
        };

        ctx.init_from_storage(storage);
        ctx
    }

    /// 히스토그램이 임계값보다 큰지 확인
    pub fn is_histogram_above_threshold(&self, threshold: f64, n: usize) -> bool {
        self.is_all(|data| data.is_histogram_above_threshold(threshold), n)
    }

    /// 히스토그램이 임계값보다 작은지 확인
    pub fn is_histogram_below_threshold(&self, threshold: f64, n: usize) -> bool {
        self.is_all(|data| data.is_histogram_below_threshold(threshold), n)
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    ///
    /// # Arguments
    /// * `n` - 현재 상태가 MACD > 시그널인 연속 캔들 수 (사용되지 않음)
    /// * `m` - 이전 상태가 MACD < 시그널인 연속 캔들 수 (사용되지 않음)
    ///
    /// # Returns
    /// * `bool` - 상향 돌파 패턴이 확인되면 true
    pub fn is_macd_crossed_above_signal(&self, _n: usize, _m: usize) -> bool {
        // 최소 2개의 캔들이 필요
        if self.items.len() < 2 {
            return false;
        }

        // 최신 캔들(items[0])에서 MACD가 시그널보다 위에 있고,
        // 그 이전 캔들들 중 하나라도 MACD가 시그널보다 아래에 있는지 확인
        let current_macd_above_signal =
            self.items[0].macd.macd_line > self.items[0].macd.signal_line;

        // 최근 일부 캔들 중에 하나라도 MACD가 시그널보다 아래에 있는지 확인 (crossover 확인)
        let mut found_prev_below = false;
        for i in 1..self.items.len().min(10) {
            // 10개 캔들까지만 검사
            if self.items[i].macd.macd_line < self.items[i].macd.signal_line {
                found_prev_below = true;
                break;
            }
        }

        current_macd_above_signal && found_prev_below
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    ///
    /// # Arguments
    /// * `n` - 현재 상태가 MACD < 시그널인 연속 캔들 수 (사용되지 않음)
    /// * `m` - 이전 상태가 MACD > 시그널인 연속 캔들 수 (사용되지 않음)
    ///
    /// # Returns
    /// * `bool` - 하향 돌파 패턴이 확인되면 true
    pub fn is_macd_crossed_below_signal(&self, _n: usize, _m: usize) -> bool {
        // 최소 2개의 캔들이 필요
        if self.items.len() < 2 {
            return false;
        }

        // 최신 캔들(items[0])에서 MACD가 시그널보다 아래에 있고,
        // 그 이전 캔들들 중 하나라도 MACD가 시그널보다 위에 있는지 확인
        let current_macd_below_signal =
            self.items[0].macd.macd_line < self.items[0].macd.signal_line;

        // 최근 일부 캔들 중에 하나라도 MACD가 시그널보다 위에 있는지 확인 (crossover 확인)
        let mut found_prev_above = false;
        for i in 1..self.items.len().min(10) {
            // 10개 캔들까지만 검사
            if self.items[i].macd.macd_line > self.items[i].macd.signal_line {
                found_prev_above = true;
                break;
            }
        }

        current_macd_below_signal && found_prev_above
    }
}

impl<C: Candle> AnalyzerOps<MACDAnalyzerData<C>, C> for MACDAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> MACDAnalyzerData<C> {
        let macd = self.macdbuilder.next(&candle);
        MACDAnalyzerData::new(candle, macd)
    }

    fn datum(&self) -> &Vec<MACDAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<MACDAnalyzerData<C>> {
        &mut self.items
    }
}
