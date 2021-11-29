use rocket::serde::{Deserialize, Serialize, json::Json};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiResponse<T> {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) status: Option<String>,
    pub(crate) data: Option<T>,
}

#[derive(Debug)]
pub(crate) struct ApiError;