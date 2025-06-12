use crate::models::TokenResponse;
use crate::settings;
use aidoku::{
	alloc::String,
	imports::{error::AidokuError, net::Request},
	prelude::*,
	Result,
};

const CLIENT_ID: &str = "neko"; // we hijack neko's login system
const AUTH_URL: &str = "https://auth.mangadex.org";
const REDIRECT_URI: &str = "neko://mangadex-auth";

fn refresh_access_token() -> Result<TokenResponse> {
	let Ok(token_response) = settings::get_token() else {
		settings::clear_token();
		return Err(AidokuError::Message("Not logged in".into()));
	};

	let Some(refresh_token) = token_response.refresh_token else {
		settings::clear_token();
		return Err(AidokuError::Message("Missing refresh token".into()));
	};

	let url = format!("{AUTH_URL}/realms/mangadex/protocol/openid-connect/token");
	let code_verifier = settings::get_code_verifier().unwrap_or_default();
	let body = format!(
		"client_id={CLIENT_ID}\
			&grant_type=refresh_token\
			&refresh_token={refresh_token}\
			&code_verifier={code_verifier}\
			&redirect_uri={REDIRECT_URI}",
	);
	let token_response = Request::post(url)?
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.data()?;

	settings::clear_token();

	let Ok(string_value) = String::from_utf8(token_response) else {
		return Err(AidokuError::Message("Failed to refresh token".into()));
	};

	let token_response = serde_json::from_str::<TokenResponse>(&string_value)
		.map_err(|_| AidokuError::JsonParseError)?;

	settings::set_token(&string_value);

	Ok(token_response)
}

pub fn auth_request<'a, T>(request: &'a mut Request) -> Result<T>
where
	T: serde::de::Deserialize<'a>,
{
	let token_response = settings::get_token()?;

	auth_request_inner(request, token_response, true)
}

fn auth_request_inner<'a, T>(
	request: &'a mut Request,
	token_response: TokenResponse,
	allow_retry: bool,
) -> Result<T>
where
	T: serde::de::Deserialize<'a>,
{
	let Some(access_token) = token_response.access_token else {
		settings::clear_token();
		return Err(AidokuError::Message("Missing access token".into()));
	};

	request.set_header("Authorization", &format!("Bearer {}", access_token));

	let data = request.get_data()?;
	request.data = Some(data);

	let status = request.status_code();

	if status == 401 && allow_retry {
		let token_response = refresh_access_token()?;
		request.reset();
		return auth_request_inner(request, token_response, false);
	}

	let value = serde_json::from_slice(request.data.as_ref().unwrap())
		.map_err(|_| AidokuError::JsonParseError)?;
	Ok(value)
}
