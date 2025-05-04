use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// MA 전략 데이터
#[derive(Debug)]
pub struct MAAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 이동평균선 집합
    pub mas: MAs,
}

impl<C: Candle> MAAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, mas: MAs) -> MAAnalyzerData<C> {
        MAAnalyzerData { candle, mas }
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

impl<C: Candle> GetCandle<C> for MAAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for MAAnalyzerData<C> {}

/// MA 전략 컨텍스트
#[derive(Debug)]
pub struct MAAnalyzer<C: Candle> {
    /// 이동평균 빌더
    pub masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<MAAnalyzerData<C>>,
}

impl<C: Candle> Display for MAAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MAs: {}", first.candle, first.mas),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> MAAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(ma_type: &MAType, ma_periods: &[usize], storage: &CandleStore<C>) -> MAAnalyzer<C> {
        let masbuilder = MAsBuilderFactory::build::<C>(ma_type, ma_periods);
        let mut ctx = MAAnalyzer {
            masbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 특정 인덱스의 이동평균 수익률이 목표치보다 작은지 확인
    pub fn is_ma_less_than_rate_of_return(
        &self,
        index: usize,
        rate_of_return: f64,
        n: usize,
    ) -> bool {
        self.is_all(
            |data| {
                data.is_less_than_target(
                    |data| {
                        data.get_rate_of_return(|data| {
                            let ma = data.mas.get_from_index(index);
                            ma.get()
                        })
                    },
                    rate_of_return,
                )
            },
            n,
        )
    }

    /// 특정 인덱스의 이동평균 수익률이 목표치보다 큰지 확인
    pub fn is_ma_greater_than_rate_of_return(
        &self,
        index: usize,
        rate_of_return: f64,
        n: usize,
    ) -> bool {
        self.is_all(
            |data| {
                data.is_greater_than_target(
                    |data| {
                        data.get_rate_of_return(|data| {
                            let ma = data.mas.get_from_index(index);
                            ma.get()
                        })
                    },
                    rate_of_return,
                )
            },
            n,
        )
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    pub fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// 골든 크로스 패턴 확인 (정규 배열이 n개 연속, 이전 m개는 아님)
    pub fn is_ma_regular_arrangement_golden_cross(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_regular_arrangement(), n, m)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    pub fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
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
}

impl<C: Candle> AnalyzerOps<MAAnalyzerData<C>, C> for MAAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> MAAnalyzerData<C> {
        let mas = self.masbuilder.next(&candle);
        MAAnalyzerData::new(candle, mas)
    }

    fn datum(&self) -> &Vec<MAAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<MAAnalyzerData<C>> {
        &mut self.items
    }
}
