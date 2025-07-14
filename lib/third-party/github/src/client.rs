use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http::Uri;
use jsonwebtoken as jwt;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use reqwest_middleware::Middleware;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};
use url::Url;

use crate::models::pulls;

// TODO: confirm
const TOKEN_ENDPOINT: &str = "https://github.com/oauth/token";
const DEFAULT_HOST: &str = "https://api.github.com";
const DEFAULT_CLIENT_AGENT: &str = "doctavious-github";

// We use 9 minutes for the life to give some buffer for clock drift between
// our clock and GitHub's. The absolute max is 10 minutes.
const MAX_JWT_TOKEN_LIFE: Duration = Duration::from_secs(60 * 9);
// 8 minutes so we refresh sooner than it actually expires
const JWT_TOKEN_REFRESH_PERIOD: Duration = Duration::from_secs(60 * 8);

mod support {
    use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

    const PATH_SET: &AsciiSet = &CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'#')
        .add(b'<')
        .add(b'>')
        .add(b'?')
        .add(b'`')
        .add(b'{')
        .add(b'}');

    pub(crate) fn encode_path(pc: &str) -> String {
        utf8_percent_encode(pc, PATH_SET).to_string()
    }
}

#[derive(Debug)]
pub struct Response<T> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: T,
}

impl<T> Response<T> {
    pub fn new(status: StatusCode, headers: HeaderMap, body: T) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
}

// TODO: what about this?
// TODO: could expand implementation to includes fields such as whats in
// https://github.com/spring-projects/spring-data-commons/blob/main/src/main/java/org/springframework/data/domain/PageImpl.java#L83
pub struct PagedResponse<T> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: T,
    pub pagination: LinkHeader,
}

/// Errors returned by the client
#[derive(Debug, Error)]
pub enum ClientError {
    /// Generic HTTP Error
    #[error("HTTP Error. Code: {status}, message: {error}")]
    HttpError {
        status: StatusCode,
        headers: HeaderMap,
        error: String,
    },

    #[error(transparent)]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),

    /// Errors returned by reqwest
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    /// Errors returned by reqwest middleware
    #[error(transparent)]
    ReqwestMiddleWareError(#[from] reqwest_middleware::Error),

    /// Serde JSON parsing error
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    /// URL Parsing Error
    #[error(transparent)]
    UrlParserError(#[from] url::ParseError),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Default)]
pub(crate) struct Message {
    pub body: Option<reqwest::Body>,
    pub content_type: Option<String>,
}

/// Entrypoint for interacting with the API client.
#[derive(Clone)]
pub struct Client {
    host: String,
    host_override: Option<String>,
    agent: String,
    client: reqwest_middleware::ClientWithMiddleware,
    credentials: Option<Credentials>,
    // TODO: http_cache
}

#[derive(Clone, Copy, Default)]
pub enum MediaType {
    /// Return json (the default)
    #[default]
    Json,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaType::Json => write!(f, "application/json"),
        }
    }
}

impl From<MediaType> for mime::Mime {
    fn from(media: MediaType) -> mime::Mime {
        match media {
            MediaType::Json => "application/json".parse().unwrap(),
        }
    }
}

/// Controls what sort of authentication is required for this request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthenticationConstraint {
    /// No constraint
    Unconstrained,
    /// Must be JWT
    JWT,
}

/// Various forms of authentication credentials supported by GitHub.
#[derive(PartialEq, Clone)]
pub enum Credentials {
    JobToken(String),
    PrivateToken(String),
    // TODO: This should be access token....how to refresh?
    /// JWT token exchange, to be performed transparently in the background
    JWT(JWTCredentials),
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credentials::JobToken(value) => f
                .debug_tuple("Credentials::JobToken")
                .field(&"*".repeat(value.len()))
                .finish(),
            Credentials::PrivateToken(value) => f
                .debug_tuple("Credentials::PrivateToken")
                .field(&"*".repeat(value.len()))
                .finish(),
            Credentials::JWT(jwt) => f
                .debug_struct("Credentials::JWT")
                .field("app_id", &jwt.app_id)
                .field("private_key", &"vec![***]")
                .finish(),
        }
    }
}

/// JSON Web Token authentication mechanism.
///
/// The GitHub client methods are all &self, but the dynamically
/// generated JWT token changes regularly. The token is also a bit
/// expensive to regenerate, so we do want to have a mutable cache.
///
/// We use a token inside a Mutex so we can have interior mutability
/// even though JWTCredentials is not mutable.
#[derive(Clone)]
pub struct JWTCredentials {
    pub app_id: i64,
    /// DER RSA key. Generate with
    /// `openssl rsa -in private_rsa_key.pem -outform DER -out private_rsa_key.der`
    pub private_key: Vec<u8>,
    cache: Arc<Mutex<ExpiringJWTCredential>>,
}

impl JWTCredentials {
    pub fn new(app_id: i64, private_key: Vec<u8>) -> ClientResult<JWTCredentials> {
        let creds = ExpiringJWTCredential::calculate(app_id, &private_key)?;

        Ok(JWTCredentials {
            app_id,
            private_key,
            cache: Arc::new(Mutex::new(creds)),
        })
    }

    /// Fetch a valid JWT token, regenerating it if necessary
    pub fn token(&self) -> String {
        let mut expiring = self.cache.lock().unwrap();
        if expiring.is_stale() {
            *expiring = ExpiringJWTCredential::calculate(self.app_id, &self.private_key)
                .expect("JWT private key worked before, it should work now...");
        }

        expiring.token.clone()
    }
}

impl PartialEq for JWTCredentials {
    fn eq(&self, other: &JWTCredentials) -> bool {
        self.app_id == other.app_id && self.private_key == other.private_key
    }
}

#[derive(Debug)]
struct ExpiringJWTCredential {
    token: String,
    created_at: std::time::Instant,
}

#[derive(Serialize)]
struct JWTCredentialClaim {
    iat: u64,
    exp: u64,
    iss: i64,
}

impl ExpiringJWTCredential {
    fn calculate(app_id: i64, private_key: &[u8]) -> ClientResult<ExpiringJWTCredential> {
        // SystemTime can go backwards, Instant can't, so always use
        // Instant for ensuring regular cycling.
        let created_at = std::time::Instant::now();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let expires = now + MAX_JWT_TOKEN_LIFE;

        let payload = JWTCredentialClaim {
            // specifying this as 60 seconds in the past to avoid problems with clock drift.
            iat: now.as_secs().saturating_sub(60),
            exp: expires.as_secs(),
            iss: app_id,
        };
        let header = jwt::Header::new(jwt::Algorithm::RS256);
        let jwt = jwt::encode(
            &header,
            &payload,
            &jsonwebtoken::EncodingKey::from_rsa_der(private_key),
        )?;

        Ok(ExpiringJWTCredential {
            created_at,
            token: jwt,
        })
    }

    fn is_stale(&self) -> bool {
        self.created_at.elapsed() >= JWT_TOKEN_REFRESH_PERIOD
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessToken {
    // #[serde(
    //     default,
    //     skip_serializing_if = "String::is_empty",
    //     deserialize_with = "crate::utils::deserialize_null_string::deserialize"
    // )]
    pub token_type: String,

    // #[serde(
    //     default,
    //     skip_serializing_if = "String::is_empty",
    //     deserialize_with = "crate::utils::deserialize_null_string::deserialize"
    // )]
    pub access_token: String,

    #[serde(default)]
    pub expires_in: i64,

    // #[serde(
    //     default,
    //     skip_serializing_if = "String::is_empty",
    //     deserialize_with = "crate::utils::deserialize_null_string::deserialize"
    // )]
    pub refresh_token: String,

    #[serde(default, alias = "x_refresh_token_expires_in")]
    pub refresh_token_expires_in: i64,

    // #[serde(
    //     default,
    //     skip_serializing_if = "String::is_empty",
    //     deserialize_with = "crate::utils::deserialize_null_string::deserialize"
    // )]
    pub scope: String,
}

/// Time in seconds before the access token expiration point that a refresh should
/// be performed. This value is subtracted from the `expires_in` value returned by
/// the provider prior to storing
const REFRESH_THRESHOLD: Duration = Duration::from_secs(60);

#[derive(Serialize)]
pub struct OffsetBasedPagination {
    /// For offset-based and keyset-based paginated result sets, the number of results to include per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u64>,
    /// For offset-based paginated result sets, page of results to retrieve. Defaults to 20 and max 100
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u64>,
}

// Note(sean): I'm somewhat confused by what Gitlab uses as the key to page as they seem to have a
// a few different ways which include: id_after, page_token, cursor.
// I think for now we won't codify it as we'll be using whatever is returned from the Link header
/// Keyset-pagination allows for more efficient retrieval of pages and - in contrast to offset-based
/// pagination - runtime is independent of the size of the collection.
/// This method is controlled by the following parameters. order_by and sort are both mandatory.
#[derive(Serialize)]
pub struct KeysetBasedPagination {
    /// For keyset-based paginated result sets, the value must be `"keyset"`
    pagination: String,
    /// For offset-based and keyset-based paginated result sets, the number of results to include per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u64>,
    /// For keyset-based paginated result sets, name of the column by which to order
    order_by: String,
    /// For keyset-based paginated result sets, sort order (`"asc"`` or `"desc"`)
    sort: String,
}

#[derive(Serialize)]
pub enum Pagination {
    Offset(OffsetBasedPagination),
    Keyset(KeysetBasedPagination),
}

// TODO: this is really split between Offset-based pagination and Keyset-based pagination
/// ListOptions specifies the optional parameters to various List methods that support pagination.
pub struct ListOptions {
    /// For keyset-based paginated result sets, the value must be `"keyset"`
    pagination: String,
    /// For offset-based and keyset-based paginated result sets, the number of results to include per page.
    per_page: u64,
    /// For offset-based paginated result sets, page of results to retrieve.
    page: u64,
    /// For keyset-based paginated result sets, tree record ID at which to fetch the next page.
    page_token: String,
    /// For keyset-based paginated result sets, name of the column by which to order
    order_by: String,
    /// For keyset-based paginated result sets, sort order (`"asc"`` or `"desc"`)
    sort: String,
}

#[derive(Debug, PartialEq)]
pub struct Link {
    /// A parsed form of the URI
    pub uri: Url,

    /// The raw text string of the URI
    pub raw_uri: String,
}

type Rel = String;
pub type RelLinkMap = HashMap<Rel, Link>;
pub struct LinkHeader {
    links: HashMap<Rel, Link>,
}

impl LinkHeader {
    // link: <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="prev", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="next", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="first", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="last"

    // TODO: do we want to return an error if we fail to parse?
    fn parse(header: &str) -> ClientResult<Self> {
        let mut links = RelLinkMap::new();
        for part in header.split(',') {
            let sections: Vec<&str> = part.trim().split(';').collect();
            if sections.len() < 2 {
                continue;
            }

            let raw_url = sections[0]
                .trim()
                .trim_start_matches('<')
                .trim_end_matches('>');
            let mut rel = "";
            for param in &sections[1..] {
                let trimmed = param.trim();
                if trimmed.starts_with("rel=") {
                    rel = trimmed.trim_start_matches("rel=").trim_matches('"');
                }
            }

            if !rel.is_empty() {
                links.insert(
                    rel.to_string(),
                    // url.to_string()
                    Link {
                        raw_uri: raw_url.to_string(),
                        uri: Url::from_str(raw_url)?,
                    },
                );
            }
        }

        Ok(Self { links })
    }

    fn get(&self, rel: &Rel) -> Option<&Link> {
        self.links.get(rel)
    }
}

impl Client {

    pub fn new<A, C>(agent: A, credentials: C) -> ClientResult<Self>
    where
        A: Into<String>,
        C: Into<Option<Credentials>>,
    {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let client = reqwest_middleware::ClientBuilder::new(http).build();

        Ok(Self {
            host: DEFAULT_HOST.to_string(),
            host_override: None,
            agent: agent.into(),
            client,
            credentials: credentials.into(),
        })
    }

    pub fn custom<A, C>(
        agent: A,
        credentials: C,
        http: reqwest_middleware::ClientWithMiddleware,
    ) -> Self
    where
        A: Into<String>,
        C: Into<Option<Credentials>>,
    {
        Self {
            host: DEFAULT_HOST.to_string(),
            host_override: None,
            agent: agent.into(),
            client: http,
            credentials: credentials.into(),
        }
    }

    pub fn get_host_override(&self) -> Option<&str> {
        self.host_override.as_deref()
    }

    pub fn url(&self, path: &str, host: Option<&str>) -> String {
        // let mut url = reqwest::Url::parse(self.get_host_override()
        //     .or(host)
        //     .unwrap_or(self.host.as_str())
        // ).expect("");
        // url.set_path(path);
        format!(
            "{}{}",
            self.get_host_override()
                .or(host)
                .unwrap_or(self.host.as_str()),
            path
        )
    }

    // TODO: Support Link header / next link
    // I would prefer if Response included pagination / link
    // TODO (sean): As of 2025-05-20 it doesn't appear that Gitlab includes etag/checksum to support
    // HTTP cache / conditional gets. When/if it does, include logic to support
    async fn request<Out>(
        &self,
        method: http::Method,
        uri: &str,
        message: Message,
        media_type: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<(Option<LinkHeader>, Response<Out>)>
    where
        Out: serde::de::DeserializeOwned + 'static + Send,
    {
        let req = self
            .make_request(method.clone(), uri, message, media_type, authentication)
            .await?;

        let response = req.send().await?;

        // TODO (sean): Rate limit

        let status = response.status();
        let headers = response.headers().clone();
        let link = response
            .headers()
            .get(http::header::LINK)
            .and_then(|l| l.to_str().ok())
            .and_then(|l| LinkHeader::parse(l).ok());
        // let next_link = link.as_ref().and_then(crate::utils::next_link);

        let response_body = response.bytes().await?;

        if status.is_success() {
            debug!("Received successful response. Read payload.");

            let parsed_response = if status == StatusCode::NO_CONTENT
                || std::any::TypeId::of::<Out>() == std::any::TypeId::of::<()>()
            {
                serde_json::from_str("null")?
            } else {
                serde_json::from_slice::<Out>(&response_body)?
            };
            Ok((None, Response::new(status, headers, parsed_response)))
        } else if status.is_redirection() {
            match status {
                StatusCode::NOT_MODIFIED => {
                    unreachable!("this should not be reachable without the an HTTP cache")
                }
                _ => {
                    let body = if std::any::TypeId::of::<Out>() == std::any::TypeId::of::<()>() {
                        serde_json::from_str("null")?
                    } else {
                        serde_json::from_slice::<Out>(&response_body)?
                    };

                    Ok((None, Response::new(status, headers, body)))
                }
            }
        } else {
            let error_msg = if response_body.is_empty() {
                "empty response".to_string()
            } else {
                String::from_utf8_lossy(&response_body).to_string()
            };

            Err(ClientError::HttpError {
                status,
                headers,
                error: error_msg,
            })
        }
    }

    async fn request_entity<D>(
        &self,
        method: http::Method,
        uri: &str,
        message: Message,
        media_type: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        let (_, r) = self
            .request(method, uri, message, media_type, authentication)
            .await?;
        Ok(r)
    }

    async fn get<D>(&self, uri: &str, message: Message) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.get_media(uri, MediaType::Json, message).await
    }

    async fn get_media<D>(
        &self,
        uri: &str,
        media: MediaType,
        message: Message,
    ) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request_entity(
            http::Method::GET,
            uri,
            message,
            media,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    async fn get_all_pages<D>(&self, uri: &str, _message: Message) -> ClientResult<Response<Vec<D>>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        let mut global_items = Vec::new();
        let (new_link, mut response) = self.get_pages(uri).await?;
        let mut link = new_link;
        while !response.body.is_empty() {
            global_items.append(&mut response.body);
            // We need to get the next link.

            if let Some(link_header) = &link {
                if let Some(next_link) = link_header.get(&"next".to_string()) {
                    let url = Url::parse(&next_link.raw_uri)?;
                    let (new_link, new_response) = self.get_pages_url(&url).await?;
                    link = new_link;
                    response = new_response;
                }
            }
        }

        Ok(Response::new(
            response.status,
            response.headers,
            global_items,
        ))
    }

    async fn get_pages<D>(&self, uri: &str) -> ClientResult<(Option<LinkHeader>, Response<Vec<D>>)>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request(
            http::Method::GET,
            uri,
            Message::default(),
            MediaType::Json,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    // TODO: this seems unnecessary? Whats the difference between this and get_pages? Just the input
    // Could we just do a AsRef<&str> / Into<&str>?
    async fn get_pages_url<D>(
        &self,
        url: &reqwest::Url,
    ) -> ClientResult<(Option<LinkHeader>, Response<Vec<D>>)>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request(
            http::Method::GET,
            url.as_str(),
            Message::default(),
            MediaType::Json,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    // TODO: change to build_request as 'make_request' sounds like the action of actually executing
    // the call but its really creating the request builder
    async fn make_request(
        &self,
        method: http::Method,
        uri: &str,
        message: Message,
        media_type: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<reqwest_middleware::RequestBuilder> {
        let (url, auth) = self.url_and_auth(uri, authentication).await?;

        let mut req = self.client.request(method, url);

        if let Some(content_type) = &message.content_type {
            req = req.header(http::header::CONTENT_TYPE, content_type.clone());
        }

        req = req.header(http::header::USER_AGENT, &*self.agent);
        req = req.header(http::header::ACCEPT, &media_type.to_string());

        if let Some(auth_str) = auth {
            req = req.header(http::header::AUTHORIZATION, &*auth_str);
        }

        if let Some(body) = message.body {
            req = req.body(body);
        }

        Ok(req)
    }

    fn credentials(&self, authentication: AuthenticationConstraint) -> Option<&Credentials> {
        match (authentication, self.credentials.as_ref()) {
            (AuthenticationConstraint::Unconstrained, creds) => creds,
            (AuthenticationConstraint::JWT, creds @ Some(&Credentials::JWT(_))) => creds,
            (AuthenticationConstraint::JWT, _) => {
                info!("Request needs JWT authentication but only a mismatched method is available");
                None
            }
        }
    }

    async fn url_and_auth(
        &self,
        uri: &str,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<(reqwest::Url, Option<String>)> {
        let mut parsed_url = uri.parse::<reqwest::Url>()?;
        match self.credentials(authentication) {
            Some(Credentials::JobToken(token)) => {
                // Job Token: `job_token` parameter or the `JOB-TOKEN` header.
                parsed_url.query_pairs_mut().append_pair("JOB-TOKEN", token);
                Ok((parsed_url, None))
            }
            Some(Credentials::PrivateToken(token)) => {
                // Personal/project/group access tokens `private_token` parameter or the `PRIVATE-TOKEN` header.
                parsed_url
                    .query_pairs_mut()
                    .append_pair("PRIVATE-TOKEN", token);
                Ok((parsed_url, None))
            }
            Some(Credentials::JWT(jwt)) => {
                // OAuth 2.0 token as either `access_token=OAUTH-TOKEN` or `Authorization` header
                let auth = format!("Bearer {}", jwt.token());
                Ok((parsed_url, Some(auth)))
            }
            None => Ok((parsed_url, None)),
        }
    }

    async fn post<D>(&self, uri: &str, message: Message) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.post_media(
            uri,
            message,
            MediaType::Json,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    async fn post_media<D>(
        &self,
        uri: &str,
        message: Message,
        media: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request_entity(http::Method::POST, uri, message, media, authentication)
            .await
    }

    async fn patch_media<D>(
        &self,
        uri: &str,
        message: Message,
        media: MediaType,
    ) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request_entity(
            http::Method::PATCH,
            uri,
            message,
            media,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    async fn patch<D>(&self, uri: &str, message: Message) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.patch_media(uri, message, MediaType::Json).await
    }

    async fn put<D>(&self, uri: &str, message: Message) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.put_media(uri, message, MediaType::Json).await
    }

    async fn put_media<D>(
        &self,
        uri: &str,
        message: Message,
        media: MediaType,
    ) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request_entity(
            http::Method::PUT,
            uri,
            message,
            media,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    async fn delete<D>(&self, uri: &str, message: Message) -> ClientResult<Response<D>>
    where
        D: serde::de::DeserializeOwned + 'static + Send,
    {
        self.request_entity(
            http::Method::DELETE,
            uri,
            message,
            MediaType::Json,
            AuthenticationConstraint::Unconstrained,
        )
        .await
    }

    // TODO: JWT refresh token

    pub fn pull_requests(&self) -> pulls::PullRequests {
        pulls::PullRequests::new(self.clone())
    }
}

// https://docs.rs/ethers-providers/2.0.14/src/ethers_providers/rpc/transports/retry.rs.html#366
// Retry Policy
// Rate Limit Retry Policy

// https://github.com/TrueLayer/reqwest-middleware
// https://github.com/TrueLayer/retry-policies
// let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
// pub struct RateLimitRetry {
//     pub header: String
// }
// impl RetryableStrategy for RateLimitRetry {
//     fn handle(&self, res: &Result<reqwest::Response>) -> Option<Retryable> {
//         match res {
//             Ok(success) => None,
//             Err(error) => default_on_request_failure(error),
//         }
//     }
// }

pub struct ClientBuilder {
    host: String,
    agent: String,
    // TODO: if we want to support more convenience fns we could make this a ClientBuilder
    http: reqwest::Client,
    middleware: Vec<Arc<dyn Middleware>>,
    credentials: Option<Credentials>,
}

impl ClientBuilder {
    pub fn new() -> ClientResult<Self> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            host: DEFAULT_HOST.to_string(),
            agent: format!("{}/{}", DEFAULT_CLIENT_AGENT, env!("CARGO_PKG_VERSION")),
            http,
            middleware: Vec::new(),
            credentials: None,
        })
    }

    pub fn with_agent(mut self, agent: &str) -> Self {
        self.agent = agent.to_string();
        self
    }

    pub fn with_host_override(mut self, host_override: &str) -> Self {
        self.host = host_override.to_string();
        self
    }

    pub fn with_http(mut self, http: reqwest::Client) -> Self {
        self.http = http;
        self
    }

    pub fn add_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.add_middleware_arc(Arc::new(middleware))
    }

    fn add_middleware_arc(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Convenience method to attach tracing middleware
    pub fn with_tracing(mut self) -> Self {
        self.add_middleware(TracingMiddleware::default())
    }

    /// Convenience method to attach retry middleware
    pub fn with_retry(mut self) -> Self {
        // TODO: conditional middleware? https://github.com/oxidecomputer/reqwest-conditional-middleware/tree/main
        // TODO: rate limit retry middleware?
        let retry_policy =
            reqwest_retry::policies::ExponentialBackoff::builder().build_with_max_retries(3);
        self.add_middleware(RetryTransientMiddleware::new_with_policy(retry_policy))
    }

    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn build(self) -> ClientResult<Client> {
        let mut builder = reqwest_middleware::ClientBuilder::new(self.http);
        for middleware in self.middleware {
            builder = builder.with_arc(middleware)
        }

        Ok(Client {
            host: self.host,
            host_override: None,
            agent: self.agent,
            client: builder.build(),
            credentials: self.credentials,
        })
    }
}