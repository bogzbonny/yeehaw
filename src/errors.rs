use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    //#[snafu(context(false))]
    //GitErr {
    //    source: git2::Error,
    //    backtrace: Backtrace,
    //},

    //#[snafu(context(false))]
    //SerdeYamlError {
    //    source: serde_yml::Error,
    //    backtrace: Backtrace,
    //},
    #[snafu(context(false))]
    SliceError {
        source: std::array::TryFromSliceError,
        backtrace: Backtrace,
    },

    //CbaError {
    //    message: String,
    //    backtrace: Backtrace,
    //},
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
}

//impl Error {
//    pub fn cba_error(message: &str) -> Self {
//        CbaSnafu { message }.build()
//    }
//}
