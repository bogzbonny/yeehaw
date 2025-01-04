use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    SliceError {
        source: std::array::TryFromSliceError,
        backtrace: Backtrace,
    },

    YeehawError {
        message: String,
    },

    #[snafu(context(false))]
    IoError {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    ArboardError {
        source: arboard::Error,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    TokioError {
        source: tokio::sync::watch::error::SendError<bool>,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    RatatuiImageError {
        source: ratatui_image::errors::Errors,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    ImageError {
        source: image::ImageError,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    AnyhowError {
        source: anyhow::Error,
        backtrace: Backtrace,
    },
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error::YeehawError {
            message: message.to_string(),
        }
    }
}
