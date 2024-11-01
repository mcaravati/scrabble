use crate::Error;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    data: Option<T>,
    error: Option<Error>,
}

impl<T: Serialize> Response<T> {
    pub fn from_data(data: T) -> Self {
        Response {
            data: Some(data),
            error: None,
        }
    }

    pub fn from_error(error: Error) -> Self {
        Response {
            data: None,
            error: Some(error),
        }
    }
}
