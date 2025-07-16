use wasm_bindgen::JsValue;

pub(crate) use anyhow::Context;

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl Error {
    pub(crate) fn new(msg: &'static str) -> Self {
        Error(anyhow::anyhow!(msg))
    }

    pub(crate) fn from_js_value(msg: JsValue) -> Self {
        let msg = format!("{msg:?}");
        Error(anyhow::anyhow!(msg))
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Error(err.into())
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        let msg = err.0.to_string();
        JsValue::from(msg)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
