use thiserror::Error;

///
/// Errors enum, it required for retry failed requests
/// 
#[derive(Error, Debug)]
pub(crate) enum FetchPackageError {
    #[error("Could not send request for `{0}` for gem `{1}`")]
    SendRequestError(String, String),
    #[error("Could not receive response text for `{0}` for gem `{1}`")]
    ResponseTextError(String, String),
    #[error("Could not find version `{0}` for gem `{1}`")]
    VersionNotFound(String, String),
    #[error("Gem not found: `{0}`, version `{1}`")]
    PackageNotFound(String, String),
    #[error("Client error occurred for gem `{0}`, version `{1}`")]
    ClientError(String, String),
    #[error("Server error occurred for gem `{0}`, version `{1}`")]
    ServerError(String, String),
    #[error("Unknown error occurred for gem `{0}`, version `{1}`")]
    UnknownError(String, String),
}
