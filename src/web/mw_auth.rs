use crate::{
	ctx::Ctx,
	model::{
		user::{UserBmc, UserForAuth},
		ModelManager,
	},
	token::{validate_web_token, Token},
	web::{set_token_cookie, Error, Result, AUTH_TOKEN},
};
use async_trait::async_trait;
use axum::{
	extract::{FromRequestParts, State},
	http::{request::Parts, Request},
	middleware::Next,
	response::Response,
};
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

pub async fn mw_ctx_require<B>(
	ctx: Result<Ctx>,
	req: Request<B>,
	next: Next<B>,
) -> Result<Response> {
	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");
	ctx?;
	Ok(next.run(req).await)
}

pub async fn mw_ctx_resolve<B>(
	mm: State<ModelManager>,
	cookies: Cookies,
	mut req: Request<B>,
	next: Next<B>,
) -> Result<Response> {
	debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");
	let ctx_ext_result = _ctx_resolve(mm, &cookies).await;
	if ctx_ext_result.is_err()
		&& !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie))
	{
		cookies.remove(Cookie::named(AUTH_TOKEN))
	}
	req.extensions_mut().insert(ctx_ext_result);
	Ok(next.run(req).await)
}

async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
	let token = cookies
		.get(AUTH_TOKEN)
		.map(|c| c.value().to_string())
		.ok_or(CtxExtError::TokenNotInCookie)?;

	let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;
	let user: UserForAuth =
		UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?
			.into();

	validate_web_token(&token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	set_token_cookie(cookies, &user.username, user.token_salt)
		.map_err(|_| CtxExtError::CannotSetTokenCookie)?;

	Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");
		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	TokenNotInCookie,
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,
	CannotSetTokenCookie,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
