use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::bband::{BollingerBands, BollingerBandsBuilder};
use std::fmt::Display;
use trading_chart::Candle;

/// 볼린저 밴드 분석기 데이터
#[derive(Debug)]
pub struct BBandAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 볼린저 밴드
    pub bband: BollingerBands,
}

impl<C: Candle> BBandAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, bband: BollingerBands) -> BBandAnalyzerData<C> {
        BBandAnalyzerData { candle, bband }
    }
}

impl<C: Candle> GetCandle<C> for BBandAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for BBandAnalyzerData<C> {}

/// 볼린저 밴드 분석기 컨텍스트
#[derive(Debug)]
pub struct BBandAnalyzer<C: Candle> {
    /// 볼린저 밴드 빌더
    pub bbandbuilder: BollingerBandsBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<BBandAnalyzerData<C>>,
}

impl<C: Candle> Display for BBandAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, 밴드: {{상: {:.2}, 중: {:.2}, 하: {:.2}}}",
                first.candle,
                first.bband.upper(),
                first.bband.middle(),
                first.bband.lower()
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> BBandAnalyzer<C> {
    /// 새 분석기 컨텍스트 생성
    pub fn new(period: usize, multiplier: f64, storage: &CandleStore<C>) -> BBandAnalyzer<C> {
        let bbandbuilder = BollingerBandsBuilder::new(period, multiplier);
        let mut ctx = BBandAnalyzer {
            bbandbuilder,
            items: vec![],
        };

        ctx.init_from_storage(storage);
        ctx
    }

    pub fn get_bband(&self) -> (f64, f64, f64) {
        match self.items.first() {
            Some(data) => (data.bband.lower(), data.bband.middle(), data.bband.upper()),
            None => (0.0, 0.0, 0.0),
        }
    }

    /// 가격이 볼린저 밴드 하한선 아래로 내려갔는지 확인
    pub fn is_below_lower_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.lower(), n)
    }

    /// n개의 연속된 캔들이 하단 밴드 위에 있는지 확인
    pub fn is_above_lower_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.lower(), n)
    }

    /// 가격이 볼린저 밴드 상한선 위로 올라갔는지 확인
    pub fn is_below_upper_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.upper(), n)
    }

    pub fn is_above_upper_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.upper(), n)
    }

    /// 가격이 볼린저 밴드 중앙선 위로 올라갔는지 확인
    pub fn is_above_middle_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.middle(), n)
    }

    /// 가격이 볼린저 밴드 중앙선 아래로 내려갔는지 확인
    pub fn is_below_middle_band(&self, n: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.middle(), n)
    }

    /// 하단 밴드 아래에서 위로 돌파한 경우 (상승 반전 신호) 확인
    pub fn is_break_through_lower_band_from_below(&self, n: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() > data.bband.lower(),
            n,
            3,
        )
    }

    /// 밴드 폭이 충분히 넓은지 확인
    pub fn is_band_width_sufficient(&self) -> bool {
        self.is_all(
            |data| {
                let band_width = (data.bband.upper() - data.bband.lower()) / data.bband.middle();
                band_width > 0.02
            },
            1,
        )
    }
}

impl<C: Candle> AnalyzerOps<BBandAnalyzerData<C>, C> for BBandAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> BBandAnalyzerData<C> {
        let bband = self.bbandbuilder.next(&candle);
        BBandAnalyzerData::new(candle, bband)
    }

    fn datum(&self) -> &Vec<BBandAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<BBandAnalyzerData<C>> {
        &mut self.items
    }
}
