use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use crate::indicator::rsi::{RSI, RSIBuilder};
use std::fmt::Display;
use trading_chart::Candle;

/// RSI 전략 데이터
#[derive(Debug)]
pub struct RSIAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// RSI 데이터
    pub rsi: RSI,
    /// 이동평균선 집합
    pub mas: MAs,
}

impl<C: Candle> RSIAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, mas: MAs, rsi: RSI) -> RSIAnalyzerData<C> {
        RSIAnalyzerData { candle, rsi, mas }
    }

    /// 이동평균이 정규 배열(오름차순)인지 확인
    pub fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균이 역배열(내림차순)인지 확인
    pub fn is_ma_reverse_arrangement(&self) -> bool {
        self.is_reverse_arrangement(|data| &data.mas, |ma| ma.get())
    }
}

impl<C: Candle> GetCandle<C> for RSIAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for RSIAnalyzerData<C> {}

/// RSI 전략 컨텍스트
#[derive(Debug)]
pub struct RSIAnalyzer<C: Candle> {
    /// RSI 빌더
    pub rsibuilder: RSIBuilder<C>,
    /// 이동평균 빌더
    pub masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<RSIAnalyzerData<C>>,
}

impl<C: Candle> Display for RSIAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, RSI: {}, MAs: {}",
                first.candle, first.rsi, first.mas
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> RSIAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        rsi_period: usize,
        ma_type: &MAType,
        ma_periods: &[usize],
        storage: &CandleStore<C>,
    ) -> RSIAnalyzer<C> {
        let rsibuilder = RSIBuilder::new(rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(ma_type, ma_periods);
        let mut ctx = RSIAnalyzer {
            rsibuilder,
            masbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    pub fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    pub fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }

    /// 골든 크로스 패턴 확인 (정규 배열이 n개 연속, 이전 m개는 아님)
    pub fn is_ma_regular_arrangement_golden_cross(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_regular_arrangement(), n, m)
    }

    /// 데드 크로스 패턴 확인 (역배열이 n개 연속, 이전 m개는 아님)
    pub fn is_ma_reverse_arrangement_dead_cross(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_reverse_arrangement(), n, m)
    }

    /// 특정 인덱스의 이동평균 값 반환
    pub fn get_ma(&self, index: usize) -> f64 {
        self.get(0, |data| data.mas.get_from_index(index).get())
    }

    /// 단기와 장기 이동평균의 교차 여부 확인
    pub fn is_ma_crossed(&self, short_index: usize, long_index: usize) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = &self.items[0];
        let previous = &self.items[1];

        let current_short = current.mas.get_from_index(short_index).get();
        let current_long = current.mas.get_from_index(long_index).get();
        let previous_short = previous.mas.get_from_index(short_index).get();
        let previous_long = previous.mas.get_from_index(long_index).get();

        (current_short > current_long) != (previous_short > previous_long)
    }

    /// RSI 값 반환
    pub fn get_rsi(&self) -> f64 {
        self.get(0, |data| data.rsi.value())
    }

    /// RSI 값이 특정 값보다 작은지 확인
    pub fn is_rsi_less_than(&self, value: f64, n: usize) -> bool {
        self.is_all(|data| data.rsi.value() < value, n)
    }

    /// RSI 값이 특정 값보다 큰지 확인
    pub fn is_rsi_greater_than(&self, value: f64, n: usize) -> bool {
        self.is_all(|data| data.rsi.value() > value, n)
    }
}

impl<C: Candle> AnalyzerOps<RSIAnalyzerData<C>, C> for RSIAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> RSIAnalyzerData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        RSIAnalyzerData::new(candle, mas, rsi)
    }

    fn datum(&self) -> &Vec<RSIAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<RSIAnalyzerData<C>> {
        &mut self.items
    }
}
