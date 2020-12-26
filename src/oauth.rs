use crate::util::globals::{TWITCH_CALLBACK_URL, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use oauth2::{
    basic::{BasicErrorResponseType, BasicTokenType},
    prelude::{NewType, SecretNewType},
};
use oauth2::{
    AccessToken, AuthType, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    ExtraTokenFields, RedirectUrl, RefreshToken, RequestTokenError, Scope, TokenResponse,
    TokenType, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

pub fn twitch_authenticate() -> (Url, CsrfToken) {
    let client = twitch_client();
    client.authorize_url(CsrfToken::new_random)
}

pub fn twitch_exchange_code(auth_code: &str) -> OAuthExchangeResult {
    let client = twitch_client().set_auth_type(AuthType::RequestBody);

    client.exchange_code(AuthorizationCode::new(auth_code.to_string()))
}

pub fn twitch_client() -> TwitchOauthClient {
    let client = TwitchOauthClient::new(
        ClientId::new(TWITCH_CLIENT_ID.to_string()),
        Some(ClientSecret::new(TWITCH_CLIENT_SECRET.to_string())),
        AuthUrl::new(Url::parse("https://id.twitch.tv/oauth2/authorize").unwrap()),
        Some(TokenUrl::new(
            Url::parse("https://id.twitch.tv/oauth2/token").unwrap(),
        )),
    )
    .add_scope(Scope::new("openid user:read:email".to_string()))
    .set_redirect_url(RedirectUrl::new(Url::parse(&TWITCH_CALLBACK_URL).unwrap()));

    client
}

pub type OAuthExchangeResult = Result<ExchangeSuccess, ExchangeError>;

pub type ExchangeSuccess = TwitchTokenResponse<TwitchFields, BasicTokenType>;

pub type ExchangeError = RequestTokenError<BasicErrorResponseType>;

pub type TwitchOauthClient = Client<
    BasicErrorResponseType,
    TwitchTokenResponse<TwitchFields, BasicTokenType>,
    BasicTokenType,
>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TwitchFields {
    id_token: String,
}

impl ExtraTokenFields for TwitchFields {}

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
