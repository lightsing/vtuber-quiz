use actix_http::Response;
use actix_web::body::Body;
use actix_web::error;
use actix_web::http::{StatusCode, header};
use serde::Serialize;

use Error::*;
use std::borrow::Borrow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Hcaptcha(#[from] crate::hcaptcha::HcaptchaError),
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorDisplay {
    pub code: u64,
    pub err: String,
}

impl Error {
    fn code(&self) -> u64 {
        match self {
            Sqlx(_) => 510000u64,
            Hcaptcha(_) => 410000u64
        }
    }

    fn user_msg(&self) -> String {
        match self {
            Sqlx(_) => "database error".to_string(),
            Hcaptcha(e) => format!("{}", e)
        }
    }

    fn as_display(&self) -> ErrorDisplay {
        ErrorDisplay {
            code: self.code(),
            err: self.user_msg()
        }
    }
}

impl error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Hcaptcha(_) => StatusCode::FORBIDDEN
        }
    }

    fn error_response(&self) -> Response<Body> {
        let mut resp = Response::new(self.status_code());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        let body = serde_json::to_string(&self.as_display())
            .unwrap_or_else(|e| {
                error!("error occurred when generating error response: {}", e);
                r#"{"code":500000, "err":"internal server error"}"#.to_string()
            });
        resp.set_body(Body::from(body))
    }
}