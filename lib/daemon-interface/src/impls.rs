use std::fmt::{write, Display, Formatter};
use serde::Serialize;
use crate::response::{ErrorPayload, Response, ResponsePayload};

impl<T: Display + Serialize, E: Display + Serialize> Display for Response<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Todo: Implement properly
        write!(f, "Todo")
    }
}

impl Display for ResponsePayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Todo: Implement properly
        write!(f, "{:?}", self)
    }
}

impl Display for ErrorPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Todo: Implement properly
        write!(f, "{:?}", self)
    }
}
