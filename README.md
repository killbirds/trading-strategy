# Trading-Common

금융 트레이딩 시스템을 위한 Rust 라이브러리입니다.

## 개요

Trading-Common은 알고리즘 트레이딩을 위한 다양한 전략과 도구를 제공하는 Rust 라이브러리입니다. 이 라이브러리는 기술적 지표(RSI, MACD, 이동평균선 등)를 활용한 거래 전략 구현과 백테스트를 지원합니다.

## 주요 기능

- **다양한 거래 전략**: RSI, MACD, 이동평균선, 볼린저 밴드 등 여러 기술적 지표 기반 전략
- **롱/숏 포지션 지원**: 모든 전략은 롱 포지션과 숏 포지션 버전을 제공
- **유연한 설정**: HashMap을 통한 전략 파라미터 설정 가능
- **데이터 스토리지**: 캔들 데이터 관리를 위한 효율적인 스토리지 시스템
- **백테스팅**: 과거 데이터를 사용한 전략 성능 검증

## 프로젝트 구조

```
trading-common/
├── src/
│   ├── candle_store.rs   - 캔들 데이터 관리 시스템
│   ├── position_calculator.rs - 포지션 계산
│   ├── position_state.rs - 포지션 상태 관리
│   ├── model.rs         - 데이터 모델 정의
│   ├── lib.rs           - 라이브러리 엔트리 포인트
│   ├── config/          - 설정 관리
│   ├── data_stream/     - 데이터 스트림 처리
│   ├── indicator/       - 기술적 지표
│   ├── report/          - 결과 보고
│   ├── risk/            - 리스크 관리 도구
│   └── strategy/        - 거래 전략 구현
├── examples/            - 사용 예제
└── config/              - 설정 파일
```

## 설치

Cargo.toml에 다음 내용을 추가합니다:

```toml
[dependencies]
trading-common = "0.2.0"
```

## 사용 방법

### 기본 사용법

```rust
use trading_common::candle_store::CandleStore;
use trading_common::model::Candle;
use trading_common::strategy::{StrategyFactory, StrategyType};

use std::collections::HashMap;

// 스토리지 초기화
let mut storage = CandleStore::new();

// 기본 설정으로 전략 생성
let strategy = StrategyFactory::build_with_default(StrategyType::RSI, &storage)
    .expect("전략 생성 실패");

// 커스텀 설정으로 전략 생성
let mut config = HashMap::new();
config.insert("rsi_period".to_string(), "14".to_string());
config.insert("rsi_lower".to_string(), "25.0".to_string());
config.insert("rsi_upper".to_string(), "75.0".to_string());

let strategy = StrategyFactory::build(StrategyType::RSI, &storage, Some(config))
    .expect("전략 생성 실패");
```

### 백테스팅 예제

examples 디렉토리의 백테스팅 예제를 실행하려면:

```bash
cargo run --example backtest_strategies_simple
```

## 지원하는 전략

다음과 같은 전략 유형을 제공합니다:

- **Dummy**: 테스트용 더미 전략
- **MA/MAShort**: 이동평균선 기반 롱/숏 전략
- **RSI/RSIShort**: 상대강도지수 기반 롱/숏 전략
- **BBand/BBandShort**: 볼린저밴드 기반 롱/숏 전략
- **MACD/MACDShort**: MACD 기반 롱/숏 전략
- **Copys/CopysShort**: 커스텀 롱/숏 전략
- **ThreeRSI/ThreeRSIShort**: 3개의 RSI 지표를 조합한 롱/숏 전략
- **Hybrid**: 여러 지표를 결합한 하이브리드 전략

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 
