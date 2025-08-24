#[derive(Debug)]
pub enum YoutubeError {
    ApiKeyNotFound,
    ApiRequestError(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for YoutubeError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        YoutubeError::ApiRequestError(error)
    }
}

impl std::fmt::Display for YoutubeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YoutubeError::ApiKeyNotFound => write!(f, "🩻🩻 API key not found in YouTube config.. 🩻🩻"),
            YoutubeError::ApiRequestError(e) => write!(f, "🩻🩻 The API request for comment data failed. {} 🩻🩻", e),
        }
    }
}

impl std::error::Error for YoutubeError {}
