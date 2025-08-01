use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 시장 구조 타입
#[derive(Debug, Clone, PartialEq)]
pub enum MarketStructure {
    /// 상승 추세 (Higher Highs, Higher Lows)
    Uptrend,
    /// 하락 추세 (Lower Highs, Lower Lows)
    Downtrend,
    /// 횡보 (Sideways)
    Sideways,
    /// 불확실 (Uncertain)
    Uncertain,
}

/// 구조 변화 타입
#[derive(Debug, Clone, PartialEq)]
pub enum StructureChange {
    /// 구조 변화 없음
    None,
    /// 상승 구조 파괴 (Break of Structure - BOS to Downtrend)
    BullishBOS,
    /// 하락 구조 파괴 (Break of Structure - BOS to Uptrend)
    BearishBOS,
    /// 상승 성격 변화 (Change of Character - CHoCH to Uptrend)
    BullishCHoCH,
    /// 하락 성격 변화 (Change of Character - CHoCH to Downtrend)
    BearishCHoCH,
}

/// Fair Value Gap (FVG) 타입
#[derive(Debug, Clone)]
pub struct FairValueGap {
    /// 갭 시작 가격
    pub start_price: f64,
    /// 갭 종료 가격
    pub end_price: f64,
    /// 갭 타입 (불리시/베어리시)
    pub gap_type: FVGType,
    /// 갭 생성 인덱스
    pub index: usize,
    /// 갭 크기
    pub size: f64,
}

/// Fair Value Gap 타입
#[derive(Debug, Clone, PartialEq)]
pub enum FVGType {
    /// 불리시 FVG (상승 갭)
    Bullish,
    /// 베어리시 FVG (하락 갭)
    Bearish,
}

/// 오더 블록 타입
#[derive(Debug, Clone)]
pub struct OrderBlock {
    /// 오더 블록 시작 가격
    pub start_price: f64,
    /// 오더 블록 종료 가격
    pub end_price: f64,
    /// 오더 블록 타입
    pub block_type: OrderBlockType,
    /// 오더 블록 생성 인덱스
    pub index: usize,
    /// 강도 (터치 횟수)
    pub strength: usize,
}

/// 오더 블록 타입
#[derive(Debug, Clone, PartialEq)]
pub enum OrderBlockType {
    /// 불리시 오더 블록 (수요 구역)
    Bullish,
    /// 베어리시 오더 블록 (공급 구역)
    Bearish,
}

/// 유동성 풀 타입
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    /// 유동성 풀 가격
    pub price: f64,
    /// 유동성 풀 타입
    pub pool_type: LiquidityPoolType,
    /// 유동성 풀 생성 인덱스
    pub index: usize,
    /// 유동성 양
    pub liquidity_amount: f64,
}

/// 유동성 풀 타입
#[derive(Debug, Clone, PartialEq)]
pub enum LiquidityPoolType {
    /// 매수 유동성 풀 (Buy Side Liquidity)
    BuyLiquidity,
    /// 매도 유동성 풀 (Sell Side Liquidity)
    SellLiquidity,
}

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

impl<C: Candle> MarketStructureAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        market_structure: MarketStructure,
        structure_change: StructureChange,
        fair_value_gaps: Vec<FairValueGap>,
        order_blocks: Vec<OrderBlock>,
        liquidity_pools: Vec<LiquidityPool>,
        market_flow_strength: f64,
        imbalance_degree: f64,
        recent_swing_high: Option<f64>,
        recent_swing_low: Option<f64>,
    ) -> MarketStructureAnalyzerData<C> {
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
        if candles.len() < self.structure_period {
            return MarketStructure::Uncertain;
        }

        let swing_points = self.identify_swing_points(candles);
        if swing_points.len() < 4 {
            return MarketStructure::Uncertain;
        }

        let highs: Vec<f64> = swing_points
            .iter()
            .filter_map(|(_, price, is_high)| if *is_high { Some(*price) } else { None })
            .collect();
        let lows: Vec<f64> = swing_points
            .iter()
            .filter_map(|(_, price, is_high)| if !*is_high { Some(*price) } else { None })
            .collect();

        if highs.len() < 2 || lows.len() < 2 {
            return MarketStructure::Uncertain;
        }

        let higher_highs = highs.windows(2).all(|w| w[0] > w[1]);
        let higher_lows = lows.windows(2).all(|w| w[0] > w[1]);
        let lower_highs = highs.windows(2).all(|w| w[0] < w[1]);
        let lower_lows = lows.windows(2).all(|w| w[0] < w[1]);

        if higher_highs && higher_lows {
            MarketStructure::Uptrend
        } else if lower_highs && lower_lows {
            MarketStructure::Downtrend
        } else {
            MarketStructure::Sideways
        }
    }

    /// 구조 변화 감지
    fn detect_structure_change(
        &self,
        candles: &[C],
        current_structure: MarketStructure,
    ) -> StructureChange {
        if self.items.is_empty() {
            return StructureChange::None;
        }

        let previous_structure = &self.items[0].market_structure;

        match (previous_structure, current_structure) {
            (MarketStructure::Uptrend, MarketStructure::Downtrend) => {
                // 상승 추세에서 하락 추세로 전환
                if self.is_strong_reversal(candles) {
                    StructureChange::BullishBOS
                } else {
                    StructureChange::BearishCHoCH
                }
            }
            (MarketStructure::Downtrend, MarketStructure::Uptrend) => {
                // 하락 추세에서 상승 추세로 전환
                if self.is_strong_reversal(candles) {
                    StructureChange::BearishBOS
                } else {
                    StructureChange::BullishCHoCH
                }
            }
            _ => StructureChange::None,
        }
    }

    /// 강한 반전인지 확인
    fn is_strong_reversal(&self, candles: &[C]) -> bool {
        if candles.len() < 10 {
            return false;
        }

        let recent_volume: f64 = candles.iter().take(5).map(|c| c.volume()).sum();
        let avg_volume: f64 = candles
            .iter()
            .skip(5)
            .take(10)
            .map(|c| c.volume())
            .sum::<f64>()
            / 10.0;

        recent_volume > avg_volume * 1.5
    }

    /// 스윙 포인트 식별
    fn identify_swing_points(&self, candles: &[C]) -> Vec<(usize, f64, bool)> {
        let mut swing_points = Vec::new();
        let strength = self.swing_strength;

        if candles.len() < strength * 2 + 1 {
            return swing_points;
        }

        for i in strength..candles.len() - strength {
            let current = &candles[i];

            // 스윙 하이 확인
            let is_swing_high = (i.saturating_sub(strength)..i)
                .chain((i + 1)..(i + strength + 1).min(candles.len()))
                .all(|j| current.high_price() > candles[j].high_price());

            // 스윙 로우 확인
            let is_swing_low = (i.saturating_sub(strength)..i)
                .chain((i + 1)..(i + strength + 1).min(candles.len()))
                .all(|j| current.low_price() < candles[j].low_price());

            if is_swing_high {
                swing_points.push((i, current.high_price(), true));
            }
            if is_swing_low {
                swing_points.push((i, current.low_price(), false));
            }
        }

        swing_points
    }

    /// Fair Value Gap 식별
    fn identify_fair_value_gaps(&self, candles: &[C]) -> Vec<FairValueGap> {
        let mut fvgs = Vec::new();

        if candles.len() < 3 {
            return fvgs;
        }

        for i in 0..candles.len() - 2 {
            let candle1 = &candles[i + 2];
            let _candle2 = &candles[i + 1];
            let candle3 = &candles[i];

            // 불리시 FVG 확인
            if candle1.high_price() < candle3.low_price() {
                let gap_size = candle3.low_price() - candle1.high_price();
                if gap_size >= self.min_fvg_size {
                    fvgs.push(FairValueGap {
                        start_price: candle1.high_price(),
                        end_price: candle3.low_price(),
                        gap_type: FVGType::Bullish,
                        index: i,
                        size: gap_size,
                    });
                }
            }

            // 베어리시 FVG 확인
            if candle1.low_price() > candle3.high_price() {
                let gap_size = candle1.low_price() - candle3.high_price();
                if gap_size >= self.min_fvg_size {
                    fvgs.push(FairValueGap {
                        start_price: candle1.low_price(),
                        end_price: candle3.high_price(),
                        gap_type: FVGType::Bearish,
                        index: i,
                        size: gap_size,
                    });
                }
            }
        }

        fvgs
    }

    /// 오더 블록 식별
    fn identify_order_blocks(&self, candles: &[C]) -> Vec<OrderBlock> {
        let mut order_blocks = Vec::new();
        let swing_points = self.identify_swing_points(candles);

        for (index, _price, is_high) in swing_points {
            if index >= candles.len() {
                continue;
            }

            let candle = &candles[index];
            let block_size = candle.high_price() - candle.low_price();

            if block_size >= self.min_order_block_size {
                let block_type = if is_high {
                    OrderBlockType::Bearish
                } else {
                    OrderBlockType::Bullish
                };

                order_blocks.push(OrderBlock {
                    start_price: candle.low_price(),
                    end_price: candle.high_price(),
                    block_type,
                    index,
                    strength: 1,
                });
            }
        }

        order_blocks
    }

    /// 유동성 풀 식별
    fn identify_liquidity_pools(&self, candles: &[C]) -> Vec<LiquidityPool> {
        let mut liquidity_pools = Vec::new();
        let swing_points = self.identify_swing_points(candles);

        for (index, price, is_high) in swing_points {
            if index >= candles.len() {
                continue;
            }

            let candle = &candles[index];
            let pool_type = if is_high {
                LiquidityPoolType::SellLiquidity
            } else {
                LiquidityPoolType::BuyLiquidity
            };

            liquidity_pools.push(LiquidityPool {
                price,
                pool_type,
                index,
                liquidity_amount: candle.volume(),
            });
        }

        liquidity_pools
    }

    /// 시장 흐름 강도 계산
    fn calculate_market_flow_strength(&self, candles: &[C]) -> f64 {
        if candles.len() < 10 {
            return 0.0;
        }

        let recent_candles = &candles[..10];
        let bullish_count = recent_candles
            .iter()
            .filter(|c| c.close_price() > c.open_price())
            .count();

        let volume_trend = self.calculate_volume_trend(recent_candles);
        let price_momentum = self.calculate_price_momentum(recent_candles);

        let bullish_ratio = bullish_count as f64 / recent_candles.len() as f64;

        (bullish_ratio + volume_trend + price_momentum) / 3.0
    }

    /// 볼륨 트렌드 계산
    fn calculate_volume_trend(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let recent_volume: f64 = candles.iter().take(5).map(|c| c.volume()).sum();
        let past_volume: f64 = candles.iter().skip(5).map(|c| c.volume()).sum();

        if past_volume == 0.0 {
            return 0.0;
        }

        ((recent_volume - past_volume) / past_volume).clamp(-1.0, 1.0)
    }

    /// 가격 모멘텀 계산
    fn calculate_price_momentum(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let current_price = candles[0].close_price();
        let past_price = candles[candles.len() - 1].close_price();

        if past_price == 0.0 {
            return 0.0;
        }

        ((current_price - past_price) / past_price).clamp(-1.0, 1.0)
    }

    /// 임밸런스 정도 계산
    fn calculate_imbalance_degree(&self, candles: &[C]) -> f64 {
        if candles.len() < 5 {
            return 0.0;
        }

        let recent_candles = &candles[..5];
        let mut imbalance_score = 0.0;

        for candle in recent_candles {
            let body_size = (candle.close_price() - candle.open_price()).abs();
            let total_size = candle.high_price() - candle.low_price();
            let upper_shadow = candle.high_price() - candle.close_price().max(candle.open_price());
            let lower_shadow = candle.close_price().min(candle.open_price()) - candle.low_price();

            if total_size > 0.0 {
                let body_ratio = body_size / total_size;
                let shadow_imbalance = (upper_shadow - lower_shadow).abs() / total_size;
                imbalance_score += body_ratio + shadow_imbalance;
            }
        }

        (imbalance_score / recent_candles.len() as f64).clamp(0.0, 1.0)
    }

    /// 최근 스윙 포인트 반환
    fn get_recent_swing_points(&self, candles: &[C]) -> (Option<f64>, Option<f64>) {
        let swing_points = self.identify_swing_points(candles);

        let recent_high = swing_points
            .iter()
            .filter(|(_, _, is_high)| *is_high)
            .map(|(_, price, _)| *price)
            .next();

        let recent_low = swing_points
            .iter()
            .filter(|(_, _, is_high)| !*is_high)
            .map(|(_, price, _)| *price)
            .next();

        (recent_high, recent_low)
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
            market_structure,
            structure_change,
            fair_value_gaps,
            order_blocks,
            liquidity_pools,
            market_flow_strength,
            imbalance_degree,
            recent_swing_high,
            recent_swing_low,
        )
    }

    fn datum(&self) -> &Vec<MarketStructureAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<MarketStructureAnalyzerData<C>> {
        &mut self.items
    }
}
