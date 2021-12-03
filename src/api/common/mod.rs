use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiResponse<T> {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) status: Option<String>,
    pub(crate) data: Option<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiError {
    pub(crate) code: i64,
    pub(crate) msg: String,
}
