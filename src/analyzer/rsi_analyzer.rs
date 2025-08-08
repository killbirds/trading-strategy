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

        ctx.init_from_storage(storage);
        ctx
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    pub fn is_ma_regular_arrangement(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n, p)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    pub fn is_ma_reverse_arrangement(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n, p)
    }

    /// 골든 크로스 패턴 확인 (정규 배열이 n개 연속, 이전 m개는 아님)
    pub fn is_ma_regular_arrangement_golden_cross(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_regular_arrangement(), n, m, p)
    }

    /// 데드 크로스 패턴 확인 (역배열이 n개 연속, 이전 m개는 아님)
    pub fn is_ma_reverse_arrangement_dead_cross(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_reverse_arrangement(), n, m, p)
    }

    /// 특정 인덱스의 이동평균 값 반환
    pub fn get_ma(&self, index: usize) -> f64 {
        self.get_value(0, |data| data.mas.get_by_key_index(index).get())
    }

    /// 단기와 장기 이동평균의 교차 여부 확인
    pub fn is_ma_crossed(&self, short_index: usize, long_index: usize) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        // 현재 캔들에서 단기선이 장기선보다 위에 있는지 확인
        let current = &self.items[0];
        let current_short = current.mas.get_by_key_index(short_index).get();
        let current_long = current.mas.get_by_key_index(long_index).get();
        let is_current_short_above_long = current_short > current_long;

        // 이전 캔들들 중에서 단기선과 장기선의 관계가 현재와 반대인 경우가 있는지 확인
        let mut found_opposite_position = false;
        for i in 1..self.items.len().min(10) {
            // 최대 10개 캔들만 확인
            let prev = &self.items[i];
            let prev_short = prev.mas.get_by_key_index(short_index).get();
            let prev_long = prev.mas.get_by_key_index(long_index).get();
            let is_prev_short_above_long = prev_short > prev_long;

            // 현재와 이전의 단기/장기 관계가 다르면 교차가 있었다는 의미
            if is_current_short_above_long != is_prev_short_above_long {
                found_opposite_position = true;
                break;
            }
        }

        found_opposite_position
    }

    /// RSI 값 반환
    pub fn get_rsi(&self) -> f64 {
        self.get_value(0, |data| data.rsi.value())
    }

    /// RSI 값이 특정 값보다 작은지 확인
    pub fn is_rsi_less_than(&self, value: f64, n: usize, p: usize) -> bool {
        self.is_all(|data| data.rsi.value() < value, n, p)
    }

    /// RSI 값이 특정 값보다 큰지 확인
    pub fn is_rsi_greater_than(&self, value: f64, n: usize, p: usize) -> bool {
        self.is_all(|data| data.rsi.value() > value, n, p)
    }

    /// n개의 연속 데이터에서 RSI가 횡보 상태인지 확인
    pub fn is_rsi_sideways(&self, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &RSIAnalyzerData<C>| data.rsi.value(),
            n,
            p,
            threshold,
        )
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
