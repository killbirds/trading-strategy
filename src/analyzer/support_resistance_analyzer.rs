use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 지지/저항 레벨 데이터
#[derive(Debug, Clone)]
pub struct SupportResistanceLevel {
    /// 레벨 가격
    pub price: f64,
    /// 터치 횟수 (강도)
    pub touch_count: usize,
    /// 레벨 타입 (지지/저항)
    pub level_type: LevelType,
    /// 마지막 터치 인덱스
    pub last_touch_index: usize,
    /// 신뢰도 점수
    pub confidence_score: f64,
}

/// 레벨 타입
#[derive(Debug, Clone, PartialEq)]
pub enum LevelType {
    Support,
    Resistance,
    Both,
}

/// 지지/저항 분석기 데이터
#[derive(Debug)]
pub struct SupportResistanceAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 지지/저항 레벨들
    pub levels: Vec<SupportResistanceLevel>,
    /// 현재 가격과 가장 가까운 지지선
    pub nearest_support: Option<SupportResistanceLevel>,
    /// 현재 가격과 가장 가까운 저항선
    pub nearest_resistance: Option<SupportResistanceLevel>,
}

impl<C: Candle> SupportResistanceAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        levels: Vec<SupportResistanceLevel>,
        nearest_support: Option<SupportResistanceLevel>,
        nearest_resistance: Option<SupportResistanceLevel>,
    ) -> SupportResistanceAnalyzerData<C> {
        SupportResistanceAnalyzerData {
            candle,
            levels,
            nearest_support,
            nearest_resistance,
        }
    }

    /// 가격이 지지선 근처에 있는지 확인
    pub fn is_near_support(&self, threshold: f64) -> bool {
        if let Some(support) = &self.nearest_support {
            let distance = (self.candle.close_price() - support.price).abs();
            distance <= threshold
        } else {
            false
        }
    }

    /// 가격이 저항선 근처에 있는지 확인
    pub fn is_near_resistance(&self, threshold: f64) -> bool {
        if let Some(resistance) = &self.nearest_resistance {
            let distance = (self.candle.close_price() - resistance.price).abs();
            distance <= threshold
        } else {
            false
        }
    }

    /// 가격이 지지선 위에 있는지 확인
    pub fn is_above_support(&self) -> bool {
        if let Some(support) = &self.nearest_support {
            self.candle.close_price() > support.price
        } else {
            false
        }
    }

    /// 가격이 저항선 아래에 있는지 확인
    pub fn is_below_resistance(&self) -> bool {
        if let Some(resistance) = &self.nearest_resistance {
            self.candle.close_price() < resistance.price
        } else {
            false
        }
    }

    /// 강한 지지선 레벨들 반환
    pub fn get_strong_support_levels(
        &self,
        min_touch_count: usize,
    ) -> Vec<&SupportResistanceLevel> {
        self.levels
            .iter()
            .filter(|level| {
                (level.level_type == LevelType::Support || level.level_type == LevelType::Both)
                    && level.touch_count >= min_touch_count
            })
            .collect()
    }

    /// 강한 저항선 레벨들 반환
    pub fn get_strong_resistance_levels(
        &self,
        min_touch_count: usize,
    ) -> Vec<&SupportResistanceLevel> {
        self.levels
            .iter()
            .filter(|level| {
                (level.level_type == LevelType::Resistance || level.level_type == LevelType::Both)
                    && level.touch_count >= min_touch_count
            })
            .collect()
    }

    /// 현재 가격과 가장 가까운 지지선까지의 거리 반환
    pub fn distance_to_nearest_support(&self) -> Option<f64> {
        self.nearest_support
            .as_ref()
            .map(|support| (self.candle.close_price() - support.price).abs())
    }

    /// 현재 가격과 가장 가까운 저항선까지의 거리 반환
    pub fn distance_to_nearest_resistance(&self) -> Option<f64> {
        self.nearest_resistance
            .as_ref()
            .map(|resistance| (self.candle.close_price() - resistance.price).abs())
    }
}

impl<C: Candle> GetCandle<C> for SupportResistanceAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for SupportResistanceAnalyzerData<C> {}

/// 지지/저항 분석기
#[derive(Debug)]
pub struct SupportResistanceAnalyzer<C: Candle> {
    /// 분석 데이터 히스토리
    pub items: Vec<SupportResistanceAnalyzerData<C>>,
    /// 레벨 식별을 위한 설정
    pub lookback_period: usize,
    /// 레벨 터치 임계값
    pub touch_threshold: f64,
    /// 최소 터치 횟수
    pub min_touch_count: usize,
}

impl<C: Candle> Display for SupportResistanceAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "SupportResistanceAnalyzer {{ candle: {}, levels: {}, nearest_support: {:?}, nearest_resistance: {:?} }}",
                first.candle,
                first.levels.len(),
                first.nearest_support.as_ref().map(|s| s.price),
                first.nearest_resistance.as_ref().map(|r| r.price)
            )
        } else {
            write!(f, "SupportResistanceAnalyzer {{ no data }}")
        }
    }
}

impl<C: Candle + 'static> SupportResistanceAnalyzer<C> {
    /// 새 지지/저항 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        lookback_period: usize,
        touch_threshold: f64,
        min_touch_count: usize,
    ) -> SupportResistanceAnalyzer<C> {
        let mut analyzer = SupportResistanceAnalyzer {
            items: Vec::new(),
            lookback_period,
            touch_threshold,
            min_touch_count,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> SupportResistanceAnalyzer<C> {
        Self::new(storage, 50, 0.5, 2)
    }

    /// 지지/저항 레벨 식별
    fn identify_levels(&self, candles: &[C]) -> Vec<SupportResistanceLevel> {
        let mut levels = Vec::new();
        let mut potential_levels = Vec::new();

        // 최소 5개의 캔들이 필요 (피벗 포인트 계산을 위해)
        if candles.len() < 5 {
            return levels;
        }

        // 피벗 포인트 찾기
        for i in 2..candles.len() - 2 {
            let current_high = candles[i].high_price();
            let current_low = candles[i].low_price();

            // 피벗 하이 (저항선 후보)
            if current_high > candles[i - 1].high_price()
                && current_high > candles[i - 2].high_price()
                && current_high > candles[i + 1].high_price()
                && current_high > candles[i + 2].high_price()
            {
                potential_levels.push((current_high, LevelType::Resistance, i));
            }

            // 피벗 로우 (지지선 후보)
            if current_low < candles[i - 1].low_price()
                && current_low < candles[i - 2].low_price()
                && current_low < candles[i + 1].low_price()
                && current_low < candles[i + 2].low_price()
            {
                potential_levels.push((current_low, LevelType::Support, i));
            }
        }

        // 터치 횟수 계산
        for (price, level_type, index) in potential_levels {
            let mut touch_count = 1;
            let mut last_touch_index = index;

            for (j, candle) in candles.iter().enumerate() {
                if j == index {
                    continue;
                }

                let is_touch = match level_type {
                    LevelType::Support => {
                        (candle.low_price() - price).abs() <= self.touch_threshold
                    }
                    LevelType::Resistance => {
                        (candle.high_price() - price).abs() <= self.touch_threshold
                    }
                    LevelType::Both => {
                        (candle.low_price() - price).abs() <= self.touch_threshold
                            || (candle.high_price() - price).abs() <= self.touch_threshold
                    }
                };

                if is_touch {
                    touch_count += 1;
                    last_touch_index = j;
                }
            }

            if touch_count >= self.min_touch_count {
                let confidence_score =
                    self.calculate_confidence_score(touch_count, last_touch_index, candles.len());

                levels.push(SupportResistanceLevel {
                    price,
                    touch_count,
                    level_type,
                    last_touch_index,
                    confidence_score,
                });
            }
        }

        levels
    }

    /// 신뢰도 점수 계산
    fn calculate_confidence_score(
        &self,
        touch_count: usize,
        last_touch_index: usize,
        total_candles: usize,
    ) -> f64 {
        let touch_score = (touch_count as f64 - 1.0) * 0.2;
        let recency_score = 1.0 - (total_candles - last_touch_index) as f64 / total_candles as f64;
        (touch_score + recency_score * 0.5).min(1.0)
    }

    /// 가장 가까운 지지/저항선 찾기
    fn find_nearest_levels(
        &self,
        current_price: f64,
        levels: &[SupportResistanceLevel],
    ) -> (
        Option<SupportResistanceLevel>,
        Option<SupportResistanceLevel>,
    ) {
        let mut nearest_support = None;
        let mut nearest_resistance = None;
        let mut min_support_distance = f64::MAX;
        let mut min_resistance_distance = f64::MAX;

        for level in levels {
            let distance = (current_price - level.price).abs();

            match level.level_type {
                LevelType::Support => {
                    if level.price < current_price && distance < min_support_distance {
                        min_support_distance = distance;
                        nearest_support = Some(level.clone());
                    }
                }
                LevelType::Resistance => {
                    if level.price > current_price && distance < min_resistance_distance {
                        min_resistance_distance = distance;
                        nearest_resistance = Some(level.clone());
                    }
                }
                LevelType::Both => {
                    if level.price < current_price && distance < min_support_distance {
                        min_support_distance = distance;
                        nearest_support = Some(level.clone());
                    }
                    if level.price > current_price && distance < min_resistance_distance {
                        min_resistance_distance = distance;
                        nearest_resistance = Some(level.clone());
                    }
                }
            }
        }

        (nearest_support, nearest_resistance)
    }

    /// 지지선 브레이크다운 확인
    pub fn is_support_breakdown(&self) -> bool {
        if let (Some(current), Some(previous)) = (self.items.first(), self.items.get(1)) {
            if let (Some(current_support), Some(previous_support)) =
                (&current.nearest_support, &previous.nearest_support)
            {
                previous.candle.close_price() >= previous_support.price
                    && current.candle.close_price() < current_support.price
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 저항선 브레이크아웃 확인
    pub fn is_resistance_breakout(&self) -> bool {
        if let (Some(current), Some(previous)) = (self.items.first(), self.items.get(1)) {
            if let (Some(current_resistance), Some(previous_resistance)) =
                (&current.nearest_resistance, &previous.nearest_resistance)
            {
                previous.candle.close_price() <= previous_resistance.price
                    && current.candle.close_price() > current_resistance.price
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 지지선에서 반등 확인
    pub fn is_support_bounce(&self, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        let current = match self.items.first() {
            Some(item) => item,
            None => return false,
        };
        if let Some(support) = &current.nearest_support {
            // 최근 n개 캔들 중 지지선 근처에서 반등했는지 확인
            let recent_low = self.items[..n]
                .iter()
                .map(|item| item.candle.low_price())
                .fold(f64::MAX, f64::min);

            let was_near_support = (recent_low - support.price).abs() <= self.touch_threshold;
            let is_bouncing = current.candle.close_price() > support.price;

            was_near_support && is_bouncing
        } else {
            false
        }
    }

    /// 저항선에서 거부 확인
    pub fn is_resistance_rejection(&self, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        let current = match self.items.first() {
            Some(item) => item,
            None => return false,
        };
        if let Some(resistance) = &current.nearest_resistance {
            // 최근 n개 캔들 중 저항선 근처에서 거부되었는지 확인
            let recent_high = self.items[..n]
                .iter()
                .map(|item| item.candle.high_price())
                .fold(f64::MIN, f64::max);

            let was_near_resistance =
                (recent_high - resistance.price).abs() <= self.touch_threshold;
            let is_rejected = current.candle.close_price() < resistance.price;

            was_near_resistance && is_rejected
        } else {
            false
        }
    }

    /// 강력한 지지선 근처 여부
    pub fn is_near_strong_support(&self, threshold: f64) -> bool {
        if let Some(data) = self.items.first() {
            let strong_supports = data.get_strong_support_levels(3);
            strong_supports
                .iter()
                .any(|support| (data.candle.close_price() - support.price).abs() <= threshold)
        } else {
            false
        }
    }

    /// 강력한 저항선 근처 여부
    pub fn is_near_strong_resistance(&self, threshold: f64) -> bool {
        if let Some(data) = self.items.first() {
            let strong_resistances = data.get_strong_resistance_levels(3);
            strong_resistances
                .iter()
                .any(|resistance| (data.candle.close_price() - resistance.price).abs() <= threshold)
        } else {
            false
        }
    }

    /// 지지선 붕괴 신호 확인 (n개 연속 지지선 붕괴, 이전 m개는 아님)
    pub fn is_support_breakdown_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(support) = &data.nearest_support {
                    data.candle.low_price() < support.price
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 저항선 돌파 신호 확인 (n개 연속 저항선 돌파, 이전 m개는 아님)
    pub fn is_resistance_breakout_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(resistance) = &data.nearest_resistance {
                    data.candle.high_price() > resistance.price
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 지지선 바운스 신호 확인 (n개 연속 지지선 바운스, 이전 m개는 아님)
    pub fn is_support_bounce_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(support) = &data.nearest_support {
                    let distance = (data.candle.low_price() - support.price).abs();
                    distance <= support.price * 0.002 // 0.2% 이내
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 저항선 거부 신호 확인 (n개 연속 저항선 거부, 이전 m개는 아님)
    pub fn is_resistance_rejection_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(resistance) = &data.nearest_resistance {
                    let distance = (data.candle.high_price() - resistance.price).abs();
                    distance <= resistance.price * 0.002 // 0.2% 이내
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 강한 지지선 근처 신호 확인 (n개 연속 강한 지지선 근처, 이전 m개는 아님)
    pub fn is_near_strong_support_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(support) = &data.nearest_support {
                    let distance = (data.candle.low_price() - support.price).abs();
                    distance <= threshold && support.touch_count >= 3
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 강한 저항선 근처 신호 확인 (n개 연속 강한 저항선 근처, 이전 m개는 아님)
    pub fn is_near_strong_resistance_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                if let Some(resistance) = &data.nearest_resistance {
                    let distance = (data.candle.high_price() - resistance.price).abs();
                    distance <= threshold && resistance.touch_count >= 3
                } else {
                    false
                }
            },
            n,
            m,
            p,
        )
    }

    /// 지지선 위 신호 확인 (n개 연속 지지선 위, 이전 m개는 아님)
    pub fn is_above_support_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_above_support(), n, m, p)
    }

    /// 저항선 아래 신호 확인 (n개 연속 저항선 아래, 이전 m개는 아님)
    pub fn is_below_resistance_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_below_resistance(), n, m, p)
    }

    /// 지지선 근처 신호 확인 (n개 연속 지지선 근처, 이전 m개는 아님)
    pub fn is_near_support_signal(&self, n: usize, m: usize, threshold: f64, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_near_support(threshold), n, m, p)
    }

    /// 저항선 근처 신호 확인 (n개 연속 저항선 근처, 이전 m개는 아님)
    pub fn is_near_resistance_signal(&self, n: usize, m: usize, threshold: f64, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_near_resistance(threshold), n, m, p)
    }

    /// n개의 연속 데이터에서 지지선 위인지 확인
    pub fn is_above_support(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_above_support(), n, p)
    }

    /// n개의 연속 데이터에서 저항선 아래인지 확인
    pub fn is_below_resistance(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_below_resistance(), n, p)
    }

    /// n개의 연속 데이터에서 지지선 근처인지 확인
    pub fn is_near_support(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(|data| data.is_near_support(threshold), n, p)
    }

    /// n개의 연속 데이터에서 저항선 근처인지 확인
    pub fn is_near_resistance(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(|data| data.is_near_resistance(threshold), n, p)
    }
}

impl<C: Candle + 'static> AnalyzerOps<SupportResistanceAnalyzerData<C>, C>
    for SupportResistanceAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> SupportResistanceAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        for item in self.items.iter().take(self.lookback_period - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 지지/저항 레벨 식별
        let levels = self.identify_levels(&recent_candles);

        // 가장 가까운 지지/저항선 찾기
        let (nearest_support, nearest_resistance) =
            self.find_nearest_levels(candle.close_price(), &levels);

        SupportResistanceAnalyzerData::new(candle, levels, nearest_support, nearest_resistance)
    }

    fn datum(&self) -> &Vec<SupportResistanceAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<SupportResistanceAnalyzerData<C>> {
        &mut self.items
    }
}
