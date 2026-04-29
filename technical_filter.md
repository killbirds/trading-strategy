# 기술적 필터 설정 가이드

이 문서는 `src/filter/` 실제 구현을 기준으로 다시 정리한 **구현 기준 레퍼런스**입니다.

- 기준 소스: `src/filter/mod.rs`, `src/filter/*.rs`, `src/analyzer/slope_analyzer.rs`, `src/indicator/ma/mod.rs`, `src/strategy/copys_common.rs`
- 지원 필터 종류: **17개**
- 전체 `filter_type` variant 수: **320개**
- 실제 조합 예시는 `ta_filter_sample/` 문서를 참고하세요.

이전 문서에 있던 장세 해석/전략 추천 성격의 설명은 코드와 1:1로 대응되지 않는 부분이 많아서, 여기서는 **코드가 실제로 허용하는 설정면**만 정리합니다.

---

## 1. 공통 설정 규칙

### 기본 구조

```toml
[[filters]]
type = "RSI"
filter_type = "Overbought"
consecutive_n = 1
p = 0
```

### `type` 값

실제 `TechnicalFilterConfig` 가 받는 값은 아래와 같습니다.

- `RSI`
- `MACD`
- `BOLLINGER_BAND`
- `ADX`
- `MOVING_AVERAGE`
- `PRICE_REFERENCE_GAP`
- `ICHIMOKU`
- `VWAP`
- `COPYS`
- `ATR`
- `SUPERTREND`
- `VOLUME`
- `THREERSI`
- `CANDLEPATTERN`
- `SUPPORTRESISTANCE`
- `MOMENTUM`
- `SLOPE`

### `filter_type` 입력 규칙

- 대부분의 필터는 `filter_type` 에 **enum 문자열** 또는 **0부터 시작하는 정수 인덱스**를 넣을 수 있습니다.
- 예외적으로 `SLOPE` 는 현재 구현에서 **정수 인덱스를 지원하지 않고 문자열만 지원**합니다.
- 이 문서는 가독성을 위해 **문자열 enum 이름 기준**으로 설명합니다.

### 공통 필드

| 필드            | 의미                                  |
| --------------- | ------------------------------------- |
| `filter_type`   | 필터별 enum 이름                      |
| `consecutive_n` | 조건을 연속으로 만족해야 하는 캔들 수 |
| `p`             | 현재 캔들 기준 과거 오프셋            |

### 중첩 값 표기 규칙

- `reference_source.type`: `MOVING_AVERAGE`, `VWAP`, `HIGHEST_HIGH`, `LOWEST_LOW`
- `indicator_type.type`: `ClosePrice`, `HighPrice`, `LowPrice`, `MovingAverage`, `RSI`, `MACD`, `MACDLine`, `MACDSignalLine`, `MACDHistogram`
- `ma_type`: `EMA`, `SMA`, `WMA`
  - 단, `PriceReferenceGap` 의 `reference_source = { type = "MOVING_AVERAGE", ... }` 는 현재 `EMA` 와 `SMA` 만 허용합니다.

> 아래 최소 필요 캔들 수는 각 필터 함수의 **상위 guard** 기준입니다. 일부 교차/패턴 계열은 내부에서 추가 히스토리를 더 확인합니다.

---

## 2. 빠른 참조

| 필터              | `type` 값             | `filter_type` 수 | 최소 필요 캔들 수                                     |
| ----------------- | --------------------- | ---------------: | ----------------------------------------------------- |
| RSI               | `RSI`                 |               23 | `period + consecutive_n`                              |
| MACD              | `MACD`                |               21 | `slow_period + signal_period + consecutive_n`         |
| BollingerBand     | `BOLLINGER_BAND`      |               31 | `period`                                              |
| ADX               | `ADX`                 |               31 | `period * 2 + consecutive_n`                          |
| MovingAverage     | `MOVING_AVERAGE`      |               23 | `max(periods)`                                        |
| Ichimoku          | `ICHIMOKU`            |               13 | `senkou_span_b_period + kijun_period + consecutive_n` |
| VWAP              | `VWAP`                |               12 | `period + consecutive_n`                              |
| PriceReferenceGap | `PRICE_REFERENCE_GAP` |                4 | 참조 소스에 따라 다름                                 |
| CopyS             | `COPYS`               |               16 | `60`                                                  |
| ATR               | `ATR`                 |                7 | `max(period, consecutive_n)`                          |
| SuperTrend        | `SUPERTREND`          |                9 | `max(period, consecutive_n)`                          |
| Volume            | `VOLUME`              |               21 | `max(period, consecutive_n)`                          |
| ThreeRSI          | `THREERSI`            |               28 | `max(ma_period, consecutive_n)`                       |
| CandlePattern     | `CANDLEPATTERN`       |               41 | `max(pattern_history_length, consecutive_n)`          |
| SupportResistance | `SUPPORTRESISTANCE`   |               10 | `max(lookback_period, consecutive_n)`                 |
| Momentum          | `MOMENTUM`            |               21 | `max(history_length, consecutive_n)`                  |
| Slope             | `SLOPE`               |                9 | `period + consecutive_n`                              |

PriceReferenceGap 최소 필요 캔들 수:

- `MOVING_AVERAGE`, `VWAP`: `period + p + consecutive_n - 1`
- `HIGHEST_HIGH`, `LOWEST_LOW` + `include_current_candle = true`: `lookback_period + p + consecutive_n - 1`
- `HIGHEST_HIGH`, `LOWEST_LOW` + `include_current_candle = false`: `lookback_period + 1 + p + consecutive_n - 1`

---

## 3. 필터별 레퍼런스

아래 `filter_type` 목록은 모두 **실제 enum 선언 순서**입니다. 숫자 인덱스를 써야 한다면 이 순서의 **0-based index** 를 사용하면 됩니다. 단 `SLOPE` 는 문자열만 사용하세요.

### RSI

- 기본값: `period=14`, `oversold=30.0`, `overbought=70.0`, `filter_type="Overbought"`, `consecutive_n=1`, `p=0`, `sideways_threshold=0.02`, `momentum_threshold=3.0`, `cross_threshold=50.0`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`:

```text
Overbought, Oversold, NormalRange, CrossAboveThreshold, CrossBelowThreshold,
CrossAbove, CrossBelow, RisingTrend, FallingTrend, Sideways,
StrongRisingMomentum, StrongFallingMomentum, NeutralRange, Above40, Below60,
Above50, Below50, Divergence, Convergence, Stable, NeutralTrend, Bullish, Bearish
```

메모:

- `CrossAboveThreshold` / `CrossBelowThreshold` 는 `(oversold + overbought) / 2` 를 기준으로 동작합니다.
- `CrossAbove` / `CrossBelow` 가 `cross_threshold` 를 사용합니다.
- `Above50` / `Below50` 는 단순 `50 초과/미만` 이 아니라 **5개 RSI 값을 이용한 패턴 체크**입니다.
- `Bullish` 는 `60~80`, `Bearish` 는 `20~40` 범위 유지 체크입니다.

### MACD

- 기본값: `fast_period=12`, `slow_period=26`, `signal_period=9`, `filter_type="MacdAboveSignal"`, `consecutive_n=1`, `threshold=0.0`, `p=0`, `overbought_threshold=0.02`, `oversold_threshold=0.02`, `sideways_threshold=0.05`
- 최소 필요 캔들 수: `slow_period + signal_period + consecutive_n`
- `filter_type`:

```text
MacdAboveSignal, MacdBelowSignal, SignalCrossAbove, SignalCrossBelow,
HistogramAboveThreshold, HistogramBelowThreshold, ZeroLineCrossAbove,
ZeroLineCrossBelow, HistogramNegativeTurn, HistogramPositiveTurn,
StrongUptrend, StrongDowntrend, MacdRising, MacdFalling,
HistogramExpanding, HistogramContracting, Divergence, Convergence,
Overbought, Oversold, Sideways
```

### BollingerBand

- 기본값: `period=20`, `dev_mult=2.0`, `filter_type="AboveUpperBand"`, `consecutive_n=1`, `p=0`, `squeeze_threshold=0.02`, `medium_threshold=0.05`, `large_threshold=0.1`, `squeeze_breakout_period=5`, `enhanced_narrowing_period=3`, `enhanced_squeeze_period=2`, `upper_touch_threshold=0.99`, `lower_touch_threshold=1.01`
- 최소 필요 캔들 수: `period`
- `filter_type`:

```text
AboveUpperBand, BelowLowerBand, InsideBand, OutsideBand, AboveMiddleBand,
BelowMiddleBand, BandWidthSufficient, BreakThroughLowerBand, SqueezeBreakout,
EnhancedSqueezeBreakout, SqueezeState, BandWidthNarrowing, SqueezeExpansionStart,
BreakThroughUpperBand, BreakThroughLowerBandFromBelow, BandWidthExpanding,
MiddleBandSideways, UpperBandSideways, LowerBandSideways, BandWidthSideways,
UpperBandTouch, LowerBandTouch, BandWidthThresholdBreakthrough,
PriceMovingToUpperFromMiddle, PriceMovingToLowerFromMiddle,
BandConvergenceThenDivergence, BandDivergenceThenConvergence,
PriceMovingToUpperWithinBand, PriceMovingToLowerWithinBand,
LowVolatility, HighVolatility
```

메모: `BreakThroughLowerBand` 와 `BreakThroughLowerBandFromBelow` 는 현재 같은 구현을 사용합니다.

### ADX

- 기본값: `period=14`, `threshold=25.0`, `filter_type="BelowThreshold"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `period * 2 + consecutive_n`
- `threshold` 검증 범위: `0.0..=100.0`
- `filter_type`:

```text
BelowThreshold, AboveThreshold, PDIAboveMDI, MDIAbovePDI, StrongUptrend,
StrongDowntrend, ADXRising, ADXFalling, DIGapExpanding, DIGapContracting,
ExtremeHigh, ExtremeLow, MiddleLevel, PDICrossAboveMDI, MDICrossAbovePDI,
Sideways, Surge, Crash, StrongDirectionality, WeakDirectionality,
TrendStrengthHigherThanDirection, ADXHigherThanMDI, PDIHigherThanADX,
MDIHigherThanADX, TrendReversalDown, TrendReversalUp, DICrossover,
ExtremePDI, ExtremeMDI, Stable, Unstable
```

### MovingAverage

- 기본값: `periods=[5,20]`, `filter_type="PriceAboveFirstMA"`, `consecutive_n=1`, `p=0`, `sideways_threshold=0.02`, `crossover_threshold=0.005`
- 최소 필요 캔들 수: `max(periods)`
- `filter_type`:

```text
PriceAboveFirstMA, PriceAboveLastMA, RegularArrangement, FirstMAAboveLastMA,
FirstMABelowLastMA, GoldenCross, PriceBetweenMA, MAConvergence,
MADivergence, AllMAAbove, AllMABelow, ReverseArrangement, DeadCross,
MASideways, StrongUptrend, StrongDowntrend, PriceCrossingMA,
ConvergenceDivergence, DivergenceConvergence, ParallelMovement,
NearCrossover, PriceBelowFirstMA, PriceBelowLastMA
```

메모: 현재 구현은 내부에서 **항상 `SMA`** 를 사용합니다. `ma_type` 설정은 없습니다.

### Ichimoku

- 기본값: `tenkan_period=9`, `kijun_period=26`, `senkou_span_b_period=52`, `filter_type="PriceAboveCloud"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `senkou_span_b_period + kijun_period + consecutive_n`
- `filter_type`:

```text
PriceAboveCloud, PriceBelowCloud, TenkanAboveKijun, GoldenCross, DeadCross,
CloudBreakoutUp, CloudBreakdown, BuySignal, SellSignal, CloudThickening,
PerfectAlignment, PerfectReverseAlignment, StrongBuySignal
```

메모: `StrongBuySignal` 은 현재 `BuySignal` 과 같은 구현을 사용합니다.

### VWAP

- 기본값: `period=20`, `filter_type="PriceAboveVWAP"`, `consecutive_n=1`, `threshold=0.05`, `p=0`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`:

```text
PriceAboveVWAP, PriceBelowVWAP, PriceNearVWAP, VWAPBreakoutUp, VWAPBreakdown,
VWAPRebound, DivergingFromVWAP, ConvergingToVWAP, StrongUptrend,
StrongDowntrend, TrendStrengthening, TrendWeakening
```

메모:

- `StrongUptrend` = `PriceAboveVWAP`
- `StrongDowntrend` = `PriceBelowVWAP`
- `TrendStrengthening` = `DivergingFromVWAP`
- `TrendWeakening` = `ConvergingToVWAP`

### PriceReferenceGap

- 기본값: `reference_source={ type="MOVING_AVERAGE", ma_type="SMA", period=20 }`, `filter_type="GapAboveThreshold"`, `gap_threshold=0.02`, `consecutive_n=1`, `p=0`
- `gap_threshold` 검증 범위: `0.0..=1.0`
- `filter_type`:

```text
GapAboveThreshold, GapBelowThreshold,
GapAboveReferenceThreshold, GapBelowReferenceThreshold,
GapBelowReferenceUpperThreshold, GapAboveReferenceLowerThreshold
```

`reference_source`:

- `{ type = "MOVING_AVERAGE", ma_type = "EMA" | "SMA", period = N }`
- `{ type = "VWAP", period = N }`
- `{ type = "HIGHEST_HIGH", lookback_period = N, include_current_candle = true | false }`
- `{ type = "LOWEST_LOW", lookback_period = N, include_current_candle = true | false }`

메모:

- `GapAboveThreshold` / `GapBelowThreshold` 는 **절대 괴리율** 기준입니다.
- `GapAboveReferenceThreshold` 는 `gap_ratio >= threshold` 입니다.
- `GapBelowReferenceThreshold` 는 `gap_ratio <= -threshold` 입니다.
- `GapBelowReferenceUpperThreshold` 는 `gap_ratio <= threshold` 입니다.
- `GapAboveReferenceLowerThreshold` 는 `gap_ratio >= -threshold` 입니다.
- `p` 는 reference window 만이 아니라 **평가 캔들 자체도 과거로 이동**시킵니다.

`gap_ratio` 계산식:

```text
gap_ratio = (current_price - reference_price) / reference_price
```

`filter_type` 판단식:

| filter_type | 조건 | 의미 |
| --- | --- | --- |
| `GapAboveThreshold` | `abs(gap_ratio) >= threshold` | 기준가와의 괴리율이 임계값 이상 |
| `GapBelowThreshold` | `abs(gap_ratio) <= threshold` | 기준가와의 괴리율이 임계값 이하 |
| `GapAboveReferenceThreshold` | `gap_ratio >= threshold` | 기준가보다 임계값 이상 높음 |
| `GapBelowReferenceThreshold` | `gap_ratio <= -threshold` | 기준가보다 임계값 이상 낮음 |
| `GapBelowReferenceUpperThreshold` | `gap_ratio <= threshold` | 기준가 대비 상단 임계값 이하 |
| `GapAboveReferenceLowerThreshold` | `gap_ratio >= -threshold` | 기준가 대비 하단 임계값 이상 |

방향별 threshold 안쪽 범위를 확인하려면 필터를 조합합니다.

- 기준가 이상이면서 상단 `threshold` 이내: `GapAboveReferenceThreshold(gap_threshold=0.0)` + `GapBelowReferenceUpperThreshold(gap_threshold=N)`
- 기준가 이하이면서 하단 `threshold` 이내: `GapBelowReferenceThreshold(gap_threshold=0.0)` + `GapAboveReferenceLowerThreshold(gap_threshold=N)`

예시:

```toml
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowThreshold"
gap_threshold = 0.02
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

# 기준가 이상이면서 +5% 이내
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowReferenceUpperThreshold"
gap_threshold = 0.05
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }
```

### CopyS

- 기본값: `rsi_period=14`, `rsi_upper=70.0`, `rsi_lower=30.0`, `filter_type="BasicBuySignal"`, `consecutive_n=1`, `p=0`, `bband_period=20`, `bband_multiplier=2.0`, `ma_periods=[5,20,60,120,200,240]`
- 최소 필요 캔들 수: `60`
- `filter_type`:

```text
BasicBuySignal, BasicSellSignal, RSIOversold, RSIOverbought, BBandLowerTouch,
BBandUpperTouch, MASupport, MAResistance, StrongBuySignal, StrongSellSignal,
WeakBuySignal, WeakSellSignal, RSINeutral, BBandInside,
MARegularArrangement, MAReverseArrangement
```

메모: CopyS 는 내부 MA 컨텍스트를 현재 **EMA 고정**으로 사용합니다.

### ATR

- 기본값: `period=14`, `threshold=0.01`, `filter_type="AboveThreshold"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`: `AboveThreshold`, `VolatilityExpanding`, `VolatilityContracting`, `HighVolatility`, `LowVolatility`, `VolatilityIncreasing`, `VolatilityDecreasing`

### SuperTrend

- 기본값: `period=10`, `multiplier=3.0`, `filter_type="AllUptrend"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`: `AllUptrend`, `AllDowntrend`, `PriceAboveSupertrend`, `PriceBelowSupertrend`, `PriceCrossingAbove`, `PriceCrossingBelow`, `TrendChanged`, `Uptrend`, `Downtrend`

### Volume

- 기본값: `period=20`, `threshold=1.5`, `filter_type="VolumeAboveAverage"`, `consecutive_n=1`, `p=0`, `stable_min_threshold=0.1`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`:

```text
VolumeAboveAverage, VolumeBelowAverage, VolumeSurge, VolumeDecline,
VolumeSignificantlyAbove, BullishWithIncreasedVolume, BearishWithIncreasedVolume,
IncreasingVolumeInUptrend, DecreasingVolumeInDowntrend, VolumeSharpDecline,
VolumeStable, VolumeVolatile, BullishWithDecreasedVolume,
BearishWithDecreasedVolume, VolumeDoubleAverage, VolumeHalfAverage,
VolumeConsecutiveIncrease, VolumeConsecutiveDecrease, VolumeSideways,
VolumeExtremelyHigh, VolumeExtremelyLow
```

메모:

- `VolumeStable` 은 `threshold` 와 `stable_min_threshold` 중 큰 값을 사용합니다.
- `VolumeSharpDecline` 은 현재 `VolumeDecline` 과 같습니다.
- `VolumeVolatile` 은 현재 `VolumeSurge` 와 같습니다.

### ThreeRSI

- 기본값: `rsi_periods=[7,14,21]`, `ma_type="SMA"`, `ma_period=20`, `adx_period=14`, `filter_type="AllRSILessThan50"`, `consecutive_n=1`, `p=0`, `cross_threshold=50.0`
- 최소 필요 캔들 수: `max(ma_period, consecutive_n)`
- `filter_type`:

```text
AllRSILessThan50, AllRSIGreaterThan50, RSIReverseArrangement,
RSIRegularArrangement, CandleLowBelowMA, CandleHighAboveMA, ADXGreaterThan20,
AllRSILessThan30, AllRSIGreaterThan70, RSIStableRange, RSIBullishRange,
RSIBearishRange, RSIOverboughtRange, RSIOversoldRange, RSICrossAbove,
RSICrossBelow, RSISideways, RSIBullishMomentum, RSIBearishMomentum,
RSIDivergence, RSIConvergence, RSIDoubleBottom, RSIDoubleTop,
RSIOverboughtReversal, RSIOversoldReversal, RSINeutralTrend,
RSIExtremeOverbought, RSIExtremeOversold
```

메모:

- 런타임에서 `ma_type` 은 `EMA`, `WMA` 를 명시하면 그 값으로 사용하고, 그 외 문자열은 `SMA` 로 처리합니다.
- 여러 고급 이름이 현재는 `regular_arrangement` / `reverse_arrangement` / `sideways` 같은 기존 체크를 재사용합니다.

### CandlePattern

- 기본값: `min_body_ratio=0.3`, `min_shadow_ratio=0.3`, `pattern_history_length=5`, `threshold=0.5`, `filter_type="StrongBullishPattern"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(pattern_history_length, consecutive_n)`
- `pattern_history_length` 는 0일 수 없습니다.
- `filter_type` (41개):

```text
StrongBullishPattern, StrongBearishPattern, ReversalPattern, ContinuationPattern,
VolumeConfirmedPattern, HighReliabilityPattern, ContextAlignedPattern,
StrongReversalSignal, HighConfidenceSignal, VolumeConfirmedSignal,
PatternClusteringSignal, HammerPattern, ShootingStarPattern, DojiPattern,
SpinningTopPattern, MarubozuPattern, MorningStarPattern, EveningStarPattern,
EngulfingPattern, PiercingPattern, DarkCloudPattern, HaramiPattern,
TweezerPattern, TriStarPattern, AdvanceBlockPattern, DeliberanceBlockPattern,
BreakawayPattern, ConcealmentPattern, CounterattackPattern,
DarkCloudCoverPattern, RisingWindowPattern, FallingWindowPattern,
HighBreakoutPattern, LowBreakoutPattern, GapPattern, GapFillPattern,
DoubleBottomPattern, DoubleTopPattern, TrianglePattern, FlagPattern,
PennantPattern
```

메모: 여러 고급 패턴 이름이 현재는 continuation/reversal/strong bullish/strong bearish 같은 공통 신호를 재사용합니다.

### SupportResistance

- 기본값: `lookback_period=20`, `touch_threshold=0.01`, `min_touch_count=2`, `threshold=0.05`, `filter_type="SupportBreakdown"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(lookback_period, consecutive_n)`
- `min_touch_count` 는 0일 수 없습니다.
- `filter_type`: `SupportBreakdown`, `ResistanceBreakout`, `SupportBounce`, `ResistanceRejection`, `NearStrongSupport`, `NearStrongResistance`, `AboveSupport`, `BelowResistance`, `NearSupport`, `NearResistance`

### Momentum

- 기본값: `rsi_period=14`, `stoch_period=14`, `williams_period=14`, `roc_period=10`, `cci_period=20`, `momentum_period=10`, `history_length=50`, `threshold=0.5`, `filter_type="StrongPositiveMomentum"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(history_length, consecutive_n)`
- `filter_type`:

```text
StrongPositiveMomentum, StrongNegativeMomentum, AcceleratingMomentum,
DeceleratingMomentum, Overbought, Oversold, MomentumDivergence,
BullishDivergence, BearishDivergence, PersistentMomentum, StableMomentum,
MomentumReversalSignal, MomentumSideways, MomentumSurge, MomentumCrash,
MomentumConvergence, MomentumDivergencePattern, MomentumParallel,
MomentumCrossover, MomentumSupportTest, MomentumResistanceTest
```

메모: 여러 이름이 현재는 같은 analyzer 체크를 재사용합니다. 예를 들어 `MomentumSurge` 는 `StrongPositiveMomentum`, `MomentumCrash` 는 `StrongNegativeMomentum` 과 같은 구현입니다.

### Slope

- 기본값: `indicator_type=ClosePrice`, `period=20`, `filter_type="Upward"`, `consecutive_n=1`, `p=0`, `use_linear_regression=null`, `strength_threshold=null`, `r_squared_threshold=null`, `short_period=null`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`: `Upward`, `Downward`, `Sideways`, `StrengthAboveThreshold`, `Accelerating`, `Decelerating`, `StrongUpward`, `StrongDownward`, `HighRSquared`

유효 기본값:

- `use_linear_regression`: 기본 `false`
- `strength_threshold`: 기본 `0.02` (`Upward`, `Downward`, `StrongUpward`, `StrongDownward`), 기본 `0.01` (`StrengthAboveThreshold`)
- `r_squared_threshold`: 기본 `0.7`
- `short_period`: 기본 `period / 2`

`indicator_type`:

```toml
{ type = "ClosePrice" }
{ type = "HighPrice" }
{ type = "LowPrice" }
{ type = "MovingAverage", ma_type = "EMA", period = 20 }
{ type = "RSI", period = 14, ma_type = "SMA", ma_periods = [14] }
{ type = "MACD", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDLine", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDSignalLine", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDHistogram", fast_period = 12, slow_period = 26, signal_period = 9 }
```

메모:

- `indicator_type = { type = "RSI", ... }` 는 `period` 만으로는 부족하고 `ma_type`, `ma_periods` 도 필요합니다.
- 현재 구현에서 `consecutive_n` 은 주로 상위 최소 캔들 수 계산에만 반영되고, 각 `filter_type` 판단식에는 직접 쓰이지 않는 경우가 많습니다.
- `SLOPE` 는 `filter_type` 정수 인덱스를 지원하지 않습니다.

---

## 4. 구현상 주의할 점

1. `technical_filter.md` 에서 과거에 사용하던 일부 count/설명은 실제 코드와 달랐습니다. 이 문서는 코드 기준으로 다시 맞춘 버전입니다.
2. sample 문서(`ta_filter_sample/`)는 `filter_type` 에 숫자를 쓰는 경우가 많지만, 이 문서는 enum 문자열 기준으로 설명합니다.
3. `PriceReferenceGap` 은 절대 괴리와 방향성 괴리가 섞여 있으므로 이름을 정확히 구분해서 써야 합니다.
4. `MovingAverage`, `CopyS`, `ThreeRSI`, `CandlePattern`, `Momentum`, `VWAP` 은 일부 enum 이름이 내부적으로 같은 체크를 공유합니다.
5. 새 필터 타입이 추가되면 **반드시 `src/filter/mod.rs` 와 실제 `src/filter/*.rs` 구현을 함께 기준으로 문서를 갱신**해야 합니다.

---

## 5. 샘플 조합 문서

실전 조합 예시는 아래 문서를 참고하세요.

- `ta_filter_sample/ta_filter_sample01.md`
- `ta_filter_sample/ta_filter_sample02.md`
- `ta_filter_sample/ta_filter_sample04.md`
- `ta_filter_sample/ta_filter_simple02.md`
