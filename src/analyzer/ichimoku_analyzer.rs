use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::ichimoku::{
    IchimokuParams, Ichimokus, IchimokusBuilder, IchimokusBuilderFactory,
};
use std::fmt::Display;
use trading_chart::Candle;

/// 일목균형표 전략 데이터
#[derive(Debug)]
pub struct IchimokuAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 일목균형표 지표 집합
    pub ichimokus: Ichimokus,
}

impl<C: Candle> IchimokuAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, ichimokus: Ichimokus) -> IchimokuAnalyzerData<C> {
        IchimokuAnalyzerData { candle, ichimokus }
    }

    /// 현재 가격이 구름 위에 있는지 확인
    pub fn is_price_above_cloud(&self, param: &IchimokuParams) -> bool {
        let price = self.candle.close_price();
        self.ichimokus.get(param).is_price_above_cloud(price)
    }

    /// 현재 가격이 구름 아래에 있는지 확인
    pub fn is_price_below_cloud(&self, param: &IchimokuParams) -> bool {
        let price = self.candle.close_price();
        self.ichimokus.get(param).is_price_below_cloud(price)
    }

    /// 현재 가격이 구름 내에 있는지 확인
    pub fn is_price_in_cloud(&self, param: &IchimokuParams) -> bool {
        let price = self.candle.close_price();
        self.ichimokus.get(param).is_price_in_cloud(price)
    }

    /// 전환선이 기준선 위에 있는지 확인 (골든 크로스 상태)
    pub fn is_tenkan_above_kijun(&self, param: &IchimokuParams) -> bool {
        self.ichimokus.get(param).is_tenkan_above_kijun()
    }

    /// 전환선이 기준선 아래에 있는지 확인 (데드 크로스 상태)
    pub fn is_tenkan_below_kijun(&self, param: &IchimokuParams) -> bool {
        self.ichimokus.get(param).is_tenkan_below_kijun()
    }

    /// 구름이 상승 트렌드인지 확인
    pub fn is_bullish_cloud(&self, param: &IchimokuParams) -> bool {
        self.ichimokus.get(param).is_bullish_cloud()
    }

    /// 구름이 하락 트렌드인지 확인
    pub fn is_bearish_cloud(&self, param: &IchimokuParams) -> bool {
        self.ichimokus.get(param).is_bearish_cloud()
    }

    /// 구름의 두께 반환
    pub fn cloud_thickness(&self, param: &IchimokuParams) -> f64 {
        self.ichimokus.get(param).cloud_thickness()
    }

    /// 매수 신호 여부 확인 (강한 상승 트렌드)
    pub fn is_buy_signal(&self, param: &IchimokuParams) -> bool {
        let ichimoku = self.ichimokus.get(param);
        let price = self.candle.close_price();

        ichimoku.is_price_above_cloud(price)
            && ichimoku.is_bullish_cloud()
            && ichimoku.is_tenkan_above_kijun()
    }

    /// 매도 신호 여부 확인 (강한 하락 트렌드)
    pub fn is_sell_signal(&self, param: &IchimokuParams) -> bool {
        let ichimoku = self.ichimokus.get(param);
        let price = self.candle.close_price();

        ichimoku.is_price_below_cloud(price)
            && ichimoku.is_bearish_cloud()
            && ichimoku.is_tenkan_below_kijun()
    }
}

impl<C: Candle> GetCandle<C> for IchimokuAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for IchimokuAnalyzerData<C> {}

/// 일목균형표 전략 컨텍스트
#[derive(Debug)]
pub struct IchimokuAnalyzer<C: Candle> {
    /// 일목균형표 빌더
    pub ichimokusbuilder: IchimokusBuilder<C>,
    /// 파라미터 리스트
    pub params: Vec<IchimokuParams>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<IchimokuAnalyzerData<C>>,
}

impl<C: Candle> Display for IchimokuAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Ichimokus: {}", first.candle, first.ichimokus),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> IchimokuAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(params: &[IchimokuParams], storage: &CandleStore<C>) -> IchimokuAnalyzer<C> {
        let ichimokusbuilder = IchimokusBuilderFactory::build::<C>(params);
        let mut ctx = IchimokuAnalyzer {
            ichimokusbuilder,
            params: params.to_vec(),
            items: vec![],
        };
        ctx.init_from_storage(storage);
        ctx
    }

    /// 기본 매개변수로 새 전략 컨텍스트 생성
    pub fn default(storage: &CandleStore<C>) -> IchimokuAnalyzer<C> {
        let params = vec![IchimokuParams {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_period: 52,
        }];
        Self::new(&params, storage)
    }

    /// 현재 가격이 구름 위에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_above_cloud(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_above_cloud(param), n, p)
    }

    /// 현재 가격이 구름 아래에 있는지 n개의 연속 데이터에서 확인
    pub fn is_price_below_cloud(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_price_below_cloud(param), n, p)
    }

    /// 전환선이 기준선 위에 있는지 n개의 연속 데이터에서 확인
    pub fn is_tenkan_above_kijun(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_tenkan_above_kijun(param), n, p)
    }

    /// 골든 크로스 발생 확인 (전환선이 기준선을 상향 돌파)
    pub fn is_golden_cross(&self, param: &IchimokuParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = self.items[0].is_tenkan_above_kijun(param);
        let previous = self.items[1].is_tenkan_above_kijun(param);

        current && !previous
    }

    /// 데드 크로스 발생 확인 (전환선이 기준선을 하향 돌파)
    pub fn is_dead_cross(&self, param: &IchimokuParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = self.items[0].is_tenkan_below_kijun(param);
        let previous = self.items[1].is_tenkan_below_kijun(param);

        current && !previous
    }

    /// 구름 돌파 발생 확인 (가격이 구름을 상향 돌파)
    pub fn is_cloud_breakout_up(&self, param: &IchimokuParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = self.items[0].is_price_above_cloud(param);
        let previous = !self.items[1].is_price_above_cloud(param);

        current && previous
    }

    /// 구름 붕괴 발생 확인 (가격이 구름을 하향 돌파)
    pub fn is_cloud_breakdown(&self, param: &IchimokuParams) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = self.items[0].is_price_below_cloud(param);
        let previous = !self.items[1].is_price_below_cloud(param);

        current && previous
    }

    /// 매수 신호 여부 확인 (강한 상승 트렌드)
    pub fn is_buy_signal(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_buy_signal(param), n, p)
    }

    /// 매도 신호 여부 확인 (강한 하락 트렌드)
    pub fn is_sell_signal(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_sell_signal(param), n, p)
    }

    /// 구름 두께 변화율 확인 (구름이 두꺼워지는지 확인)
    pub fn is_cloud_thickening(&self, param: &IchimokuParams, n: usize, p: usize) -> bool {
        if self.items.len() < n + p + 1 {
            return false;
        }

        for i in p..p + n {
            let current = self.items[i].cloud_thickness(param).abs();
            let previous = self.items[i + 1].cloud_thickness(param).abs();
            if current <= previous {
                return false;
            }
        }

        true
    }

    /// 골든 크로스 신호 확인 (n개 연속 골든 크로스, 이전 m개는 아님)
    pub fn is_golden_cross_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_golden_cross(param), n, m, p)
    }

    /// 데드 크로스 신호 확인 (n개 연속 데드 크로스, 이전 m개는 아님)
    pub fn is_dead_cross_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_dead_cross(param), n, m, p)
    }

    /// 구름 상향 돌파 신호 확인 (n개 연속 구름 상향 돌파, 이전 m개는 아님)
    pub fn is_cloud_breakout_up_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_cloud_breakout_up(param), n, m, p)
    }

    /// 구름 하향 돌파 신호 확인 (n개 연속 구름 하향 돌파, 이전 m개는 아님)
    pub fn is_cloud_breakdown_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|_| self.is_cloud_breakdown(param), n, m, p)
    }

    /// 매수 신호 확인 (n개 연속 매수 신호, 이전 m개는 아님)
    pub fn is_buy_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_buy_signal(param), n, m, p)
    }

    /// 매도 신호 확인 (n개 연속 매도 신호, 이전 m개는 아님)
    pub fn is_sell_signal_confirmed(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_sell_signal(param), n, m, p)
    }

    /// 구름 위 신호 확인 (n개 연속 가격이 구름 위, 이전 m개는 아님)
    pub fn is_price_above_cloud_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_above_cloud(param), n, m, p)
    }

    /// 구름 아래 신호 확인 (n개 연속 가격이 구름 아래, 이전 m개는 아님)
    pub fn is_price_below_cloud_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_price_below_cloud(param), n, m, p)
    }

    /// 전환선이 기준선 위 신호 확인 (n개 연속 전환선 > 기준선, 이전 m개는 아님)
    pub fn is_tenkan_above_kijun_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_tenkan_above_kijun(param), n, m, p)
    }

    /// 전환선이 기준선 아래 신호 확인 (n개 연속 전환선 < 기준선, 이전 m개는 아님)
    pub fn is_tenkan_below_kijun_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_tenkan_below_kijun(param), n, m, p)
    }

    /// 강한 상승 신호 확인 (n개 연속 강한 상승, 이전 m개는 아님)
    pub fn is_strong_bullish_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                // 강한 상승 신호 = 매수 신호 + 가격이 구름 위 + 전환선 > 기준선
                data.is_buy_signal(param)
                    && data.is_price_above_cloud(param)
                    && data.is_tenkan_above_kijun(param)
            },
            n,
            m,
            p,
        )
    }

    /// 강한 하락 신호 확인 (n개 연속 강한 하락, 이전 m개는 아님)
    pub fn is_strong_bearish_signal(
        &self,
        n: usize,
        m: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                // 강한 하락 신호 = 매도 신호 + 가격이 구름 아래 + 전환선 < 기준선
                data.is_sell_signal(param)
                    && data.is_price_below_cloud(param)
                    && data.is_tenkan_below_kijun(param)
            },
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 매수 신호인지 확인
    pub fn is_buy_signal_continuous(&self, n: usize, param: &IchimokuParams, p: usize) -> bool {
        self.is_all(|data| data.is_buy_signal(param), n, p)
    }

    /// n개의 연속 데이터에서 매도 신호인지 확인
    pub fn is_sell_signal_continuous(&self, n: usize, param: &IchimokuParams, p: usize) -> bool {
        self.is_all(|data| data.is_sell_signal(param), n, p)
    }

    /// n개의 연속 데이터에서 가격이 구름 위인지 확인
    pub fn is_price_above_cloud_continuous(
        &self,
        n: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_all(|data| data.is_price_above_cloud(param), n, p)
    }

    /// n개의 연속 데이터에서 가격이 구름 아래인지 확인
    pub fn is_price_below_cloud_continuous(
        &self,
        n: usize,
        param: &IchimokuParams,
        p: usize,
    ) -> bool {
        self.is_all(|data| data.is_price_below_cloud(param), n, p)
    }
}

impl<C: Candle> AnalyzerOps<IchimokuAnalyzerData<C>, C> for IchimokuAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> IchimokuAnalyzerData<C> {
        let ichimokus = self.ichimokusbuilder.next(&candle);
        IchimokuAnalyzerData::new(candle, ichimokus)
    }

    fn datum(&self) -> &Vec<IchimokuAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<IchimokuAnalyzerData<C>> {
        &mut self.items
    }
}
