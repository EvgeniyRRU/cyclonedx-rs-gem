use anyhow::Result;
use reqwest::{Client, Response};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::{ClientBuilder, Result as MiddlewareResult};
use reqwest_retry::{
    RetryTransientMiddleware, Retryable, RetryableStrategy, default_on_request_failure,
    policies::ExponentialBackoff,
};

///
/// Strategy for retry all failed requests, except 404
/// (gem not found)
///
struct RetryAllExcept404;
impl RetryableStrategy for RetryAllExcept404 {
    fn handle(&self, res: &MiddlewareResult<Response>) -> Option<Retryable> {
        match res {
            // don't repeat if gem not found
            Ok(success) if success.status() == 404 => None,
            Ok(_) => Some(Retryable::Transient),
            Err(error) => default_on_request_failure(error),
        }
    }
}

///
/// Configure reqwest http client with custom retry strategy
///
pub(crate) fn get_client() -> Result<ClientWithMiddleware> {
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let retry_middleware =
        RetryTransientMiddleware::new_with_policy_and_strategy(retry_policy, RetryAllExcept404);
    let client = ClientBuilder::new(http).with(retry_middleware).build();

    Ok(client)
}

///
/// Configure reqwest http client for nexus requests
///
pub(crate) fn get_nexus_client() -> Result<ClientWithMiddleware> {
    let http = Client::builder().build()?;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
    let client = ClientBuilder::new(http)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    Ok(client)
}
