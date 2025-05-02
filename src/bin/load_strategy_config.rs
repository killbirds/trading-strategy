use log::{debug, error, info, warn};
use std::env;
use std::path::PathBuf;
use trading_chart::OhlcvCandle;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::strategy::{StrategyFactory, StrategyType};

fn main() {
    // 로그 초기화
    env_logger::init();

    info!("전략 설정 로더 시작");
    debug!("커맨드 라인 인수 파싱 시작");

    // 커맨드 라인 인수 파싱
    let args: Vec<String> = env::args().collect();
    debug!("커맨드 라인 인수: {:?}", args);

    if args.len() < 2 {
        error!("인수가 충분하지 않습니다. 전략 타입이 필요합니다.");
        println!("사용법: {} <전략_타입> [설정_파일_경로]", args[0]);
        println!(
            "지원되는 전략 타입: bband, bband_short, macd, macd_short, ma, ma_short, rsi, rsi_short 등"
        );
        return;
    }

    // 전략 타입 파싱
    let strategy_type_str = &args[1];
    debug!("전략 타입 문자열: {}", strategy_type_str);

    let strategy_type = match strategy_type_str.as_str() {
        "bband" => StrategyType::BBand,
        "bband_short" => StrategyType::BBandShort,
        "macd" => StrategyType::MACD,
        "macd_short" => StrategyType::MACDShort,
        "ma" => StrategyType::MA,
        "ma_short" => StrategyType::MAShort,
        "rsi" => StrategyType::RSI,
        "rsi_short" => StrategyType::RSIShort,
        "dummy" => StrategyType::Dummy,
        _ => {
            let error_msg = format!("지원되지 않는 전략 타입: {}", strategy_type_str);
            error!("{}", error_msg);
            println!("{}", error_msg);
            println!(
                "지원되는 전략 타입: bband, bband_short, macd, macd_short, ma, ma_short, rsi, rsi_short, dummy"
            );
            return;
        }
    };

    info!("전략 타입 파싱 완료: {:?}", strategy_type);

    // 설정 파일 경로 (지정되지 않은 경우 기본 경로 사용)
    let config_path = if args.len() >= 3 {
        debug!("사용자 지정 설정 파일 사용: {}", args[2]);
        PathBuf::from(&args[2])
    } else {
        debug!("기본 설정 파일 경로 사용");
        let path = StrategyFactory::default_config_path(strategy_type);
        debug!("기본 설정 파일 경로: {}", path.display());
        path
    };

    if !config_path.exists() {
        warn!("설정 파일이 존재하지 않습니다: {}", config_path.display());
        println!(
            "경고: 설정 파일이 존재하지 않습니다: {}",
            config_path.display()
        );
        println!("전략이 기본 설정으로 로드될 수 있습니다.");
    }

    info!(
        "전략 타입: {:?}, 설정 파일: {}",
        strategy_type,
        config_path.display()
    );
    println!("전략 타입: {:?}", strategy_type);
    println!("설정 파일: {}", config_path.display());

    // 빈 캔들 저장소 생성 (예제용으로만 사용)
    debug!("빈 캔들 저장소 생성");
    let storage = CandleStore::<OhlcvCandle>::new(vec![], 1000, true);

    // 설정 파일에서 전략 로드
    info!("전략 로드 시작");
    match StrategyFactory::build_from_config(strategy_type, &storage, &config_path) {
        Ok(strategy) => {
            let success_msg = format!("전략 로드 성공: {}", strategy);
            info!("{}", success_msg);
            println!("전략 로드 성공:");
            println!("{}", strategy);
        }
        Err(err) => {
            let error_msg = format!("전략 로드 실패: {}", err);
            error!("{}", error_msg);
            println!("전략 로드 실패: {}", err);

            // 에러 종류에 따라 추가 정보 제공
            if err.to_string().contains("파일이 존재하지 않습니다") {
                println!("해결 방법: 설정 파일 경로를 확인하거나 기본 설정을 사용하세요.");
            } else if err.to_string().contains("파싱 실패") {
                println!("해결 방법: 설정 파일 형식이 올바른지 확인하세요.");
            } else if err.to_string().contains("유효성 검사") {
                println!("해결 방법: 설정 값이 유효 범위 내에 있는지 확인하세요.");
            }
        }
    }

    info!("전략 설정 로더 종료");
}
