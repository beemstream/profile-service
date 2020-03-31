use oauth2::{
    RefreshToken,
    AccessToken,
    AuthorizationCode,
    AuthUrl,
    AuthType,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenUrl,
    ExtraTokenFields,
    RequestTokenError,
    TokenType,
    Client,
    TokenResponse
};
use oauth2::basic::{BasicTokenType, BasicErrorResponseType};
use oauth2::prelude::*;
use url::Url;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TwitchFields {
    id_token: String
}

impl ExtraTokenFields for TwitchFields{}

type OAuthExchangeResult = 
    Result<
        TwitchTokenResponse<TwitchFields, BasicTokenType>,
        RequestTokenError<BasicErrorResponseType>
    >;

pub fn twitch_authenticate() -> (Url, CsrfToken) {
    let client = twitch_client();
    client.authorize_url(CsrfToken::new_random)
}

pub fn twitch_exchange_code(auth_code: String) -> OAuthExchangeResult {
    let client = twitch_client()
        .set_auth_type(AuthType::RequestBody);

    client.exchange_code(AuthorizationCode::new(auth_code))
}

pub fn twitch_client() -> TwitchOauthClient {
    let client = TwitchOauthClient::new(
        ClientId::new(std::env::var("TWITCH_CLIENT_ID").expect("Set TWITCH_CLIENT_ID")),
        Some(ClientSecret::new(std::env::var("TWITCH_CLIENT_SECRET").expect("Set TWITCH_CLIENT_SECRET"))),
        AuthUrl::new(Url::parse("https://id.twitch.tv/oauth2/authorize").unwrap()),
        Some(TokenUrl::new(Url::parse("https://id.twitch.tv/oauth2/token").unwrap()))
        )
        .add_scope(Scope::new("openid user:read:email".to_string()))
        .set_redirect_url(RedirectUrl::new(Url::parse("http://localhost:4200/auth/twitch/callback").unwrap()));

    client
}

pub type TwitchOauthClient = Client<BasicErrorResponseType, TwitchTokenResponse<TwitchFields, BasicTokenType>, BasicTokenType>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

impl<EF, TT> TokenResponse<TT> for TwitchTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    ///
    /// REQUIRED. The access token issued by the authorization server.
    ///
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
    ///
    /// REQUIRED. The type of the token issued as described in
    /// [Section 7.1](https://tools.ietf.org/html/rfc6749#section-7.1).
    /// Value is case insensitive and deserialized to the generic `TokenType` parameter.
    ///
    fn token_type(&self) -> &TT {
        &self.token_type
    }
    ///
    /// RECOMMENDED. The lifetime in seconds of the access token. For example, the value 3600
    /// denotes that the access token will expire in one hour from the time the response was
    /// generated. If omitted, the authorization server SHOULD provide the expiration time via
    /// other means or document the default value.
    ///
    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.map(Duration::from_secs)
    }
    ///
    /// OPTIONAL. The refresh token, which can be used to obtain new access tokens using the same
    /// authorization grant as described in
    /// [Section 6](https://tools.ietf.org/html/rfc6749#section-6).
    ///
    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }
    ///
    /// OPTIONAL, if identical to the scope requested by the client; otherwise, REQUIRED. The
    /// scipe of the access token as described by
    /// [Section 3.3](https://tools.ietf.org/html/rfc6749#section-3.3). If included in the response,
    /// this space-delimited field is parsed into a `Vec` of individual scopes. If omitted from
    /// the response, this field is `None`.
    ///
    fn scopes(&self) -> Option<&Vec<Scope>> {
        self.scopes.as_ref()
    }
}

