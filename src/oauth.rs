use oauth2::reqwest::http_client;
use oauth2::{
    basic::{BasicErrorResponse, BasicTokenType},
    reqwest::HttpClientError,
};
use oauth2::{
    AccessToken, AuthType, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    ExtraTokenFields, RedirectUrl, RefreshToken, RequestTokenError, Scope, TokenResponse,
    TokenType, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub fn twitch_authenticate(
    client_id: String,
    client_secret: String,
    callback_url: String,
) -> (oauth2::url::Url, CsrfToken) {
    let client = twitch_client(client_id, client_secret, callback_url);
    client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid user:read:email".to_string()))
        .url()
}

pub fn twitch_exchange_code(
    auth_code: &str,
    client_id: String,
    client_secret: String,
    callback_url: String,
) -> Result<ExchangeSuccess, ExchangeError> {
    let client =
        twitch_client(client_id, client_secret, callback_url).set_auth_type(AuthType::RequestBody);

    let c: Result<ExchangeSuccess, RequestTokenError<HttpClientError, BasicErrorResponse>> = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .request(http_client);

    c
}

pub fn twitch_client(
    client_id: String,
    client_secret: String,
    callback_url: String,
) -> TwitchOauthClient {
    let client = TwitchOauthClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://id.twitch.tv/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://id.twitch.tv/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_url(RedirectUrl::new(callback_url).unwrap());

    client
}

pub type ExchangeSuccess = TwitchTokenResponse<TwitchFields, BasicTokenType>;

pub type ExchangeError = RequestTokenError<HttpClientError, BasicErrorResponse>;

pub type TwitchOauthClient =
    Client<BasicErrorResponse, TwitchTokenResponse<TwitchFields, BasicTokenType>, BasicTokenType>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TwitchFields {
    id_token: String,
}

impl ExtraTokenFields for TwitchFields {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TwitchTokenResponse<EF: ExtraTokenFields, TT: TokenType> {
    access_token: AccessToken,
    #[serde(bound = "TT: TokenType")]
    #[serde(deserialize_with = "oauth2::helpers::deserialize_untagged_enum_case_insensitive")]
    token_type: TT,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    #[serde(rename = "scope")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    scopes: Option<Vec<Scope>>,

    #[serde(bound = "EF: ExtraTokenFields")]
    #[serde(flatten)]
    extra_fields: EF,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct StandardTokenResponse<EF, TT>
// where
//     EF: ExtraTokenFields,
//     TT: TokenType,
// {
//     access_token: AccessToken,
//     #[serde(bound = "TT: TokenType")]
//     #[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
//     token_type: TT,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     expires_in: Option<u64>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     refresh_token: Option<RefreshToken>,
//     #[serde(rename = "scope")]
//     #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
//     #[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[serde(default)]
//     scopes: Option<Vec<Scope>>,

//     #[serde(bound = "EF: ExtraTokenFields")]
//     #[serde(flatten)]
//     extra_fields: EF,
// }
impl<EF, TT> TokenResponse<TT> for TwitchTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    fn token_type(&self) -> &TT {
        &self.token_type
    }

    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.map(Duration::from_secs)
    }

    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    fn scopes(&self) -> Option<&Vec<Scope>> {
        self.scopes.as_ref()
    }
}
