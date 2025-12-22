use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 리스크 레벨 타입
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// 포지션 타입
#[derive(Debug, Clone, PartialEq)]
pub enum PositionType {
    Long,
    Short,
}

/// 포지션 크기 계산 방법
#[derive(Debug, Clone, PartialEq)]
pub enum PositionSizingMethod {
    /// 고정 비율 (Fixed Percentage)
    FixedPercentage,
    /// 켈리 기준 (Kelly Criterion)
    KellyCriterion,
    /// 변동성 기반 (Volatility Based)
    VolatilityBased,
    /// ATR 기반 (ATR Based)
    ATRBased,
}

/// 리스크 관리 계산 결과
#[derive(Debug, Clone)]
pub struct RiskCalculation {
    /// 진입 가격
    pub entry_price: f64,
    /// 손절매 가격
    pub stop_loss_price: f64,
    /// 타겟 가격
    pub target_price: f64,
    /// 포지션 크기
    pub position_size: f64,
    /// 리스크 금액
    pub risk_amount: f64,
    /// 잠재적 수익
    pub potential_profit: f64,
    /// 리스크/리워드 비율
    pub risk_reward_ratio: f64,
    /// 예상 수익률
    pub expected_return: f64,
    /// 변동성 점수
    pub volatility_score: f64,
    /// 신뢰도 점수
    pub confidence_score: f64,
}

/// Risk Management 분석기 데이터
#[derive(Debug)]
pub struct RiskManagementAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 현재 리스크 레벨
    pub risk_level: RiskLevel,
    /// 변동성 (ATR)
    pub atr: f64,
    /// 변동성 백분율
    pub volatility_percentage: f64,
    /// 일일 변동성
    pub daily_volatility: f64,
    /// 주간 변동성
    pub weekly_volatility: f64,
    /// 최대 손실 비율
    pub max_drawdown: f64,
    /// 샤프 비율
    pub sharpe_ratio: f64,
    /// 변동성 지수
    pub volatility_index: f64,
    /// 리스크 조정 수익률
    pub risk_adjusted_return: f64,
    /// 최적 포지션 크기 (계좌 대비 비율)
    pub optimal_position_size: f64,
    /// 권장 손절매 거리
    pub recommended_stop_distance: f64,
    /// 권장 타겟 거리
    pub recommended_target_distance: f64,
}

impl<C: Candle> RiskManagementAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(
        candle: C,
        risk_level: RiskLevel,
        atr: f64,
        volatility_percentage: f64,
        daily_volatility: f64,
        weekly_volatility: f64,
        max_drawdown: f64,
        sharpe_ratio: f64,
        volatility_index: f64,
        risk_adjusted_return: f64,
        optimal_position_size: f64,
        recommended_stop_distance: f64,
        recommended_target_distance: f64,
    ) -> RiskManagementAnalyzerData<C> {
        RiskManagementAnalyzerData {
            candle,
            risk_level,
            atr,
            volatility_percentage,
            daily_volatility,
            weekly_volatility,
            max_drawdown,
            sharpe_ratio,
            volatility_index,
            risk_adjusted_return,
            optimal_position_size,
            recommended_stop_distance,
            recommended_target_distance,
        }
    }

    /// 높은 리스크 상황인지 확인
    pub fn is_high_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::High | RiskLevel::VeryHigh)
    }

    /// 낮은 리스크 상황인지 확인
    pub fn is_low_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::Low | RiskLevel::VeryLow)
    }

    /// 높은 변동성인지 확인
    pub fn is_high_volatility(&self) -> bool {
        self.volatility_percentage > 5.0
    }

    /// 낮은 변동성인지 확인
    pub fn is_low_volatility(&self) -> bool {
        self.volatility_percentage < 1.0
    }

    /// 좋은 샤프 비율인지 확인
    pub fn is_good_sharpe_ratio(&self) -> bool {
        self.sharpe_ratio > 1.0
    }

    /// 큰 포지션을 권장하는지 확인
    pub fn is_large_position_recommended(&self) -> bool {
        self.optimal_position_size > 0.1 && !self.is_high_risk()
    }

    /// 작은 포지션을 권장하는지 확인
    pub fn is_small_position_recommended(&self) -> bool {
        self.optimal_position_size < 0.02 || self.is_high_risk()
    }

    /// 리스크 조정 수익률이 양수인지 확인
    pub fn is_positive_risk_adjusted_return(&self) -> bool {
        self.risk_adjusted_return > 0.0
    }

    /// 최대 손실이 허용 가능한지 확인
    pub fn is_acceptable_drawdown(&self, max_acceptable: f64) -> bool {
        self.max_drawdown < max_acceptable
    }

    /// 변동성 기반 스톱 로스 계산
    pub fn calculate_volatility_stop_loss(
        &self,
        entry_price: f64,
        position_type: PositionType,
    ) -> f64 {
        match position_type {
            PositionType::Long => entry_price - (self.atr * 2.0),
            PositionType::Short => entry_price + (self.atr * 2.0),
        }
    }

    /// 변동성 기반 타겟 가격 계산
    pub fn calculate_volatility_target(
        &self,
        entry_price: f64,
        position_type: PositionType,
    ) -> f64 {
        match position_type {
            PositionType::Long => entry_price + (self.atr * 3.0),
            PositionType::Short => entry_price - (self.atr * 3.0),
        }
    }

    /// 현재 상황에서 거래 권장 여부
    pub fn is_trading_recommended(&self) -> bool {
        !self.is_high_risk()
            && self.is_positive_risk_adjusted_return()
            && self.sharpe_ratio > 0.5
            && self.optimal_position_size > 0.01
    }
}

impl<C: Candle> GetCandle<C> for RiskManagementAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for RiskManagementAnalyzerData<C> {}

/// Risk Management 분석기
#[derive(Debug)]
pub struct RiskManagementAnalyzer<C: Candle> {
    /// 분석 데이터 히스토리
    pub items: Vec<RiskManagementAnalyzerData<C>>,
    /// ATR 계산 기간
    pub atr_period: usize,
    /// 변동성 계산 기간
    pub volatility_period: usize,
    /// 최대 리스크 비율
    pub max_risk_percentage: f64,
    /// 리스크 프리 수익률
    pub risk_free_rate: f64,
}

impl<C: Candle> Display for RiskManagementAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "RiskManagementAnalyzer {{ candle: {}, risk_level: {:?}, atr: {:.2}, position_size: {:.2}% }}",
                first.candle,
                first.risk_level,
                first.atr,
                first.optimal_position_size * 100.0
            )
        } else {
            write!(f, "RiskManagementAnalyzer {{ no data }}")
        }
    }
}

impl<C: Candle + Clone + 'static> RiskManagementAnalyzer<C> {
    /// 새 Risk Management 분석기 생성
    pub fn new(
        storage: &CandleStore<C>,
        atr_period: usize,
        volatility_period: usize,
        max_risk_percentage: f64,
        risk_free_rate: f64,
    ) -> RiskManagementAnalyzer<C> {
        let mut analyzer = RiskManagementAnalyzer {
            items: Vec::new(),
            atr_period,
            volatility_period,
            max_risk_percentage,
            risk_free_rate,
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 기본 설정으로 분석기 생성
    pub fn default(storage: &CandleStore<C>) -> RiskManagementAnalyzer<C> {
        Self::new(storage, 14, 30, 0.02, 0.03)
    }

    /// ATR 계산
    fn calculate_atr(&self, candles: &[C]) -> f64 {
        if candles.len() < self.atr_period {
            return 0.0;
        }

        let mut tr_sum = 0.0;
        for i in 1..self.atr_period.min(candles.len()) {
            let current = &candles[i];
            let previous = &candles[i - 1];

            let tr1 = current.high_price() - current.low_price();
            let tr2 = (current.high_price() - previous.close_price()).abs();
            let tr3 = (current.low_price() - previous.close_price()).abs();

            tr_sum += tr1.max(tr2).max(tr3);
        }

        tr_sum / (self.atr_period - 1) as f64
    }

    /// 변동성 계산
    fn calculate_volatility(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = candles
            .windows(2)
            .map(|w| {
                let current = w[0].close_price();
                let previous = w[1].close_price();
                (current / previous).ln()
            })
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;

        variance.sqrt() * 100.0
    }

    /// 일일 변동성 계산
    fn calculate_daily_volatility(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let daily_returns: Vec<f64> = candles
            .windows(2)
            .map(|w| {
                let current = w[0].close_price();
                let previous = w[1].close_price();
                ((current - previous) / previous).abs()
            })
            .collect();

        if daily_returns.is_empty() {
            return 0.0;
        }

        daily_returns.iter().sum::<f64>() / daily_returns.len() as f64
    }

    /// 주간 변동성 계산
    fn calculate_weekly_volatility(&self, candles: &[C]) -> f64 {
        if candles.len() < 7 {
            return 0.0;
        }

        let weekly_returns: Vec<f64> = candles
            .chunks(7)
            .filter_map(|week| {
                if week.len() < 2 {
                    return None;
                }
                let start = week.last()?.close_price();
                let end = week.first()?.close_price();
                Some(((end - start) / start).abs())
            })
            .collect();

        if weekly_returns.is_empty() {
            return 0.0;
        }

        weekly_returns.iter().sum::<f64>() / weekly_returns.len() as f64
    }

    /// 최대 손실 계산
    fn calculate_max_drawdown(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let mut max_drawdown = 0.0;
        let mut peak = candles.last().map(|c| c.close_price()).unwrap_or(0.0);

        for candle in candles.iter().rev() {
            let current_price = candle.close_price();
            if current_price > peak {
                peak = current_price;
            }

            let drawdown = (peak - current_price) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    /// 샤프 비율 계산
    fn calculate_sharpe_ratio(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = candles
            .windows(2)
            .map(|w| {
                let current = w[0].close_price();
                let previous = w[1].close_price();
                (current - previous) / previous
            })
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        (mean_return - self.risk_free_rate / 365.0) / std_dev
    }

    /// 변동성 지수 계산
    fn calculate_volatility_index(&self, candles: &[C]) -> f64 {
        if candles.len() < 10 {
            return 0.0;
        }

        let recent_volatility = self.calculate_volatility(&candles[..10]);
        let long_term_volatility = self.calculate_volatility(candles);

        if long_term_volatility == 0.0 {
            return 0.0;
        }

        recent_volatility / long_term_volatility
    }

    /// 리스크 조정 수익률 계산
    fn calculate_risk_adjusted_return(&self, candles: &[C]) -> f64 {
        if candles.len() < 2 {
            return 0.0;
        }

        let last_price = candles.last().map(|c| c.close_price()).unwrap_or(0.0);
        if last_price == 0.0 {
            return 0.0;
        }
        let total_return = (candles[0].close_price() - last_price) / last_price;
        let volatility = self.calculate_volatility(candles);

        if volatility == 0.0 {
            return 0.0;
        }

        total_return / volatility
    }

    /// 최적 포지션 크기 계산
    fn calculate_optimal_position_size(&self, candles: &[C]) -> f64 {
        let atr = self.calculate_atr(candles);
        let volatility = self.calculate_volatility(candles);
        let current_price = candles[0].close_price();

        if current_price == 0.0 || atr == 0.0 {
            return 0.0;
        }

        // 변동성 기반 포지션 크기 계산
        let volatility_factor = if volatility > 5.0 {
            0.5
        } else if volatility > 2.0 {
            0.7
        } else {
            1.0
        };

        let base_position_size = self.max_risk_percentage / (atr / current_price);
        (base_position_size * volatility_factor).min(0.2) // 최대 20%로 제한
    }

    /// 권장 스톱 로스 거리 계산
    fn calculate_recommended_stop_distance(&self, candles: &[C]) -> f64 {
        let atr = self.calculate_atr(candles);
        let volatility = self.calculate_volatility(candles);

        // 변동성에 따른 스톱 로스 거리 조정
        let multiplier = if volatility > 5.0 {
            3.0
        } else if volatility > 2.0 {
            2.5
        } else {
            2.0
        };

        atr * multiplier
    }

    /// 권장 타겟 거리 계산
    fn calculate_recommended_target_distance(&self, candles: &[C]) -> f64 {
        let stop_distance = self.calculate_recommended_stop_distance(candles);
        stop_distance * 2.0 // 리스크/리워드 비율 1:2
    }

    /// 리스크 레벨 계산
    fn calculate_risk_level(&self, candles: &[C]) -> RiskLevel {
        let volatility = self.calculate_volatility(candles);
        let drawdown = self.calculate_max_drawdown(candles);
        let sharpe_ratio = self.calculate_sharpe_ratio(candles);

        // 여러 지표를 종합하여 리스크 레벨 결정
        let risk_score = (volatility / 10.0) + (drawdown * 2.0) - (sharpe_ratio.max(0.0) * 0.5);

        if risk_score < 0.2 {
            RiskLevel::VeryLow
        } else if risk_score < 0.5 {
            RiskLevel::Low
        } else if risk_score < 1.0 {
            RiskLevel::Medium
        } else if risk_score < 2.0 {
            RiskLevel::High
        } else {
            RiskLevel::VeryHigh
        }
    }

    /// 리스크 계산 수행
    pub fn calculate_risk(
        &self,
        entry_price: f64,
        position_type: PositionType,
        account_balance: f64,
        sizing_method: PositionSizingMethod,
    ) -> Option<RiskCalculation> {
        if let Some(data) = self.items.first() {
            let stop_loss_price =
                data.calculate_volatility_stop_loss(entry_price, position_type.clone());
            let target_price = data.calculate_volatility_target(entry_price, position_type.clone());

            let risk_per_unit = match position_type {
                PositionType::Long => (entry_price - stop_loss_price).abs(),
                PositionType::Short => (stop_loss_price - entry_price).abs(),
            };

            let position_size = match sizing_method {
                PositionSizingMethod::FixedPercentage => {
                    let risk_amount = account_balance * self.max_risk_percentage;
                    risk_amount / risk_per_unit
                }
                PositionSizingMethod::VolatilityBased => {
                    data.optimal_position_size * account_balance / entry_price
                }
                PositionSizingMethod::ATRBased => {
                    let risk_amount = account_balance * self.max_risk_percentage;
                    risk_amount / (data.atr * 2.0)
                }
                PositionSizingMethod::KellyCriterion => {
                    // 켈리 기준 (간단한 버전)
                    let win_rate = 0.6; // 실제로는 백테스트 결과에서 계산
                    let avg_win_loss_ratio = 1.5; // 실제로는 백테스트 결과에서 계산
                    let kelly_fraction: f64 =
                        (win_rate * avg_win_loss_ratio - (1.0 - win_rate)) / avg_win_loss_ratio;
                    let safe_kelly = kelly_fraction.clamp(0.0, 0.25) * 0.5; // 안전을 위해 절반만 사용
                    safe_kelly * account_balance / entry_price
                }
            };

            let risk_amount = position_size * risk_per_unit;
            let potential_profit = match position_type {
                PositionType::Long => (target_price - entry_price) * position_size,
                PositionType::Short => (entry_price - target_price) * position_size,
            };

            let risk_reward_ratio = if risk_amount > 0.0 {
                potential_profit / risk_amount
            } else {
                0.0
            };

            let expected_return = potential_profit / (position_size * entry_price);

            Some(RiskCalculation {
                entry_price,
                stop_loss_price,
                target_price,
                position_size,
                risk_amount,
                potential_profit,
                risk_reward_ratio,
                expected_return,
                volatility_score: data.volatility_percentage,
                confidence_score: data.sharpe_ratio.clamp(0.0, 1.0),
            })
        } else {
            None
        }
    }

    /// 포트폴리오 리스크 평가
    pub fn evaluate_portfolio_risk(&self, positions: &[RiskCalculation]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let total_risk = positions.iter().map(|p| p.risk_amount).sum::<f64>();
        let total_potential_profit = positions.iter().map(|p| p.potential_profit).sum::<f64>();

        if total_risk == 0.0 {
            return 0.0;
        }

        total_potential_profit / total_risk
    }

    /// 리스크 경고 확인
    pub fn check_risk_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if let Some(data) = self.items.first() {
            if data.is_high_risk() {
                warnings.push("높은 리스크 상황입니다.".to_string());
            }

            if data.is_high_volatility() {
                warnings.push("높은 변동성이 감지되었습니다.".to_string());
            }

            if data.max_drawdown > 0.2 {
                warnings.push("최대 손실이 20%를 초과했습니다.".to_string());
            }

            if data.sharpe_ratio < 0.0 {
                warnings.push("음수 샤프 비율입니다.".to_string());
            }

            if data.optimal_position_size < 0.01 {
                warnings.push("권장 포지션 크기가 매우 작습니다.".to_string());
            }
        }

        warnings
    }

    /// 고위험 신호 확인 (n개 연속 고위험, 이전 m개는 아님)
    pub fn is_high_risk_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_risk(), n, m, p)
    }

    /// 저위험 신호 확인 (n개 연속 저위험, 이전 m개는 아님)
    pub fn is_low_risk_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_low_risk(), n, m, p)
    }

    /// 고변동성 신호 확인 (n개 연속 고변동성, 이전 m개는 아님)
    pub fn is_high_volatility_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_high_volatility(), n, m, p)
    }

    /// 저변동성 신호 확인 (n개 연속 저변동성, 이전 m개는 아님)
    pub fn is_low_volatility_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_low_volatility(), n, m, p)
    }

    /// 좋은 샤프 비율 신호 확인 (n개 연속 좋은 샤프 비율, 이전 m개는 아님)
    pub fn is_good_sharpe_ratio_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_good_sharpe_ratio(), n, m, p)
    }

    /// 큰 포지션 권장 신호 확인 (n개 연속 큰 포지션 권장, 이전 m개는 아님)
    pub fn is_large_position_recommended_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_large_position_recommended(), n, m, p)
    }

    /// 작은 포지션 권장 신호 확인 (n개 연속 작은 포지션 권장, 이전 m개는 아님)
    pub fn is_small_position_recommended_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_small_position_recommended(), n, m, p)
    }

    /// 양의 리스크 조정 수익률 신호 확인 (n개 연속 양의 리스크 조정 수익률, 이전 m개는 아님)
    pub fn is_positive_risk_adjusted_return_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_positive_risk_adjusted_return(), n, m, p)
    }

    /// 거래 권장 신호 확인 (n개 연속 거래 권장, 이전 m개는 아님)
    pub fn is_trading_recommended_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_trading_recommended(), n, m, p)
    }

    /// 허용 가능한 손실 신호 확인 (n개 연속 허용 가능한 손실, 이전 m개는 아님)
    pub fn is_acceptable_drawdown_signal(
        &self,
        n: usize,
        m: usize,
        max_acceptable: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_acceptable_drawdown(max_acceptable),
            n,
            m,
            p,
        )
    }

    /// 포지션 크기 임계값 돌파 신호 확인 (n개 연속 포지션 크기 임계값 초과, 이전 m개는 아님)
    pub fn is_position_size_breakthrough(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.optimal_position_size > threshold, n, m, p)
    }

    /// 변동성 임계값 돌파 신호 확인 (n개 연속 변동성 임계값 초과, 이전 m개는 아님)
    pub fn is_volatility_breakthrough(&self, n: usize, m: usize, threshold: f64, p: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.volatility_percentage > threshold, n, m, p)
    }

    /// 샤프 비율 임계값 돌파 신호 확인 (n개 연속 샤프 비율 임계값 초과, 이전 m개는 아님)
    pub fn is_sharpe_ratio_breakthrough(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(|data| data.sharpe_ratio > threshold, n, m, p)
    }

    /// n개의 연속 데이터에서 고위험인지 확인
    pub fn is_high_risk(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_high_risk(), n, p)
    }

    /// n개의 연속 데이터에서 저위험인지 확인
    pub fn is_low_risk(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_low_risk(), n, p)
    }

    /// n개의 연속 데이터에서 고변동성인지 확인
    pub fn is_high_volatility(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_high_volatility(), n, p)
    }

    /// n개의 연속 데이터에서 저변동성인지 확인
    pub fn is_low_volatility(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_low_volatility(), n, p)
    }

    /// n개의 연속 데이터에서 거래 권장인지 확인
    pub fn is_trading_recommended(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_trading_recommended(), n, p)
    }
}

impl<C: Candle + Clone + 'static> AnalyzerOps<RiskManagementAnalyzerData<C>, C>
    for RiskManagementAnalyzer<C>
{
    fn next_data(&mut self, candle: C) -> RiskManagementAnalyzerData<C> {
        // 최근 캔들들을 수집
        let mut recent_candles = Vec::new();
        recent_candles.push(candle.clone());

        // 기존 데이터에서 캔들 추가
        let max_lookback = self.volatility_period.max(100);
        for item in self.items.iter().take(max_lookback - 1) {
            recent_candles.push(item.candle.clone());
        }

        // 분석 수행
        let atr = self.calculate_atr(&recent_candles);
        let volatility_percentage = self.calculate_volatility(&recent_candles);
        let daily_volatility = self.calculate_daily_volatility(&recent_candles);
        let weekly_volatility = self.calculate_weekly_volatility(&recent_candles);
        let max_drawdown = self.calculate_max_drawdown(&recent_candles);
        let sharpe_ratio = self.calculate_sharpe_ratio(&recent_candles);
        let volatility_index = self.calculate_volatility_index(&recent_candles);
        let risk_adjusted_return = self.calculate_risk_adjusted_return(&recent_candles);
        let optimal_position_size = self.calculate_optimal_position_size(&recent_candles);
        let recommended_stop_distance = self.calculate_recommended_stop_distance(&recent_candles);
        let recommended_target_distance =
            self.calculate_recommended_target_distance(&recent_candles);
        let risk_level = self.calculate_risk_level(&recent_candles);

        RiskManagementAnalyzerData::new(
            candle,
            risk_level,
            atr,
            volatility_percentage,
            daily_volatility,
            weekly_volatility,
            max_drawdown,
            sharpe_ratio,
            volatility_index,
            risk_adjusted_return,
            optimal_position_size,
            recommended_stop_distance,
            recommended_target_distance,
        )
    }

    fn datum(&self) -> &Vec<RiskManagementAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<RiskManagementAnalyzerData<C>> {
        &mut self.items
    }
}
