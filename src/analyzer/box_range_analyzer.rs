use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::box_range::{BoxRange, BoxRangeBreakout, BoxRangeBuilder};
use std::fmt::Display;
use trading_chart::Candle;

/// 박스권 분석기 데이터
#[derive(Debug)]
pub struct BoxRangeAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 박스권 지표
    pub box_range: BoxRange,
}

impl<C: Candle> BoxRangeAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, box_range: BoxRange) -> BoxRangeAnalyzerData<C> {
        BoxRangeAnalyzerData { candle, box_range }
    }

    /// 현재 종가가 박스 안에 있는지 확인
    pub fn is_close_inside_box(&self) -> bool {
        self.box_range.contains_price(self.candle.close_price())
    }

    /// 현재 종가 기준 박스 돌파 방향 반환
    pub fn close_breakout_direction(&self) -> BoxRangeBreakout {
        self.box_range.breakout_direction(self.candle.close_price())
    }
}

impl<C: Candle> GetCandle<C> for BoxRangeAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for BoxRangeAnalyzerData<C> {}

/// 박스권 분석기
#[derive(Debug)]
pub struct BoxRangeAnalyzer<C: Candle> {
    /// 박스권 빌더
    pub box_range_builder: BoxRangeBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<BoxRangeAnalyzerData<C>>,
}

impl<C: Candle> Display for BoxRangeAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, 박스권: {{상: {:.2}, 중: {:.2}, 하: {:.2}, 비율: {:.4}, 여부: {}}}",
                first.candle,
                first.box_range.upper(),
                first.box_range.middle(),
                first.box_range.lower(),
                first.box_range.width_ratio(),
                first.box_range.is_box_range()
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> BoxRangeAnalyzer<C> {
    /// 새 박스권 분석기 생성
    pub fn new(
        period: usize,
        max_width_ratio: f64,
        storage: &CandleStore<C>,
    ) -> BoxRangeAnalyzer<C> {
        let box_range_builder = BoxRangeBuilder::new(period, max_width_ratio);
        let mut analyzer = BoxRangeAnalyzer {
            box_range_builder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 현재 박스권 하단, 중간, 상단 반환
    pub fn get_box_range(&self) -> (f64, f64, f64) {
        match self.items.first() {
            Some(data) => (
                data.box_range.lower(),
                data.box_range.middle(),
                data.box_range.upper(),
            ),
            None => (0.0, 0.0, 0.0),
        }
    }

    /// 현재 박스권 폭 비율 반환
    pub fn get_box_width_ratio(&self) -> f64 {
        self.items
            .first()
            .map(|data| data.box_range.width_ratio())
            .unwrap_or(0.0)
    }

    /// n개의 연속 데이터가 박스권인지 확인
    pub fn is_box_range(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.box_range.is_box_range(), n, p)
    }

    /// n개의 연속 종가가 박스 안에 있는지 확인
    pub fn is_close_inside_box(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_close_inside_box(), n, p)
    }

    /// n개의 연속 종가가 박스 상단 위에 있는지 확인
    pub fn is_close_above_box(&self, n: usize, p: usize) -> bool {
        self.is_against_previous_box(n, p, |current, previous| {
            current.candle.close_price() > previous.box_range.upper()
        })
    }

    /// n개의 연속 종가가 박스 하단 아래에 있는지 확인
    pub fn is_close_below_box(&self, n: usize, p: usize) -> bool {
        self.is_against_previous_box(n, p, |current, previous| {
            current.candle.close_price() < previous.box_range.lower()
        })
    }

    /// 고가가 박스 상단을 돌파했는지 확인
    pub fn is_high_break_through_upper_box(&self, n: usize, p: usize) -> bool {
        self.is_against_previous_box(n, p, |current, previous| {
            current.candle.high_price() > previous.box_range.upper()
        })
    }

    /// 저가가 박스 하단을 이탈했는지 확인
    pub fn is_low_break_through_lower_box(&self, n: usize, p: usize) -> bool {
        self.is_against_previous_box(n, p, |current, previous| {
            current.candle.low_price() < previous.box_range.lower()
        })
    }

    /// 박스권 상태가 새로 시작되는지 확인
    pub fn is_box_range_start(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.box_range.is_box_range(), n, m, p)
    }

    /// 박스권 이후 종가가 상단을 돌파하는지 확인
    pub fn is_box_breakout_above(&self, box_period: usize) -> bool {
        if self.items.len() <= box_period {
            return false;
        }

        let current = match self.items.first() {
            Some(data) => data,
            None => return false,
        };

        let previous = match self.items.get(1) {
            Some(data) => data,
            None => return false,
        };

        current.candle.close_price() > previous.box_range.upper()
            && self.is_box_range(box_period, 1)
    }

    /// 박스권 이후 종가가 하단을 이탈하는지 확인
    pub fn is_box_breakout_below(&self, box_period: usize) -> bool {
        if self.items.len() <= box_period {
            return false;
        }

        let current = match self.items.first() {
            Some(data) => data,
            None => return false,
        };

        let previous = match self.items.get(1) {
            Some(data) => data,
            None => return false,
        };

        current.candle.close_price() < previous.box_range.lower()
            && self.is_box_range(box_period, 1)
    }

    fn is_against_previous_box(
        &self,
        n: usize,
        p: usize,
        predicate: impl Fn(&BoxRangeAnalyzerData<C>, &BoxRangeAnalyzerData<C>) -> bool,
    ) -> bool {
        if self.items.len() < n + p + 1 {
            return false;
        }

        for index in p..p + n {
            let current = match self.items.get(index) {
                Some(data) => data,
                None => return false,
            };
            let previous = match self.items.get(index + 1) {
                Some(data) => data,
                None => return false,
            };

            if !previous.box_range.is_box_range() || !predicate(current, previous) {
                return false;
            }
        }

        true
    }
}

impl<C: Candle> AnalyzerOps<BoxRangeAnalyzerData<C>, C> for BoxRangeAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> BoxRangeAnalyzerData<C> {
        let box_range = self.box_range_builder.next(&candle);
        BoxRangeAnalyzerData::new(candle, box_range)
    }

    fn items(&self) -> &Vec<BoxRangeAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<BoxRangeAnalyzerData<C>> {
        &mut self.items
    }
}
