pub const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    #[error("the response was larger than {0} bytes")]
    TooLarge(usize),
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
}

pub async fn read_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, BodyError> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(BodyError::TooLarge(limit));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > limit {
            return Err(BodyError::TooLarge(limit));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_of(length: usize) -> reqwest::Response {
        reqwest::Response::from(http::Response::new(vec![b'x'; length]))
    }

    #[tokio::test]
    async fn a_body_at_the_limit_is_read_in_full() {
        let body = read_limited(response_of(64), 64).await.expect("body");
        assert_eq!(body.len(), 64);
    }

    #[tokio::test]
    async fn a_body_over_the_limit_is_refused() {
        let refused = read_limited(response_of(65), 64).await;
        assert!(matches!(refused, Err(BodyError::TooLarge(64))));
    }
}
