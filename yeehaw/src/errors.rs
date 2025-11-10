use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    YeehawError {
        message: String,
    },

    #[snafu(context(false))]
    SliceError {
        source: std::array::TryFromSliceError,
        backtrace: Backtrace,
    },

    #[snafu(context(false))]
    IoError {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[cfg(feature = "textbox")]
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

    #[cfg(feature = "ratatui")]
    #[snafu(context(false))]
    RatatuiImageError {
        source: ratatui_image::errors::Errors,
        backtrace: Backtrace,
    },

    #[cfg(feature = "image")]
    #[snafu(context(false))]
    ImageError {
        source: image::ImageError,
        backtrace: Backtrace,
    },

    #[cfg(feature = "terminal")]
    #[snafu(context(false))]
    AnyhowError {
        source: anyhow::Error,
        backtrace: Backtrace,
    },

    #[cfg(feature = "bat")]
    #[snafu(context(false))]
    BatError {
        source: Box<bat::error::Error>,
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
