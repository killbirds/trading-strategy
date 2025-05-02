use log::{debug, error, info, warn};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 설정 로드 오류
#[derive(Debug)]
pub enum ConfigError {
    /// 파일 오류
    FileError(String),
    /// 파싱 오류
    ParseError(String),
    /// 유효성 검사 오류
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileError(msg) => write!(f, "설정 파일 오류: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "설정 파싱 오류: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "설정 유효성 검사 오류: {}", msg),
        }
    }
}

/// String으로 ConfigError 변환
impl From<ConfigError> for String {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::FileError(msg) => format!("설정 파일 오류: {}", msg),
            ConfigError::ParseError(msg) => format!("설정 파싱 오류: {}", msg),
            ConfigError::ValidationError(msg) => format!("설정 유효성 검사 오류: {}", msg),
        }
    }
}

/// 설정 로드 결과
pub type ConfigResult<T> = Result<T, ConfigError>;

/// 설정 형식
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON 형식
    Json,
    /// TOML 형식
    Toml,
    /// 자동 감지 (파일 확장자로부터)
    Auto,
}

/// 설정 로더 트레이트
pub trait ConfigValidation {
    /// 설정 유효성 검사
    fn validate(&self) -> ConfigResult<()>;
}

/// 설정 파일 로더
#[derive(Debug)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// 파일에서 설정 로드
    ///
    /// # Arguments
    /// * `path` - 설정 파일 경로
    /// * `format` - 설정 파일 형식 (기본값: Auto)
    ///
    /// # Returns
    /// * `ConfigResult<T>` - 설정 객체 또는 오류
    pub fn load_from_file<T>(path: &Path, format: ConfigFormat) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
    {
        debug!("설정 파일 로드 시작: {}", path.display());

        let format = if format == ConfigFormat::Auto {
            match Self::detect_format(path) {
                Ok(fmt) => {
                    debug!("설정 파일 형식 감지됨: {:?}", fmt);
                    fmt
                }
                Err(e) => {
                    error!("설정 파일 형식 감지 실패: {}", path.display());
                    return Err(e);
                }
            }
        } else {
            format
        };

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("설정 파일 열기 실패: {} - {}", path.display(), e);
                return Err(ConfigError::FileError(format!("파일 열기 실패: {}", e)));
            }
        };

        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            error!("설정 파일 읽기 실패: {} - {}", path.display(), e);
            return Err(ConfigError::FileError(format!("파일 읽기 실패: {}", e)));
        }

        let config: T = match format {
            ConfigFormat::Json => {
                debug!("JSON 설정 파일 파싱 시작");
                match Self::parse_json(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("JSON 설정 파일 파싱 실패: {} - {}", path.display(), e);
                        return Err(e);
                    }
                }
            }
            ConfigFormat::Toml => {
                debug!("TOML 설정 파일 파싱 시작");
                match Self::parse_toml(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("TOML 설정 파일 파싱 실패: {} - {}", path.display(), e);
                        return Err(e);
                    }
                }
            }
            ConfigFormat::Auto => unreachable!(),
        };

        // 유효성 검사
        if let Err(e) = config.validate() {
            error!("설정 유효성 검사 실패: {}", e);
            return Err(e);
        }

        info!("설정 파일 로드 완료: {}", path.display());
        Ok(config)
    }

    /// 문자열에서 설정 로드
    ///
    /// # Arguments
    /// * `content` - 설정 문자열
    /// * `format` - 설정 형식
    ///
    /// # Returns
    /// * `ConfigResult<T>` - 설정 객체 또는 오류
    pub fn load_from_string<T>(content: &str, format: ConfigFormat) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
    {
        let config: T = match format {
            ConfigFormat::Json => Self::parse_json(content)?,
            ConfigFormat::Toml => Self::parse_toml(content)?,
            ConfigFormat::Auto => {
                // JSON으로 먼저 시도 후 실패하면 TOML 시도
                match Self::parse_json::<T>(content) {
                    Ok(config) => config,
                    Err(_) => Self::parse_toml(content)?,
                }
            }
        };

        // 유효성 검사
        config.validate()?;

        Ok(config)
    }

    /// HashMap에서 설정 로드
    ///
    /// # Arguments
    /// * `map` - 설정 맵
    ///
    /// # Returns
    /// * `ConfigResult<T>` - 설정 객체 또는 오류
    pub fn load_from_map<T, V>(map: &HashMap<String, V>) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
        V: Serialize,
    {
        // HashMap을 JSON으로 변환 후 다시 역직렬화
        let json = serde_json::to_string(map)
            .map_err(|e| ConfigError::ParseError(format!("맵을 JSON으로 변환 실패: {}", e)))?;

        let config: T = Self::parse_json(&json)?;

        // 유효성 검사
        config.validate()?;

        Ok(config)
    }

    /// 설정 파일 저장
    ///
    /// # Arguments
    /// * `config` - 설정 객체
    /// * `path` - 저장할 파일 경로
    /// * `format` - 설정 파일 형식
    ///
    /// # Returns
    /// * `ConfigResult<()>` - 성공 또는 오류
    pub fn save_to_file<T>(config: &T, path: &Path, format: ConfigFormat) -> ConfigResult<()>
    where
        T: Serialize + ConfigValidation,
    {
        debug!("설정 파일 저장 시작: {}", path.display());

        let format = if format == ConfigFormat::Auto {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("json") => {
                    debug!("파일 확장자에 따라 JSON 형식 선택됨");
                    ConfigFormat::Json
                }
                Some("toml") => {
                    debug!("파일 확장자에 따라 TOML 형식 선택됨");
                    ConfigFormat::Toml
                }
                _ => {
                    debug!("확장자 없음, 기본 TOML 형식 선택됨");
                    ConfigFormat::Toml // 기본값은 TOML
                }
            }
        } else {
            format
        };

        let content = match format {
            ConfigFormat::Json => match serde_json::to_string_pretty(config) {
                Ok(content) => content,
                Err(e) => {
                    error!("JSON 직렬화 실패: {}", e);
                    return Err(ConfigError::ParseError(format!("JSON 직렬화 실패: {}", e)));
                }
            },
            ConfigFormat::Toml => match toml::to_string_pretty(config) {
                Ok(content) => content,
                Err(e) => {
                    error!("TOML 직렬화 실패: {}", e);
                    return Err(ConfigError::ParseError(format!("TOML 직렬화 실패: {}", e)));
                }
            },
            ConfigFormat::Auto => unreachable!(),
        };

        if let Err(e) = std::fs::write(path, &content) {
            error!("설정 파일 쓰기 실패: {} - {}", path.display(), e);
            return Err(ConfigError::FileError(format!("파일 쓰기 실패: {}", e)));
        }

        info!("설정 파일 저장 완료: {}", path.display());
        Ok(())
    }

    // 내부 헬퍼 메서드

    /// JSON 파싱
    fn parse_json<T: DeserializeOwned>(content: &str) -> ConfigResult<T> {
        match serde_json::from_str(content) {
            Ok(obj) => Ok(obj),
            Err(e) => {
                warn!("JSON 파싱 실패: {}", e);
                Err(ConfigError::ParseError(format!("JSON 파싱 실패: {}", e)))
            }
        }
    }

    /// TOML 파싱
    fn parse_toml<T: DeserializeOwned>(content: &str) -> ConfigResult<T> {
        match toml::from_str(content) {
            Ok(obj) => Ok(obj),
            Err(e) => {
                warn!("TOML 파싱 실패: {}", e);
                Err(ConfigError::ParseError(format!("TOML 파싱 실패: {}", e)))
            }
        }
    }

    /// 파일 형식 감지
    fn detect_format(path: &Path) -> ConfigResult<ConfigFormat> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                debug!("JSON 파일 형식 감지됨: {}", path.display());
                Ok(ConfigFormat::Json)
            }
            Some("toml") => {
                debug!("TOML 파일 형식 감지됨: {}", path.display());
                Ok(ConfigFormat::Toml)
            }
            _ => {
                warn!("지원되지 않는 파일 형식: {}", path.display());
                Err(ConfigError::FileError(format!(
                    "파일 형식을 감지할 수 없음: {}",
                    path.display()
                )))
            }
        }
    }
}

/// 설정 객체 변환 유틸리티
pub trait ConfigConversion<T> {
    /// 다른 유형의 설정으로 변환
    fn convert(self) -> ConfigResult<T>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::NamedTempFile;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig {
        pub name: String,
        pub value: i32,
        pub enabled: bool,
    }

    impl ConfigValidation for TestConfig {
        fn validate(&self) -> ConfigResult<()> {
            if self.value < 0 {
                return Err(ConfigError::ValidationError(
                    "value는 0 이상이어야 합니다".to_string(),
                ));
            }
            Ok(())
        }
    }

    #[test]
    fn test_load_from_json_string() {
        let json = r#"{"name":"test","value":42,"enabled":true}"#;
        let config =
            ConfigLoader::load_from_string::<TestConfig>(json, ConfigFormat::Json).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        assert!(config.enabled);
    }

    #[test]
    fn test_load_from_toml_string() {
        let toml_str = r#"
            name = "test"
            value = 42
            enabled = true
        "#;
        let config =
            ConfigLoader::load_from_string::<TestConfig>(toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        assert!(config.enabled);
    }

    #[test]
    fn test_validation_error() {
        let json = r#"{"name":"test","value":-1,"enabled":true}"#;
        let result = ConfigLoader::load_from_string::<TestConfig>(json, ConfigFormat::Json);
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError(_)) => (),
            _ => panic!("유효성 검사 오류가 발생해야 함"),
        }
    }

    #[test]
    fn test_save_and_load_file() {
        let config = TestConfig {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };

        // JSON 저장 및 로드 테스트
        let json_file = NamedTempFile::new().unwrap();
        let json_path = json_file.path().with_extension("json");
        let _ = std::fs::rename(json_file.path(), &json_path);

        ConfigLoader::save_to_file(&config, &json_path, ConfigFormat::Json).unwrap();
        let loaded_json =
            ConfigLoader::load_from_file::<TestConfig>(&json_path, ConfigFormat::Json).unwrap();
        assert_eq!(loaded_json.name, "test");

        // TOML 저장 및 로드 테스트
        let toml_file = NamedTempFile::new().unwrap();
        let toml_path = toml_file.path().with_extension("toml");
        let _ = std::fs::rename(toml_file.path(), &toml_path);

        ConfigLoader::save_to_file(&config, &toml_path, ConfigFormat::Toml).unwrap();
        let loaded_toml =
            ConfigLoader::load_from_file::<TestConfig>(&toml_path, ConfigFormat::Toml).unwrap();
        assert_eq!(loaded_toml.name, "test");
    }
}
