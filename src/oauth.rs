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
        .set_redirect_url(RedirectUrl::new(Url::parse(&std::env::var("TWITCH_CALLBACK_URL").expect("set TWITCH_CALLBACK_URL")).unwrap()));

    client
}

pub type OAuthExchangeResult = 
    Result<
        TwitchTokenResponse<TwitchFields, BasicTokenType>,
        RequestTokenError<BasicErrorResponseType>
    >;


pub type TwitchOauthClient = Client<BasicErrorResponseType, TwitchTokenResponse<TwitchFields, BasicTokenType>, BasicTokenType>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TwitchFields {
    id_token: String
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

