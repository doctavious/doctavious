pub mod merge_requests;

// I liked the look of the design from https://github.com/oxidecomputer/third-party-api-clients/tree/main
// Good portion of code lifted from that
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jsonwebtoken as jwt;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

// TODO: this will be different
const TOKEN_ENDPOINT: &str = "https://gitlab.com/oauth/token";

// We use 9 minutes for the life to give some buffer for clock drift between
// our clock and GitHub's. The absolute max is 10 minutes.
const MAX_JWT_TOKEN_LIFE: Duration = Duration::from_secs(60 * 9);
// 8 minutes so we refresh sooner than it actually expires
const JWT_TOKEN_REFRESH_PERIOD: Duration = Duration::from_secs(60 * 8);

mod support {
    use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

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

    /// Serde JSON parsing error
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    /// URL Parsing Error
    #[error(transparent)]
    UrlParserError(#[from] url::ParseError),
}

type ClientResult<T> = Result<T, ClientError>;

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
    // #[cfg(feature = "middleware")]
    // client: reqwest_middleware::ClientWithMiddleware,
    // #[cfg(not(feature = "middleware"))]
    client: reqwest::Client,
    credentials: Option<Credentials>,
    // #[cfg(feature = "httpcache")]
    // http_cache: crate::http_cache::BoxedHttpCache,

    // auto_refresh: bool,
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

impl Client {
    pub fn new<A, C>(agent: A, credentials: C) -> ClientResult<Self>
    where
        A: Into<String>,
        C: Into<Option<Credentials>>,
    {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            // TODO: default server host
            host: "".to_string(),
            host_override: None,
            agent: agent.into(),
            client: http,
            credentials: credentials.into(),
        })
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

    // TODO: may want to support link headers / NextLink
    async fn request<Out>(
        &self,
        method: http::Method,
        uri: &str,
        message: Message,
        media_type: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<Response<Out>>
    where
        Out: serde::de::DeserializeOwned + 'static + Send,
    {
        let req = self
            .make_request(method.clone(), uri, message, media_type, authentication)
            .await?;

        let response = req.send().await?;

        // TODO: any rate limiting headers?
        // let (remaining, reset) = crate::utils::get_header_values(response.headers());

        let status = response.status();
        let headers = response.headers().clone();
        // let link = response
        //     .headers()
        //     .get(http::header::LINK)
        //     .and_then(|l| l.to_str().ok());
        //     .and_then(|l| parse_link_header::parse(l).ok());
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
            Ok(Response::new(status, headers, parsed_response))
        } else if status.is_redirection() {
            match status {
                // StatusCode::NOT_MODIFIED => {
                //     #[cfg(not(feature = "httpcache"))]
                //     {
                //         unreachable!(
                //             "this should not be reachable without the httpcache feature enabled"
                //         )
                //     }
                // }
                _ => {
                    // The body still needs to be parsed. Except in the case of 304 (handled above),
                    // returning a body in the response is allowed.
                    let body = if std::any::TypeId::of::<Out>() == std::any::TypeId::of::<()>() {
                        serde_json::from_str("null")?
                    } else {
                        serde_json::from_slice::<Out>(&response_body)?
                    };

                    Ok(Response::new(status, headers, body))
                }
            }
        } else {
            // let error = match (remaining, reset) {
            //     (Some(remaining), Some(reset)) if remaining == 0 => {
            //         let now = std::time::SystemTime::now()
            //             .duration_since(std::time::UNIX_EPOCH)
            //             .unwrap()
            //             .as_secs();
            //         ClientError::RateLimited {
            //             duration: u64::from(reset).saturating_sub(now),
            //         }
            //     }
            //     _ => {
            //         if response_body.is_empty() {
            //             ClientError::HttpError {
            //                 status,
            //                 headers,
            //                 error: "empty response".into(),
            //             }
            //         } else {
            //             ClientError::HttpError {
            //                 status,
            //                 headers,
            //                 error: String::from_utf8_lossy(&response_body).into(),
            //             }
            //         }
            //     }
            // };
            let error = if response_body.is_empty() {
                ClientError::HttpError {
                    status,
                    headers,
                    error: "empty response".into(),
                }
            } else {
                ClientError::HttpError {
                    status,
                    headers,
                    error: String::from_utf8_lossy(&response_body).into(),
                }
            };
            Err(error)
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
        let r = self
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

    // TODO: change to build_request as 'make_request' sounds like the action of actually executing
    // the call but its really creating the request builder
    async fn make_request(
        &self,
        method: http::Method,
        uri: &str,
        message: Message,
        media_type: MediaType,
        authentication: AuthenticationConstraint,
    ) -> ClientResult<reqwest::RequestBuilder> {
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

    // parameters = 'client_id=APP_ID&code=RETURNED_CODE&grant_type=authorization_code&redirect_uri=REDIRECT_URI&code_verifier=CODE_VERIFIER'
    // RestClient.post 'https://gitlab.example.com/oauth/token', parameters
    // {
    // "access_token": "de6780bc506a0446309bd9362820ba8aed28aa506c71eedbe1c5c4f9dd350e54",
    // "token_type": "bearer",
    // "expires_in": 7200,
    // "refresh_token": "8257e65c97202ed1726cf9571600918f3bffb2544b26e00a61df9897668c33a1",
    // "created_at": 1607635748
    // }
    // To retrieve a new access_token, use the refresh_token parameter.
    // Refresh tokens may be used even after the access_token itself expires. This request:
    // parameters = 'client_id=APP_ID&refresh_token=REFRESH_TOKEN&grant_type=refresh_token&redirect_uri=REDIRECT_URI&code_verifier=CODE_VERIFIER'
    // RestClient.post 'https://gitlab.example.com/oauth/token', parameters
    // {
    // "access_token": "c97d1fe52119f38c7f67f0a14db68d60caa35ddc86fd12401718b649dcfa9c68",
    // "token_type": "bearer",
    // "expires_in": 7200,
    // "refresh_token": "803c1fd487fec35562c205dac93e9d8e08f9d3652a24079d704df3039df1158f",
    // "created_at": 1628711391
    // }

    // /// Refresh an access token from a refresh token. Client must have a refresh token for this to work.
    // pub async fn refresh_access_token(&self) -> ClientResult<AccessToken> {
    //     let response = {
    //         let refresh_token = &self.token.read().await.refresh_token;
    //
    //         if refresh_token.is_empty() {
    //             return Err(ClientError::EmptyRefreshToken);
    //         }
    //
    //         let mut headers = reqwest::header::HeaderMap::new();
    //         headers.append(
    //             reqwest::header::ACCEPT,
    //             reqwest::header::HeaderValue::from_static("application/json"),
    //         );
    //
    //         let params = [
    //             ("grant_type", "refresh_token"),
    //             ("refresh_token", refresh_token),
    //             ("client_id", &self.client_id),
    //             ("client_secret", &self.client_secret),
    //             ("redirect_uri", &self.redirect_uri),
    //         ];
    //         let client = reqwest::Client::new();
    //         client
    //             .post(TOKEN_ENDPOINT)
    //             .headers(headers)
    //             .form(&params)
    //             .basic_auth(&self.client_id, Some(&self.client_secret))
    //             .send()
    //             .await?
    //     };
    //
    //     // Unwrap the response.
    //     let t: AccessToken = response.json().await?;
    //
    //     let refresh_token = self.token.read().await.refresh_token.clone();
    //
    //     *self.token.write().await = InnerToken {
    //         access_token: t.access_token.clone(),
    //         refresh_token,
    //         expires_at: Self::compute_expires_at(t.expires_in),
    //     };
    //
    //     Ok(t)
    // }

    pub fn merge_requests(&self) -> merge_requests::MergeRequests {
        merge_requests::MergeRequests::new(self.clone())
    }
}
