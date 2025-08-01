use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::vwap::{VWAPParams, VWAPs, VWAPsBuilder, VWAPsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// VWAP 전략 데이터
#[derive(Debug)]
pub struct VWAPAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// VWAP 지표 집합
    pub vwaps: VWAPs,
}

impl<C: Candle> VWAPAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, vwaps: VWAPs) -> VWAPAnalyzerData<C> {
        VWAPAnalyzerData { candle, vwaps }
    }

    /// 현재 가격이 VWAP 위에 있는지 확인
    pub fn is_price_above_vwap(&self, param: &VWAPParams) -> bool {
        let price = self.candle.close_price();
        self.vwaps.get(param).is_price_above(price)
    }

    /// 현재 가격이 VWAP 아래에 있는지 확인
    pub fn is_price_below_vwap(&self, param: &VWAPParams) -> bool {
        let price = self.candle.close_price();
        self.vwaps.get(param).is_price_below(price)
    }

    /// 현재 가격과 VWAP의 거리(백분율)
    pub fn price_to_vwap_percent(&self, param: &VWAPParams) -> f64 {
        let price = self.candle.close_price();
        self.vwaps.get(param).price_to_vwap_percent(price)
    }

    /// 모든 가격이 모든 VWAP 위에 있는지 확인
    pub fn is_price_above_all_vwaps(&self) -> bool {
        let price = self.candle.close_price();
        self.vwaps
            .get_all()
            .iter()
            .all(|vwap| vwap.is_price_above(price))
    }

    /// 모든 가격이 모든 VWAP 아래에 있는지 확인
    pub fn is_price_below_all_vwaps(&self) -> bool {
        let price = self.candle.close_price();
        self.vwaps
            .get_all()
            .iter()
            .all(|vwap| vwap.is_price_below(price))
    }

    /// 현재 가격이 VWAP에 매우 가까운지 확인
    pub fn is_price_near_vwap(&self, param: &VWAPParams, threshold: f64) -> bool {
        let percent = self.price_to_vwap_percent(param).abs();
        percent < threshold
    }

    /// 현재 가격이 VWAP에서 크게 벗어났는지 확인
    pub fn is_price_far_from_vwap(&self, param: &VWAPParams, threshold: f64) -> bool {
        let percent = self.price_to_vwap_percent(param).abs();
        percent > threshold
    }
}

impl<C: Candle> GetCandle<C> for VWAPAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for VWAPAnalyzerData<C> {}

/// VWAP 전략 컨텍스트
#[derive(Debug)]
pub struct VWAPAnalyzer<C: Candle> {
    /// VWAP 빌더
    pub vwapsbuilder: VWAPsBuilder<C>,
    /// 파라미터 리스트
    pub params: Vec<VWAPParams>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<VWAPAnalyzerData<C>>,
}

impl<C: Candle> Display for VWAPAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, VWAPs: {}", first.candle, first.vwaps),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> VWAPAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(params: &[VWAPParams], storage: &CandleStore<C>) -> VWAPAnalyzer<C> {
        let vwapsbuilder = VWAPsBuilderFactory::build::<C>(params);
        let mut ctx = VWAPAnalyzer {
            vwapsbuilder,
            params: params.to_vec(),
            items: vec![],
        };
        ctx.init_from_storage(storage);
        ctx
    }

    /// 기본 매개변수로 새 전략 컨텍스트 생성
    pub fn default(storage: &CandleStore<C>) -> VWAPAnalyzer<C> {
        let params = vec![VWAPParams::default()];
        Self::new(&params, storage)
    }

    /// 현재 가격이 VWAP 위에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_above_vwap(&self, param: &VWAPParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_above_vwap(param), n, p)
    }

    /// 현재 가격이 VWAP 아래에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_below_vwap(&self, param: &VWAPParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_below_vwap(param), n, p)
    }

    /// 현재 가격이 모든 VWAP 위에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_above_all_vwaps(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_above_all_vwaps(), n, p)
    }

    /// 현재 가격이 모든 VWAP 아래에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_below_all_vwaps(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_below_all_vwaps(), n, p)
    }

    /// VWAP 돌파 확인 (가격이 VWAP 위로 이동)
    pub fn is_vwap_breakout_up(&self, param: &VWAPParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_above = self.items[0].is_price_above_vwap(param);
        let previous_above = self.items[1].is_price_above_vwap(param);

        current_above && !previous_above
    }

    /// VWAP 붕괴 확인 (가격이 VWAP 아래로 이동)
    pub fn is_vwap_breakdown(&self, param: &VWAPParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_below = self.items[0].is_price_below_vwap(param);
        let previous_below = self.items[1].is_price_below_vwap(param);

        current_below && !previous_below
    }

    /// VWAP 리바운드 확인 (가격이 VWAP에 닿고 반등)
    pub fn is_vwap_rebound(&self, param: &VWAPParams, threshold: f64) -> bool {
        if self.items.len() < 3 {
            return false;
        }

        // 현재 VWAP에서 멀어지고 있는지
        let current_percent = self.items[0].price_to_vwap_percent(param);
        let previous_percent = self.items[1].price_to_vwap_percent(param);
        let more_previous_percent = self.items[2].price_to_vwap_percent(param);

        // 상승 반등: VWAP 아래에서 VWAP에 가까워졌다가 다시 상승
        let up_rebound = current_percent > previous_percent
            && previous_percent.abs() < threshold
            && more_previous_percent < previous_percent;

        // 하락 반등: VWAP 위에서 VWAP에 가까워졌다가 다시 하락
        let down_rebound = current_percent < previous_percent
            && previous_percent.abs() < threshold
            && more_previous_percent > previous_percent;

        up_rebound || down_rebound
    }

    /// 가격과 VWAP의 거리가 점점 벌어지는지 확인 (VWAP에서 점점 멀어짐)
    pub fn is_diverging_from_vwap(&self, param: &VWAPParams, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        for i in 0..n - 1 {
            let current_percent = self.items[i].price_to_vwap_percent(param).abs();
            let next_percent = self.items[i + 1].price_to_vwap_percent(param).abs();
            if current_percent <= next_percent {
                return false;
            }
        }

        true
    }

    /// 가격과 VWAP의 거리가 점점 좁아지는지 확인 (VWAP로 회귀)
    pub fn is_converging_to_vwap(&self, param: &VWAPParams, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        for i in 0..n - 1 {
            let current_percent = self.items[i].price_to_vwap_percent(param).abs();
            let next_percent = self.items[i + 1].price_to_vwap_percent(param).abs();
            if current_percent >= next_percent {
                return false;
            }
        }

        true
    }

    /// VWAP 위 신호 확인 (n개 연속 가격 > VWAP, 이전 m개는 아님)
    pub fn is_price_above_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_above_vwap(param), n, m, p)
    }

    /// VWAP 아래 신호 확인 (n개 연속 가격 < VWAP, 이전 m개는 아님)
    pub fn is_price_below_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_below_vwap(param), n, m, p)
    }

    /// 모든 VWAP 위 신호 확인 (n개 연속 모든 VWAP 위, 이전 m개는 아님)
    pub fn is_price_above_all_vwaps_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_above_all_vwaps(), n, m, p)
    }

    /// 모든 VWAP 아래 신호 확인 (n개 연속 모든 VWAP 아래, 이전 m개는 아님)
    pub fn is_price_below_all_vwaps_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_below_all_vwaps(), n, m, p)
    }

    /// VWAP 근처 신호 확인 (n개 연속 VWAP 근처, 이전 m개는 아님)
    pub fn is_price_near_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_price_near_vwap(param, threshold),
            n,
            m,
            p,
        )
    }

    /// VWAP에서 멀리 신호 확인 (n개 연속 VWAP에서 멀리, 이전 m개는 아님)
    pub fn is_price_far_from_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_price_far_from_vwap(param, threshold),
            n,
            m,
            p,
        )
    }

    /// VWAP 상향 돌파 신호 확인 (n개 연속 상향 돌파, 이전 m개는 아님)
    pub fn is_vwap_breakout_up_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_vwap_breakout_up(param), n, m, p)
    }

    /// VWAP 하향 돌파 신호 확인 (n개 연속 하향 돌파, 이전 m개는 아님)
    pub fn is_vwap_breakdown_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_vwap_breakdown(param), n, m, p)
    }

    /// VWAP 리바운드 신호 확인 (n개 연속 리바운드, 이전 m개는 아님)
    pub fn is_vwap_rebound_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_vwap_rebound(param, threshold), n, m, p)
    }

    /// VWAP에서 발산 신호 확인 (n개 연속 발산, 이전 m개는 아님)
    pub fn is_diverging_from_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        diverge_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.is_diverging_from_vwap(param, diverge_period),
            n,
            m,
            p,
        )
    }

    /// VWAP로 수렴 신호 확인 (n개 연속 수렴, 이전 m개는 아님)
    pub fn is_converging_to_vwap_signal(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        converge_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| self.is_converging_to_vwap(param, converge_period),
            n,
            m,
            p,
        )
    }

    /// VWAP 가격 비율 임계값 돌파 신호 확인 (n개 연속 임계값 초과, 이전 m개는 아님)
    pub fn is_vwap_price_ratio_breakthrough(
        &self,
        n: usize,
        m: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.price_to_vwap_percent(param).abs() > threshold,
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 가격이 VWAP 위인지 확인
    pub fn is_price_above_vwap_continuous(&self, n: usize, param: &VWAPParams, p: usize) -> bool {
        self.is_all(|data| data.is_price_above_vwap(param), n, p)
    }

    /// n개의 연속 데이터에서 가격이 VWAP 아래인지 확인
    pub fn is_price_below_vwap_continuous(&self, n: usize, param: &VWAPParams, p: usize) -> bool {
        self.is_all(|data| data.is_price_below_vwap(param), n, p)
    }

    /// n개의 연속 데이터에서 가격이 모든 VWAP 위인지 확인
    pub fn is_price_above_all_vwaps_continuous(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_above_all_vwaps(), n, p)
    }

    /// n개의 연속 데이터에서 가격이 모든 VWAP 아래인지 확인
    pub fn is_price_below_all_vwaps_continuous(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_below_all_vwaps(), n, p)
    }

    /// n개의 연속 데이터에서 가격이 VWAP 근처인지 확인
    pub fn is_price_near_vwap(
        &self,
        n: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_all(|data| data.is_price_near_vwap(param, threshold), n, p)
    }

    /// n개의 연속 데이터에서 가격이 VWAP에서 멀리 있는지 확인
    pub fn is_price_far_from_vwap(
        &self,
        n: usize,
        param: &VWAPParams,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_all(|data| data.is_price_far_from_vwap(param, threshold), n, p)
    }
}

impl<C: Candle> AnalyzerOps<VWAPAnalyzerData<C>, C> for VWAPAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> VWAPAnalyzerData<C> {
        let vwaps = self.vwapsbuilder.next(&candle);
        VWAPAnalyzerData::new(candle, vwaps)
    }

    fn datum(&self) -> &Vec<VWAPAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<VWAPAnalyzerData<C>> {
        &mut self.items
    }
}
