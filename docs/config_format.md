# 전략 설정 파일 형식 가이드

트레이딩 전략 프로젝트에서는 TOML 형식의 설정 파일을 사용하여 전략 파라미터를 쉽게 설정할 수 있습니다.

## 설정 파일 위치

각 전략별 설정 파일은 `config` 디렉토리에 위치하며, 파일명은 전략 유형에 따라 다음과 같은 명명 규칙을 따릅니다:

```
config/[전략명].toml
```

예: `config/bband.toml`, `config/macd_short.toml`, `config/rsi.toml` 등

## 설정 파일 형식

TOML 형식은 가독성이 높고 직관적인 설정 파일 형식입니다. 다음은 일반적인 전략 설정 파일의 예시입니다:

```toml
# 볼린저 밴드 전략 설정
count = 2                # 확인 캔들 수
period = 20              # 볼린저 밴드 계산 기간
multiplier = 2.0         # 볼린저 밴드 승수 (표준편차 배수)
```

각 전략별 설정 가능한 파라미터는 다음과 같습니다:

### 볼린저 밴드 전략 (BBand)

```toml
count = 2                # 확인 캔들 수
period = 20              # 볼린저 밴드 계산 기간
multiplier = 2.0         # 볼린저 밴드 승수 (표준편차 배수)
```

### 볼린저 밴드 숏 전략 (BBandShort)

```toml
count = 3                # 확인 캔들 수
period = 20              # 볼린저 밴드 계산 기간
multiplier = 2.0         # 볼린저 밴드 승수 (표준편차 배수)
```

### MACD 전략 (MACD)

```toml
fast_period = 12         # 빠른 EMA 기간
slow_period = 26         # 느린 EMA 기간
signal_period = 9        # 시그널 라인 기간
histogram_threshold = 0.0 # 히스토그램 임계값 (0보다 클 때 롱 진입)
confirm_period = 3       # 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
```

### MACD 숏 전략 (MACDShort)

```toml
fast_period = 12         # 빠른 EMA 기간
slow_period = 26         # 느린 EMA 기간
signal_period = 9        # 시그널 라인 기간
histogram_threshold = -0.01 # 히스토그램 임계값 (0보다 작을 때 숏 진입)
confirm_period = 3       # 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
```

### 이동평균 전략 (MA)

```toml
ma_type = "sma"         # 이동평균 타입 (sma 또는 ema)
ma_periods = [5, 20]    # 이동평균 기간 배열 (짧은 기간에서 긴 기간 순으로 정렬)
cross_previous_periods = 1  # 크로스 판정 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
```

### RSI 전략 (RSI)

```toml
rsi_period = 14         # RSI 계산 기간
rsi_upper = 70.0        # RSI 상단 기준값 (이 값보다 높으면 과매수)
rsi_lower = 30.0        # RSI 하단 기준값 (이 값보다 낮으면 과매도)
rsi_count = 3           # RSI 판정 횟수 (연속으로 조건이 만족되어야 하는 횟수)
ma_periods = [20, 50]   # 이동평균 기간 배열 (짧은 기간에서 긴 기간 순으로 정렬)
```

## 설정 파일 사용 방법

설정 파일을 사용하여 전략을 로드하는 방법은 다음과 같습니다:

```rust
use trading_strategy::strategy::{StrategyFactory, StrategyType};
use std::path::Path;

// 캔들 저장소 생성
let storage = CandleStore::new();

// 설정 파일 경로 지정
let config_path = Path::new("config/bband.toml");

// 설정 파일에서 전략 로드
let strategy = StrategyFactory::build_from_config(
    StrategyType::BBand,
    &storage,
    &config_path
)?;
```

또는 예제 프로그램을 사용하여 설정 파일 테스트:

```bash
cargo run --bin load_strategy_config bband config/bband.toml
```

## 설정 파일 유효성 검사

모든 설정 파일은 로드 시 자동으로 유효성 검사를 수행합니다. 유효하지 않은 파라미터가 있는 경우 오류 메시지와 함께 로드가 실패합니다. 