use thiserror::Error;

///
/// Errors for requests to rubygems.org
///
#[derive(Error, Debug)]
pub(crate) enum FetchPackageError {
    #[error("Could not send request for gem {0} version {1}")]
    SendRequestError(String, String),
    #[error("Could not parse response for gem {0} version {1}")]
    ParseResponseError(String, String),
    #[error("Could not find version {0} for gem {1}")]
    VersionNotFound(String, String),
    #[error("Gem not found: {0}, version {1}")]
    PackageNotFound(String, String),
    #[error("Client error occurred for gem {0}, version {1}")]
    ClientError(String, String),
    #[error("Server error occurred for gem {0}, version {1}")]
    ServerError(String, String),
    #[error("Unknown error occurred for gem {0}, version {1}")]
    UnknownError(String, String),
}

///
/// Errors for requests to nexus repository
///
#[derive(Error, Debug)]
pub(crate) enum NexusError {
    #[error("Incorrect Nexus url: {0}")]
    UrlParse(String),
    #[error("Failed to build http client for Nexus")]
    BuildClient,
    #[error("Could not send request to Nexus for gem {0} version {1}")]
    SendRequest(String, String),
    #[error("Could not parse Nexus response for gem {0} version {1}")]
    ParseResponse(String, String),
}
