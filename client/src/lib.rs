use std::rc::Rc;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::window;

use crate::app::Application;
use crate::error::{Error, Result};

mod app;
mod client;
mod error;
mod keygen;
mod util;

#[wasm_bindgen]
pub async fn init() -> Result<JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    let webauthn = Application::new(window())?;

    if !webauthn.has_prf_support().await.unwrap() {
        return Err(Error::new("prf support is required"));
    }

    #[wasm_bindgen]
    struct ApplicationRc(Rc<Application>);

    Ok(ApplicationRc(webauthn).into())
}
