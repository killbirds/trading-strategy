# Trading-Strategy

Rust로 구현된 포괄적인 트레이딩 전략 라이브러리입니다. 다양한 기술적 지표와 분석기를 활용한 매매 전략을 제공합니다.

## 주요 특징

- **다양한 트레이딩 전략**: 이동평균, RSI, 볼린저 밴드, MACD 등 다양한 기술적 지표 기반 전략
- **롱/숏 전략 지원**: 각 전략의 롱 및 숏 버전 제공
- **고급 분석 도구**: 여러 지표를 결합한 하이브리드 전략 및 멀티 타임프레임 분석
- **유연한 설정**: TOML 및 JSON 형식의 설정 파일 지원
- **확장 가능한 구조**: 모듈화된 설계로 새로운 전략 추가 용이

## 지원하는 전략

### 기본 전략

- **MA (Moving Average)**: 이동평균선 기반 전략
- **RSI**: 상대강도지수 기반 전략
- **BBand (Bollinger Band)**: 볼린저 밴드 기반 전략
- **MACD**: MACD 지표 기반 전략
- **ThreeRSI**: 3개의 RSI 지표를 조합한 전략
- **Copys**: 커스텀 복합 전략

### 고급 전략

- **Hybrid**: 여러 지표를 결합한 하이브리드 전략
- **MultiTimeframe**: 여러 타임프레임을 분석하는 전략

각 전략은 롱(Long) 및 숏(Short) 버전을 제공합니다.

## 프로젝트 구조

```
src/
├── analyzer/          # 기술적 지표 분석기
│   ├── adx_analyzer.rs
│   ├── atr_analyzer.rs
│   ├── bband_analyzer.rs
│   ├── macd_analyzer.rs
│   ├── rsi_analyzer.rs
│   └── ...
├── filter/            # 기술적 필터링 기준
│   ├── adx.rs
│   ├── bollinger_band.rs
│   ├── macd.rs
│   └── ...
├── indicator/         # 기술적 지표 계산
│   ├── bband.rs
│   ├── macd.rs
│   ├── rsi.rs
│   └── ...
├── strategy/          # 트레이딩 전략 구현
│   ├── bband_strategy.rs
│   ├── macd_strategy.rs
│   ├── rsi_strategy.rs
│   └── ...
├── candle_store.rs    # 캔들 데이터 저장소
└── model.rs           # 데이터 모델
```

## 설치

### 요구사항

- Rust 1.70 이상
- Cargo

### 빌드

```bash
cargo build --release
```

### 의존성

주요 의존성:
- `chrono`: 날짜 및 시간 처리
- `serde`: 직렬화/역직렬화
- `toml`: TOML 설정 파일 파싱
- `serde_json`: JSON 설정 파일 파싱
- `anyhow`: 에러 처리

## 사용 방법

### 기본 사용 예시

```rust
use trading_strategy::strategy::BBandStrategy;
use trading_strategy::candle_store::CandleStore;
use std::collections::HashMap;

// 캔들 저장소 생성
let storage = CandleStore::new();

// 기본 설정으로 전략 생성
let strategy = BBandStrategy::new_with_config(&storage, None)?;

// 커스텀 설정으로 전략 생성
let mut config = HashMap::new();
config.insert("period".to_string(), "20".to_string());
config.insert("multiplier".to_string(), "2.0".to_string());
config.insert("narrowing_period".to_string(), "7".to_string());
config.insert("squeeze_period".to_string(), "8".to_string());
config.insert("squeeze_threshold".to_string(), "0.015".to_string());

let strategy = BBandStrategy::new_with_config(&storage, Some(config))?;
```

## 볼린저 밴드 스퀴즈 돌파 전략

향상된 볼린저 밴드 전략은 다음과 같은 정교한 패턴을 감지합니다:

1. **밴드 폭 감소 단계**: 일정 기간 동안 밴드 폭이 연속적으로 감소
2. **스퀴즈 유지 단계**: 좁아진 상태를 일정 기간 유지
3. **돌파 단계**: 고가가 상단을 돌파하고 종가가 상단 위에 위치

### 설정 파라미터

- `count`: 확인 캔들 수
- `period`: 볼린저 밴드 계산 기간 (기본: 20)
- `multiplier`: 볼린저 밴드 승수 (기본: 2.0)
- `narrowing_period`: 밴드 폭 감소 확인 기간 (기본: 5)
- `squeeze_period`: 좁은 상태 유지 기간 (기본: 5)
- `squeeze_threshold`: 스퀴즈 임계값 (기본: 0.02)

### 설정 파일 예시

**TOML 설정 (`config/bband_squeeze_strategy.toml`)**:
```toml
count = 2
period = 20
multiplier = 2.0
narrowing_period = 7
squeeze_period = 8
squeeze_threshold = 0.015
```

**JSON 설정 (`config/bband_squeeze.json`)**:
```json
{
  "count": 2,
  "period": 20,
  "multiplier": 2.0,
  "narrowing_period": 6,
  "squeeze_period": 6,
  "squeeze_threshold": 0.02
}
```

### 주요 메서드

#### BBandAnalyzer

- `is_band_width_narrowing(n)`: 밴드 폭이 좁아지는지 확인
- `is_band_width_squeeze(n, threshold)`: 밴드 폭이 좁은 상태인지 확인
- `is_narrowing_then_squeeze_pattern(narrowing_period, squeeze_period, threshold)`: 좁아지다가 좁은 상태 유지 패턴 확인
- `is_enhanced_squeeze_breakout_with_close_above_upper(narrowing_period, squeeze_period, threshold)`: 향상된 스퀴즈 돌파 패턴 확인
- `is_squeeze_expansion_start(threshold)`: 스퀴즈 상태에서 밴드 폭 확대 시작 확인

## 기술적 분석기

라이브러리는 다양한 기술적 지표 분석기를 제공합니다:

- **ADX Analyzer**: 추세 강도 분석
- **ATR Analyzer**: 평균 진폭 분석
- **BBand Analyzer**: 볼린저 밴드 분석
- **MACD Analyzer**: MACD 지표 분석
- **RSI Analyzer**: 상대강도지수 분석
- **MA Analyzer**: 이동평균 분석
- **Ichimoku Analyzer**: 이치모쿠 분석
- **SuperTrend Analyzer**: SuperTrend 분석
- **Volume Analyzer**: 거래량 분석
- **VWAP Analyzer**: VWAP 분석
- **Hybrid Analyzer**: 여러 지표를 결합한 분석

## 기술적 필터

다음과 같은 기술적 필터를 제공합니다:

- RSI, MACD, 볼린저 밴드, ADX, 이동평균선
- 이치모쿠, VWAP, ATR, SuperTrend
- 거래량, ThreeRSI, 캔들 패턴
- 지지/저항, 모멘텀

## 테스트

```bash
# 모든 테스트 실행
cargo test

# 특정 테스트 실행
cargo test --test bband_strategy_tests

# 문서 테스트 포함
cargo test --doc
```

## 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다.

## 기여

버그 리포트, 기능 제안, 풀 리퀘스트를 환영합니다.
