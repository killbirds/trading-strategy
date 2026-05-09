# 간단한 매수 필터 조합 06: 독립 standalone 지표 필터 예시

이 예시는 Donchian, KAMA, PPO처럼 새로 추가된 standalone 지표가 하나의 `MARKET_INDICATOR` 묶음이 아니라 각각 독립된 필터 타입으로 설정되는 방식을 보여줍니다.

```toml
[[filters]]
type = "DONCHIAN"
period = 20
filter_type = "BreakoutAbove"
consecutive_n = 1
p = 0
edge_threshold = 0.1

[[filters]]
type = "KAMA"
period = 10
fast_period = 2
slow_period = 30
filter_type = "PriceAbove"
consecutive_n = 1
p = 0
er_threshold = 0.6
er_low_threshold = 0.2

[[filters]]
type = "PPO"
fast_period = 12
slow_period = 26
signal_period = 9
filter_type = "AboveSignal"
consecutive_n = 1
p = 0
```

## 사용 가능한 standalone 필터 타입

- `DONCHIAN`
- `KELTNER`
- `OBV`
- `MFI`
- `AROON`
- `CHOPPINESS`
- `KAMA`
- `CHAIKIN`
- `PPO`
- `PARABOLIC_SAR`

가격 비교가 필요한 조건은 캔들의 최신 종가가 아니라 호출자가 전달한 외부 `current_price` 를 기준으로 평가합니다.
