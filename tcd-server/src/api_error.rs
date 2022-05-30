
use std::fmt;

use serde::{Serialize, Deserialize};

use http_api::error::{ApiError, Error as ErrorTrait, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	Internal(String),
	Request(String)
}

impl ApiError for Error {

	fn internal<E: ErrorTrait>(e: E) -> Self {
		Self::Internal(e.to_string())
	}

	fn request<E: ErrorTrait>(e: E) -> Self {
		Self::Request(e.to_string())
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::Internal(_) => StatusCode::InternalServerError,
			Self::Request(_) => StatusCode::BadRequest
		}
	}

}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}