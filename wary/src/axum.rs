use core::ops::{Deref, DerefMut};

use axum::{extract::{FromRequest, FromRequestParts, Request}, http::{request::Parts, StatusCode}, response::{IntoResponse, Response}};

use crate::Report;

pub struct Wary<T>(pub T);

impl<T> Wary<T> {
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Deref for Wary<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Wary<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub enum WaryRejection<T, E> {
	Wary(T),
	Inner(E),
}

impl<T, E> IntoResponse for WaryRejection<T, E>
where
	T: serde::Serialize,
	E: IntoResponse,
{
		fn into_response(self) -> Response {
				match self {
						WaryRejection::Wary(report) => {
							(StatusCode::UNPROCESSABLE_ENTITY, axum::Json(report)).into_response()
						}
						WaryRejection::Inner(e) => e.into_response(),
				}
		}
}

impl<S, T, I> FromRequest<S> for Wary<T>
where
	T: FromRequest<S>,
	T: DerefMut<Target = I>,
	I: crate::Wary<S>,
	S: Sync
{
		type Rejection = WaryRejection<Report, T::Rejection>;

		async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
			let mut inner = T::from_request(req, state).await.map_err(WaryRejection::Inner)?;
			inner.wary(state).map_err(WaryRejection::Wary)?;

			Ok(Wary(inner))
		}
}

impl<S, T, I> FromRequestParts<S> for Wary<T>
where
	T: FromRequestParts<S>,
	T: DerefMut<Target = I>,
	I: crate::Wary<S>,
	S: Sync
{
		type Rejection = WaryRejection<Report, T::Rejection>;

		async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
			let mut inner = T::from_request_parts(parts, state).await.map_err(WaryRejection::Inner)?;
			inner.wary(state).map_err(WaryRejection::Wary)?;

			Ok(Wary(inner))
		}
}
