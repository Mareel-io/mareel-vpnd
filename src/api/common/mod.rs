use std::sync::{Arc, Mutex};

use prometheus::Registry;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiResponse<T> {
    pub(crate) status: String,
    pub(crate) data: Option<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiError {
    pub(crate) status: String,
    pub(crate) code: i64,
    pub(crate) message: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Result<Json<Self>, Json<ApiError>> {
        Ok(Json(Self {
            status: "ok".to_string(),
            data: Some(data),
        }))
    }

    pub fn err(code: i64, message: &str) -> Result<Json<ApiResponse<T>>, Json<ApiError>> {
        Err(Json(ApiError {
            status: "error".to_string(),
            message: message.to_string(),
            code,
        }))
    }
}

pub(crate) struct PrometheusStore {
    pub(crate) registry: Arc<Mutex<Registry>>,
}

pub(crate) type ApiResponseType<T> = (Status, Result<Json<ApiResponse<T>>, Json<ApiError>>);
