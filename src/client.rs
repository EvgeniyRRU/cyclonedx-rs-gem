use anyhow::Result;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{default_on_request_failure, policies::ExponentialBackoff, Retryable, RetryableStrategy, RetryTransientMiddleware};
use reqwest::{Response, Client};
use reqwest_middleware::{ClientBuilder, Result as MiddlewareResult};

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

pub(crate) fn get_client() -> Result<ClientWithMiddleware> {
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let retry_middleware = RetryTransientMiddleware::new_with_policy_and_strategy(
        retry_policy,
        RetryAllExcept404,
    );
    let client = ClientBuilder::new(http)
        .with(retry_middleware)
        .build();

    Ok(client)
}
