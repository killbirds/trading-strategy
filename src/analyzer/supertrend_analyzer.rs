use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::supertrend::{
    SuperTrend, SuperTrends, SuperTrendsBuilder, SuperTrendsBuilderFactory,
};
use std::fmt::Display;
use trading_chart::Candle;

/// 슈퍼트렌드 분석기 데이터
#[derive(Debug)]
pub struct SuperTrendAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 슈퍼트렌드 지표 집합
    pub supertrends: SuperTrends,
}

impl<C: Candle> SuperTrendAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, supertrends: SuperTrends) -> SuperTrendAnalyzerData<C> {
        SuperTrendAnalyzerData {
            candle,
            supertrends,
        }
    }

    /// 특정 슈퍼트렌드 값 반환
    pub fn get_supertrend(&self, period: &usize, multiplier: &f64) -> SuperTrend {
        self.supertrends.get(period, multiplier)
    }

    /// 특정 설정의 슈퍼트렌드가 상승 추세인지 확인
    pub fn is_uptrend(&self, period: &usize, multiplier: &f64) -> bool {
        self.supertrends.get(period, multiplier).is_uptrend()
    }

    /// 특정 설정의 슈퍼트렌드가 하락 추세인지 확인
    pub fn is_downtrend(&self, period: &usize, multiplier: &f64) -> bool {
        self.supertrends.get(period, multiplier).is_downtrend()
    }
}

impl<C: Candle> GetCandle<C> for SuperTrendAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for SuperTrendAnalyzerData<C> {}

/// 슈퍼트렌드 분석기
#[derive(Debug)]
pub struct SuperTrendAnalyzer<C: Candle> {
    /// 슈퍼트렌드 빌더
    pub supertrendsbuilder: SuperTrendsBuilder<C>,
    /// 분석 데이터 히스토리
    pub items: Vec<SuperTrendAnalyzerData<C>>,
    /// 기간 및 승수 설정 목록
    periods: Vec<(usize, f64)>,
}

impl<C: Candle> Display for SuperTrendAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let settings = self
            .periods
            .iter()
            .map(|(p, m)| format!("({p}:{m})"))
            .collect::<Vec<_>>()
            .join(", ");

        write!(
            f,
            "SuperTrendAnalyzer {{ settings: [{}], items: {} }}",
            settings,
            self.items.len()
        )
    }
}

impl<C: Candle> SuperTrendAnalyzer<C> {
    /// 새 슈퍼트렌드 분석기 생성
    pub fn new(periods: &[(usize, f64)], storage: &CandleStore<C>) -> SuperTrendAnalyzer<C> {
        let supertrendsbuilder = SuperTrendsBuilderFactory::build(periods);
        let mut analyzer = SuperTrendAnalyzer {
            supertrendsbuilder,
            items: Vec::new(),
            periods: periods.to_vec(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 저장소에서 초기 데이터 계산
    pub fn init_from_storage(&mut self, storage: &CandleStore<C>) {
        for candle in storage.get_time_ordered_items().iter() {
            self.next(candle.clone());
        }
    }

    /// 모든 슈퍼트렌드가 상승 추세인지 확인
    pub fn is_all_uptrend(&self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        for (period, multiplier) in &self.periods {
            if !self.items[0].is_uptrend(period, multiplier) {
                return false;
            }
        }

        true
    }

    /// 모든 슈퍼트렌드가 하락 추세인지 확인
    pub fn is_all_downtrend(&self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        for (period, multiplier) in &self.periods {
            if !self.items[0].is_downtrend(period, multiplier) {
                return false;
            }
        }

        true
    }

    /// 특정 슈퍼트렌드에서 추세 전환이 일어났는지 확인 (상승->하락 또는 하락->상승)
    pub fn is_trend_changed(&self, period: &usize, multiplier: &f64, n: usize) -> bool {
        if self.items.len() <= n {
            return false;
        }

        let current_direction = self.items[0].get_supertrend(period, multiplier).direction;
        let previous_direction = self.items[n].get_supertrend(period, multiplier).direction;

        current_direction != previous_direction && current_direction != 0 && previous_direction != 0
    }

    /// 가격이 슈퍼트렌드 위에 있는지 확인 (가격 > 슈퍼트렌드)
    pub fn is_price_above_supertrend(&self, period: &usize, multiplier: &f64) -> bool {
        if self.items.is_empty() {
            return false;
        }

        let candle = &self.items[0].candle;
        let st = self.items[0].get_supertrend(period, multiplier);

        candle.close_price() > st.value
    }

    /// 가격이 슈퍼트렌드 아래에 있는지 확인 (가격 < 슈퍼트렌드)
    pub fn is_price_below_supertrend(&self, period: &usize, multiplier: &f64) -> bool {
        if self.items.is_empty() {
            return false;
        }

        let candle = &self.items[0].candle;
        let st = self.items[0].get_supertrend(period, multiplier);

        candle.close_price() < st.value
    }

    /// 가격이 슈퍼트렌드를 상향 돌파했는지 확인 (전: 가격<슈퍼트렌드, 현재: 가격>슈퍼트렌드)
    pub fn is_price_crossing_above_supertrend(&self, period: &usize, multiplier: &f64) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_candle = &self.items[0].candle;
        let previous_candle = &self.items[1].candle;

        let current_st = self.items[0].get_supertrend(period, multiplier);
        let previous_st = self.items[1].get_supertrend(period, multiplier);

        previous_candle.close_price() < previous_st.value
            && current_candle.close_price() > current_st.value
    }

    /// 가격이 슈퍼트렌드를 하향 돌파했는지 확인 (전: 가격>슈퍼트렌드, 현재: 가격<슈퍼트렌드)
    pub fn is_price_crossing_below_supertrend(&self, period: &usize, multiplier: &f64) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_candle = &self.items[0].candle;
        let previous_candle = &self.items[1].candle;

        let current_st = self.items[0].get_supertrend(period, multiplier);
        let previous_st = self.items[1].get_supertrend(period, multiplier);

        previous_candle.close_price() > previous_st.value
            && current_candle.close_price() < current_st.value
    }

    /// 가격이 슈퍼트렌드 위 신호 확인 (n개 연속 가격 > 슈퍼트렌드, 이전 m개는 아님)
    pub fn is_price_above_supertrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let st = data.get_supertrend(&period, &multiplier);
                data.candle.close_price() > st.value
            },
            n,
            m,
            p,
        )
    }

    /// 가격이 슈퍼트렌드 아래 신호 확인 (n개 연속 가격 < 슈퍼트렌드, 이전 m개는 아님)
    pub fn is_price_below_supertrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let st = data.get_supertrend(&period, &multiplier);
                data.candle.close_price() < st.value
            },
            n,
            m,
            p,
        )
    }

    /// 상승 추세 신호 확인 (n개 연속 상승 추세, 이전 m개는 아님)
    pub fn is_uptrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_uptrend(&period, &multiplier), n, m, p)
    }

    /// 하락 추세 신호 확인 (n개 연속 하락 추세, 이전 m개는 아님)
    pub fn is_downtrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_downtrend(&period, &multiplier), n, m, p)
    }

    /// 전체 상승 추세 신호 확인 (n개 연속 모든 슈퍼트렌드 상승, 이전 m개는 아님)
    pub fn is_all_uptrend_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                if self.items.is_empty() {
                    return false;
                }

                for (period, multiplier) in &self.periods {
                    let data = &self.items[0];
                    if !data.is_uptrend(period, multiplier) {
                        return false;
                    }
                }
                true
            },
            n,
            m,
            p,
        )
    }

    /// 전체 하락 추세 신호 확인 (n개 연속 모든 슈퍼트렌드 하락, 이전 m개는 아님)
    pub fn is_all_downtrend_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                if self.items.is_empty() {
                    return false;
                }

                for (period, multiplier) in &self.periods {
                    let data = &self.items[0];
                    if !data.is_downtrend(period, multiplier) {
                        return false;
                    }
                }
                true
            },
            n,
            m,
            p,
        )
    }

    /// 슈퍼트렌드 상향 돌파 신호 확인 (n개 연속 상향 돌파, 이전 m개는 아님)
    pub fn is_price_crossing_above_supertrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.is_price_crossing_above_supertrend(&period, &multiplier),
            n,
            m,
            p,
        )
    }

    /// 슈퍼트렌드 하향 돌파 신호 확인 (n개 연속 하향 돌파, 이전 m개는 아님)
    pub fn is_price_crossing_below_supertrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.is_price_crossing_below_supertrend(&period, &multiplier),
            n,
            m,
            p,
        )
    }

    /// 추세 변화 신호 확인 (n개 연속 추세 변화, 이전 m개는 아님)
    pub fn is_trend_changed_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        multiplier: f64,
        trend_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.is_trend_changed(&period, &multiplier, trend_period),
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 가격이 슈퍼트렌드 위인지 확인
    pub fn is_price_above_supertrend_continuous(
        &self,
        n: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_all(
            |data| {
                let st = data.get_supertrend(&period, &multiplier);
                data.candle.close_price() > st.value
            },
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 가격이 슈퍼트렌드 아래인지 확인
    pub fn is_price_below_supertrend_continuous(
        &self,
        n: usize,
        period: usize,
        multiplier: f64,
        p: usize,
    ) -> bool {
        self.is_all(
            |data| {
                let st = data.get_supertrend(&period, &multiplier);
                data.candle.close_price() < st.value
            },
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 상승 추세인지 확인
    pub fn is_uptrend(&self, n: usize, period: usize, multiplier: f64, p: usize) -> bool {
        self.is_all(|data| data.is_uptrend(&period, &multiplier), n, p)
    }

    /// n개의 연속 데이터에서 하락 추세인지 확인
    pub fn is_downtrend(&self, n: usize, period: usize, multiplier: f64, p: usize) -> bool {
        self.is_all(|data| data.is_downtrend(&period, &multiplier), n, p)
    }
}

impl<C: Candle> AnalyzerOps<SuperTrendAnalyzerData<C>, C> for SuperTrendAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> SuperTrendAnalyzerData<C> {
        let supertrends = self.supertrendsbuilder.next(&candle);
        SuperTrendAnalyzerData::new(candle, supertrends)
    }

    fn datum(&self) -> &Vec<SuperTrendAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<SuperTrendAnalyzerData<C>> {
        &mut self.items
    }
}
