#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Database,
    Io,
    Icon,
    Network,
    Config,
    Scan,
    NotFound,
    Validation,
    Busy,
    Timeout,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

/// 应用统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("图标提取失败: {0}")]
    Icon(String),
    #[error("网络错误: {0}")]
    Network(String),
    #[error("配置错误: {0}")]
    Config(String),
    #[error("扫描错误: {0}")]
    Scan(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    Validation(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let response = ErrorResponse::from(self);
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", &response.code)?;
        s.serialize_field("message", &response.message)?;
        s.end()
    }
}

impl From<&AppError> for ErrorResponse {
    fn from(e: &AppError) -> Self {
        let (code, message) = match e {
            AppError::Database(msg) => {
                if msg.contains("database is locked") || msg.contains("SQLITE_BUSY") {
                    (ErrorCode::Busy, "数据库繁忙，请稍后重试".to_string())
                } else {
                    (ErrorCode::Database, msg.clone())
                }
            }
            AppError::Io(err) => (ErrorCode::Io, err.to_string()),
            AppError::Icon(msg) => (ErrorCode::Icon, msg.clone()),
            AppError::Network(msg) => (ErrorCode::Network, msg.clone()),
            AppError::Config(msg) => (ErrorCode::Config, msg.clone()),
            AppError::Scan(msg) => (ErrorCode::Scan, msg.clone()),
            AppError::NotFound(msg) => (ErrorCode::NotFound, msg.clone()),
            AppError::Validation(msg) => (ErrorCode::Validation, msg.clone()),
        };
        ErrorResponse { code, message }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}
