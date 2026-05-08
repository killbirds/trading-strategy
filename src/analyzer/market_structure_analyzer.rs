use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::market_structure as indicator_market_structure;
pub use crate::indicator::market_structure::{
    FVGType, FairValueGap, LiquidityPool, LiquidityPoolType, MarketStructure, OrderBlock,
    OrderBlockType, StructureChange,
};
use std::fmt::Display;
use trading_chart::Candle;

/// Market Structure 분석기 데이터
#[derive(Debug)]
pub struct MarketStructureAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 현재 시장 구조
    pub market_structure: MarketStructure,
    /// 구조 변화 타입
    pub structure_change: StructureChange,
    /// Fair Value Gaps
    pub fair_value_gaps: Vec<FairValueGap>,
    /// 오더 블록들
    pub order_blocks: Vec<OrderBlock>,
    /// 유동성 풀들
    pub liquidity_pools: Vec<LiquidityPool>,
    /// 시장 흐름 강도
    pub market_flow_strength: f64,
    /// 임밸런스 정도
    pub imbalance_degree: f64,
    /// 최근 스윙 하이
    pub recent_swing_high: Option<f64>,
    /// 최근 스윙 로우
    pub recent_swing_low: Option<f64>,
}

#[derive(Debug)]
pub struct MarketStructureAnalyzerDataParams {
    pub market_structure: MarketStructure,
    pub structure_change: StructureChange,
    pub fair_value_gaps: Vec<FairValueGap>,
    pub order_blocks: Vec<OrderBlock>,
    pub liquidity_pools: Vec<LiquidityPool>,
    pub market_flow_strength: f64,
    pub imbalance_degree: f64,
    pub recent_swing_high: Option<f64>,
    pub recent_swing_low: Option<f64>,
}

impl<C: Candle> MarketStructureAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        params: MarketStructureAnalyzerDataParams,
    ) -> MarketStructureAnalyzerData<C> {
        let MarketStructureAnalyzerDataParams {
            market_structure,
            structure_change,
            fair_value_gaps,
            order_blocks,
            liquidity_pools,
            market_flow_strength,
            imbalance_degree,
            recent_swing_high,
            recent_swing_low,
        } = params;
        MarketStructureAnalyzerData {
            candle,
            market_structure,
            structure_change,
            fair_value_gaps,
            order_blocks,
            liquidity_pools,
            market_flow_strength,
            imbalance_degree,
            recent_swing_high,
            recent_swing_low,
        }
    }

    /// 상승 추세인지 확인
    pub fn is_uptrend(&self) -> bool {
        self.market_structure == MarketStructure::Uptrend
    }

    /// 하락 추세인지 확인
    pub fn is_downtrend(&self) -> bool {
        self.market_structure == MarketStructure::Downtrend
    }

    /// 횡보인지 확인
    pub fn is_sideways(&self) -> bool {
        self.market_structure == MarketStructure::Sideways
    }

    /// 구조 변화가 발생했는지 확인
    pub fn has_structure_change(&self) -> bool {
        self.structure_change != StructureChange::None
    }

    /// 불리시 구조 변화인지 확인
    pub fn is_bullish_structure_change(&self) -> bool {
        matches!(
            self.structure_change,
            StructureChange::BearishBOS | StructureChange::BullishCHoCH
        )
    }

    /// 베어리시 구조 변화인지 확인
    pub fn is_bearish_structure_change(&self) -> bool {
        matches!(
            self.structure_change,
            StructureChange::BullishBOS | StructureChange::BearishCHoCH
        )
    }

    /// 현재 가격이 Fair Value Gap 내부에 있는지 확인
    pub fn is_in_fair_value_gap(&self) -> bool {
        let current_price = self.candle.close_price();
        self.fair_value_gaps.iter().any(|fvg| {
            current_price >= fvg.start_price.min(fvg.end_price)
                && current_price <= fvg.start_price.max(fvg.end_price)
        })
    }

    /// 현재 가격이 오더 블록 내부에 있는지 확인
    pub fn is_in_order_block(&self) -> bool {
        let current_price = self.candle.close_price();
        self.order_blocks.iter().any(|ob| {
            current_price >= ob.start_price.min(ob.end_price)
                && current_price <= ob.start_price.max(ob.end_price)
        })
    }

    /// 유동성 풀 근처에 있는지 확인
    pub fn is_near_liquidity_pool(&self, threshold: f64) -> bool {
        let current_price = self.candle.close_price();
        self.liquidity_pools
            .iter()
            .any(|lp| (current_price - lp.price).abs() <= threshold)
    }

    /// 불리시 Fair Value Gap 반환
    pub fn get_bullish_fvgs(&self) -> Vec<&FairValueGap> {
        self.fair_value_gaps
            .iter()
            .filter(|fvg| fvg.gap_type == FVGType::Bullish)
            .collect()
    }

    /// 베어리시 Fair Value Gap 반환
    pub fn get_bearish_fvgs(&self) -> Vec<&FairValueGap> {
        self.fair_value_gaps
            .iter()
            .filter(|fvg| fvg.gap_type == FVGType::Bearish)
            .collect()
    }

    /// 불리시 오더 블록 반환
    pub fn get_bullish_order_blocks(&self) -> Vec<&OrderBlock> {
        self.order_blocks
            .iter()
            .filter(|ob| ob.block_type == OrderBlockType::Bullish)
            .collect()
    }

    /// 베어리시 오더 블록 반환
    pub fn get_bearish_order_blocks(&self) -> Vec<&OrderBlock> {
        self.order_blocks
            .iter()
            .filter(|ob| ob.block_type == OrderBlockType::Bearish)
            .collect()
    }

    /// 강한 시장 흐름인지 확인
    pub fn is_strong_market_flow(&self) -> bool {
        self.market_flow_strength > 0.7
    }

    /// 높은 임밸런스인지 확인
    pub fn is_high_imbalance(&self) -> bool {
        self.imbalance_degree > 0.6
    }

    /// 최근 스윙 하이를 돌파했는지 확인
    pub fn is_swing_high_broken(&self) -> bool {
        if let Some(swing_high) = self.recent_swing_high {
            self.candle.close_price() > swing_high
        } else {
            false
        }
    }

    /// 최근 스윙 로우를 돌파했는지 확인
    pub fn is_swing_low_broken(&self) -> bool {
        if let Some(swing_low) = self.recent_swing_low {
            self.candle.close_price() < swing_low
        } else {
            false
        }
    }
}

impl<C: Candle> GetCandle<C> for MarketStructureAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for MarketStructureAnalyzerData<C> {}

/// Market Structure 분석기
#[derive(Debug)]
pub struct MarketStructureAnalyzer<C: Candle> {
    /// 분석 데이터 히스토리
    pub items: Vec<MarketStructureAnalyzerData<C>>,
    /// 스윙 포인트 감지를 위한 강도
    pub swing_strength: usize,
    /// 구조 변화 감지를 위한 기간
    pub structure_period: usize,
    /// FVG 최소 크기
    pub min_fvg_size: f64,
    /// 오더 블록 최소 크기
    pub min_order_block_size: f64,
}

impl<C: Candle> Display for MarketStructureAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "MarketStructureAnalyzer {{ candle: {}, structure: {:?}, change: {:?}, flow: {:.2} }}",
                first.candle,
                first.market_structure,
                first.structure_change,
                first.market_flow_strength
            )
        } else {
            write!(f, "MarketStructureAnalyzer {{ no data }}")
        }
    }
}

impl<C: Candle + Clone + 'static> MarketStructureAnalyzer<C> {
    /// 새 Market Structure 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        swing_strength: usize,
        structure_period: usize,
        min_fvg_size: f64,
        min_order_block_size: f64,
    ) -> MarketStructureAnalyzer<C> {
        let mut analyzer = MarketStructureAnalyzer {
            items: Vec::new(),
            swing_strength,
            structure_period,
            min_fvg_size,
            min_order_block_size,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> MarketStructureAnalyzer<C> {
        Self::new(storage, 3, 20, 0.5, 1.0)
    }

    /// 시장 구조 분석
    fn analyze_market_structure(&self, candles: &[C]) -> MarketStructure {
        indicator_market_structure::analyze_market_structure(
            candles,
            self.swing_strength,
            self.structure_period,
        )
    }

    /// 구조 변화 감지
    fn detect_structure_change(
        &self,
        candles: &[C],
        current_structure: MarketStructure,
    ) -> StructureChange {
        let previous_structure = self.items.first().map(|item| &item.market_structure);
        indicator_market_structure::detect_structure_change(
            candles,
            previous_structure,
            current_structure,
        )
    }

    /// Fair Value Gap 식별
    fn identify_fair_value_gaps(&self, candles: &[C]) -> Vec<FairValueGap> {
        indicator_market_structure::identify_fair_value_gaps(candles, self.min_fvg_size)
    }

    /// 오더 블록 식별
    fn identify_order_blocks(&self, candles: &[C]) -> Vec<OrderBlock> {
        indicator_market_structure::identify_order_blocks(
            candles,
            self.swing_strength,
            self.min_order_block_size,
        )
    }

    /// 유동성 풀 식별
    fn identify_liquidity_pools(&self, candles: &[C]) -> Vec<LiquidityPool> {
        indicator_market_structure::identify_liquidity_pools(candles, self.swing_strength)
    }

    /// 시장 흐름 강도 계산
    fn calculate_market_flow_strength(&self, candles: &[C]) -> f64 {
        indicator_market_structure::calculate_market_flow_strength(candles)
    }

    /// 임밸런스 정도 계산
    fn calculate_imbalance_degree(&self, candles: &[C]) -> f64 {
        indicator_market_structure::calculate_imbalance_degree(candles)
    }

    /// 최근 스윙 포인트 반환
    fn get_recent_swing_points(&self, candles: &[C]) -> (Option<f64>, Option<f64>) {
        indicator_market_structure::get_recent_swing_points(candles, self.swing_strength)
    }

    /// 시장 구조 강도 확인
    pub fn is_strong_structure(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_strong_market_flow() && !data.is_sideways()
        } else {
            false
        }
    }

    /// 구조 변화 신호 확인
    pub fn is_structure_change_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.has_structure_change() && data.is_strong_market_flow()
        } else {
            false
        }
    }

    /// 유동성 사냥 신호 확인
    pub fn is_liquidity_hunt_signal(&self) -> bool {
        if let Some(data) = self.items.first() {
            data.is_near_liquidity_pool(1.0) && data.is_high_imbalance()
        } else {
            false
        }
    }

    /// 상승 추세 구조 신호 확인 (n개 연속 상승 추세, 이전 m개는 아님)
    pub fn is_uptrend_structure_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_uptrend(), n, m, p)
    }

    /// 하락 추세 구조 신호 확인 (n개 연속 하락 추세, 이전 m개는 아님)
    pub fn is_downtrend_structure_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_downtrend(), n, m, p)
    }

    /// 구조 변화 돌파 신호 확인 (n개 연속 구조 변화, 이전 m개는 아님)
    pub fn is_structure_change_breakthrough(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.has_structure_change(), n, m, p)
    }

    /// 강한 시장 구조 신호 확인 (n개 연속 강한 시장 구조, 이전 m개는 아님)
    pub fn is_strong_market_structure_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_strong_market_flow(), n, m, p)
    }

    /// 불리시 구조 변화 신호 확인 (n개 연속 불리시 구조 변화, 이전 m개는 아님)
    pub fn is_bullish_structure_change_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_bullish_structure_change(), n, m, p)
    }

    /// 베어리시 구조 변화 신호 확인 (n개 연속 베어리시 구조 변화, 이전 m개는 아님)
    pub fn is_bearish_structure_change_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_bearish_structure_change(), n, m, p)
    }

    /// 높은 임밸런스 신호 확인 (n개 연속 높은 임밸런스, 이전 m개는 아님)
    pub fn is_high_imbalance_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_imbalance(), n, m, p)
    }

    /// 유동성 사냥 돌파 신호 확인 (n개 연속 유동성 풀 근접, 이전 m개는 아님)
    pub fn is_liquidity_hunt_breakthrough(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_near_liquidity_pool(threshold), n, m, p)
    }

    /// 오더 블록 신호 확인 (n개 연속 오더 블록 내부, 이전 m개는 아님)
    pub fn is_order_block_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_in_order_block(), n, m, p)
    }

    /// Fair Value Gap 신호 확인 (n개 연속 FVG 내부, 이전 m개는 아님)
    pub fn is_fair_value_gap_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_in_fair_value_gap(), n, m, p)
    }

    /// 스윙 하이 돌파 신호 확인 (n개 연속 스윙 하이 돌파, 이전 m개는 아님)
    pub fn is_swing_high_break_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_swing_high_broken(), n, m, p)
    }

    /// 스윙 로우 돌파 신호 확인 (n개 연속 스윙 로우 돌파, 이전 m개는 아님)
    pub fn is_swing_low_break_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_swing_low_broken(), n, m, p)
    }

    /// n개의 연속 데이터에서 상승 추세인지 확인
    pub fn is_uptrend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_uptrend(), n, p)
    }

    /// n개의 연속 데이터에서 하락 추세인지 확인
    pub fn is_downtrend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_downtrend(), n, p)
    }

    /// n개의 연속 데이터에서 횡보인지 확인
    pub fn is_sideways(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_sideways(), n, p)
    }

    /// n개의 연속 데이터에서 강한 시장 구조인지 확인
    pub fn is_strong_market_flow(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_strong_market_flow(), n, p)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<MarketStructureAnalyzerData<C>, C>
    for MarketStructureAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> MarketStructureAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = self.structure_period.max(50);
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 분석 수행
        let market_structure = self.analyze_market_structure(&recent_candles);
        let structure_change =
            self.detect_structure_change(&recent_candles, market_structure.clone());
        let fair_value_gaps = self.identify_fair_value_gaps(&recent_candles);
        let order_blocks = self.identify_order_blocks(&recent_candles);
        let liquidity_pools = self.identify_liquidity_pools(&recent_candles);
        let market_flow_strength = self.calculate_market_flow_strength(&recent_candles);
        let imbalance_degree = self.calculate_imbalance_degree(&recent_candles);
        let (recent_swing_high, recent_swing_low) = self.get_recent_swing_points(&recent_candles);

        MarketStructureAnalyzerData::new(
            candle,
            MarketStructureAnalyzerDataParams {
                market_structure,
                structure_change,
                fair_value_gaps,
                order_blocks,
                liquidity_pools,
                market_flow_strength,
                imbalance_degree,
                recent_swing_high,
                recent_swing_low,
            },
        )
    }

    fn items(&self) -> &Vec<MarketStructureAnalyzerData<C>> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Vec<MarketStructureAnalyzerData<C>> {
        &mut self.items
    }
}
