use crate::candle_store::CandleStore;
use crate::strategy::Strategy;
use crate::tests::TestCandle;
use chrono::Utc;
use trading_chart::Candle;

/// 테스트용 캔들 생성 함수
///
/// 특정 가격과 시간을 가진 테스트용 캔들을 생성합니다.
///
/// # 인자
///
/// * `price` - 캔들의 시가, 고가, 저가, 종가로 사용할 가격
/// * `timestamp` - 캔들의 시간 (유닉스 타임스탬프)
///
/// # 반환값
///
/// * `TestCandle` - 생성된 캔들 데이터
#[allow(dead_code)]
pub fn create_test_candle(price: f64, timestamp: i64, volume: f64, _market: &str) -> TestCandle {
    TestCandle {
        timestamp,
        open: price,
        high: price,
        low: price,
        close: price,
        volume,
    }
}

/// 테스트용 캔들 스토리지 생성 함수
///
/// 주어진 캔들 벡터로부터 테스트용 스토리지를 생성합니다.
///
/// # 인자
///
/// * `candles` - 캔들 벡터
///
/// # 반환값
///
/// * `CandleStore<TestCandle>` - 생성된 캔들 스토리지
#[allow(dead_code)]
pub fn create_test_storage(candles: Vec<TestCandle>) -> CandleStore<TestCandle> {
    let mut storage = CandleStore::new(Vec::new(), 1000, false);
    for candle in candles {
        storage.add(candle);
    }
    storage
}

/// 상승 트렌드 캔들 시퀀스 생성 함수
///
/// 상승 추세를 나타내는 캔들 시퀀스를 생성합니다.
///
/// # 인자
///
/// * `count` - 생성할 캔들 수
/// * `base_price` - 시작 가격
/// * `price_increment` - 각 캔들마다 증가할 가격
///
/// # 반환값
///
/// * `Vec<TestCandle>` - 상승 트렌드 캔들 시퀀스
#[allow(dead_code)]
pub fn create_uptrend_candles(
    count: usize,
    base_price: f64,
    price_increment: f64,
) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        let price = base_price + (i as f64 * price_increment);
        let timestamp = now.timestamp() + (i as i64 * 60); // 1분 간격
        // 노이즈 추가: 0 ~ price_increment의 100% 범위 내에서 의사 난수 값
        let noise = if i > 0 {
            ((i as f64 * 13.0) % 10.0 - 5.0) * price_increment * 0.5
        } else {
            0.0
        };
        let noisy_price = price + noise;
        let candle = TestCandle {
            timestamp,
            open: noisy_price - price_increment / 2.0,
            high: noisy_price + price_increment / 4.0,
            low: noisy_price - price_increment / 4.0,
            close: noisy_price,
            volume: 100.0,
        };
        candles.push(candle);
    }

    candles
}

/// 하락 트렌드 캔들 시퀀스 생성 함수
///
/// 하락 추세를 나타내는 캔들 시퀀스를 생성합니다.
///
/// # 인자
///
/// * `count` - 생성할 캔들 수
/// * `base_price` - 시작 가격
/// * `price_decrement` - 각 캔들마다 감소할 가격
///
/// # 반환값
///
/// * `Vec<TestCandle>` - 하락 트렌드 캔들 시퀀스
#[allow(dead_code)]
pub fn create_downtrend_candles(
    count: usize,
    base_price: f64,
    price_decrement: f64,
) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        let price = base_price - (i as f64 * price_decrement);
        let timestamp = now.timestamp() + (i as i64 * 60); // 1분 간격
        let candle = TestCandle {
            timestamp,
            open: price + price_decrement / 2.0,
            high: price + price_decrement / 4.0,
            low: price - price_decrement / 4.0,
            close: price,
            volume: 100.0,
        };
        candles.push(candle);
    }

    candles
}

/// 횡보 트렌드 캔들 시퀀스 생성 함수
///
/// 횡보 추세를 나타내는 캔들 시퀀스를 생성합니다.
///
/// # 인자
///
/// * `count` - 생성할 캔들 수
/// * `base_price` - 기준 가격
/// * `range` - 가격 변동 범위
///
/// # 반환값
///
/// * `Vec<TestCandle>` - 횡보 트렌드 캔들 시퀀스
#[allow(dead_code)]
pub fn create_sideways_candles(count: usize, base_price: f64, range: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        let oscillation = (i % 3) as f64 * range / 3.0;
        let price = base_price + oscillation;
        let timestamp = now.timestamp() + (i as i64 * 60); // 1분 간격
        let candle = TestCandle {
            timestamp,
            open: price,
            high: price + range / 4.0,
            low: price - range / 4.0,
            close: price,
            volume: 100.0,
        };
        candles.push(candle);
    }

    candles
}

/// 전략 백테스팅 결과
#[derive(Debug)]
#[allow(dead_code)]
pub struct BacktestResult {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub total_profit_percentage: f64,
    pub max_drawdown_percentage: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_profit_per_trade: f64,
    pub avg_loss_per_trade: f64,
}

/// 전략 백테스팅 함수
///
/// # Arguments
/// * `strategy` - 테스트할 전략
/// * `candles` - 테스트용 캔들 데이터
/// * `initial_capital` - 초기 자본
///
/// # Returns
/// * `BacktestResult` - 백테스팅 결과
pub fn backtest_strategy<C: Candle, S: Strategy<C>>(
    mut strategy: S,
    candles: Vec<C>,
    initial_capital: f64,
) -> BacktestResult {
    let mut capital = initial_capital;
    let mut position: Option<(f64, f64)> = None; // (진입 가격, 수량)

    let mut trades = Vec::new();
    let mut max_drawdown = 0.0;
    let mut equity_peak = initial_capital;

    // 백테스팅 로직
    for candle in &candles {
        strategy.next(candle.clone());

        // 포지션이 없는 경우 매수 신호 확인
        if position.is_none() {
            if strategy.should_enter(candle) {
                let price = candle.close_price();
                let quantity = capital / price;
                position = Some((price, quantity));
            }
        }
        // 포지션이 있는 경우 매도 신호 확인
        else if let Some((entry_price, quantity)) = position {
            if strategy.should_exit(candle) {
                let exit_price = candle.close_price();
                let profit = (exit_price - entry_price) * quantity;
                let profit_percentage = (exit_price / entry_price - 1.0) * 100.0;

                trades.push(profit_percentage);
                capital += profit;
                position = None;

                // 자산 최고점 업데이트
                if capital > equity_peak {
                    equity_peak = capital;
                } else {
                    let current_drawdown = (equity_peak - capital) / equity_peak * 100.0;
                    if current_drawdown > max_drawdown {
                        max_drawdown = current_drawdown;
                    }
                }
            }
        }
    }

    // 결과 계산
    let total_trades = trades.len();
    let (winning_trades, losing_trades): (Vec<f64>, Vec<f64>) =
        trades.iter().partition(|&&profit| profit > 0.0);

    let total_profit_percentage = trades.iter().sum::<f64>();
    let win_rate = if total_trades > 0 {
        winning_trades.len() as f64 / total_trades as f64
    } else {
        0.0
    };

    let total_profits = winning_trades.iter().sum::<f64>();
    let total_losses = losing_trades.iter().map(|&x| x.abs()).sum::<f64>();

    let profit_factor = if total_losses > 0.0 {
        total_profits / total_losses
    } else {
        f64::INFINITY
    };

    let avg_profit_per_trade = if !winning_trades.is_empty() {
        total_profits / winning_trades.len() as f64
    } else {
        0.0
    };

    let avg_loss_per_trade = if !losing_trades.is_empty() {
        total_losses / losing_trades.len() as f64
    } else {
        0.0
    };

    BacktestResult {
        total_trades,
        winning_trades: winning_trades.len(),
        losing_trades: losing_trades.len(),
        total_profit_percentage,
        max_drawdown_percentage: max_drawdown,
        win_rate,
        profit_factor,
        avg_profit_per_trade,
        avg_loss_per_trade,
    }
}
