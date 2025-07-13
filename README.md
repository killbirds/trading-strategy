# Trading-Strategy

볼린저 밴드 스퀴즈 돌파 전략을 포함한 다양한 트레이딩 전략 라이브러리입니다.

## 주요 기능

### 볼린저 밴드 스퀴즈 돌파 전략

향상된 볼린저 밴드 전략은 다음과 같은 정교한 패턴을 감지합니다:

1. **밴드 폭 감소 단계**: 일정 기간 동안 밴드 폭이 연속적으로 감소
2. **스퀴즈 유지 단계**: 좁아진 상태를 일정 기간 유지
3. **돌파 단계**: 고가가 상단을 돌파하고 종가가 상단 위에 위치

#### 설정 파라미터

- `count`: 확인 캔들 수
- `period`: 볼린저 밴드 계산 기간 (기본: 20)
- `multiplier`: 볼린저 밴드 승수 (기본: 2.0)
- `narrowing_period`: 밴드 폭 감소 확인 기간 (기본: 5)
- `squeeze_period`: 좁은 상태 유지 기간 (기본: 5)
- `squeeze_threshold`: 스퀴즈 임계값 (기본: 0.02)

#### 사용 예시

```rust
// 기본 설정 사용
let strategy = BBandStrategy::new_with_config(&storage, None)?;

// 커스텀 설정 사용
let mut config = HashMap::new();
config.insert("period".to_string(), "20".to_string());
config.insert("multiplier".to_string(), "2.0".to_string());
config.insert("narrowing_period".to_string(), "7".to_string());
config.insert("squeeze_period".to_string(), "8".to_string());
config.insert("squeeze_threshold".to_string(), "0.015".to_string());

let strategy = BBandStrategy::new_with_config(&storage, Some(config))?;
```

#### 설정 파일 예시

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

## 빌드 및 실행

```bash
cargo build --release
```

## 테스트

```bash
cargo test
```

