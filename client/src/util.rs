use js_sys::{ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AuthenticationExtensionsClientInputs, CredentialCreationOptions, CredentialRequestOptions,
    Document, Element, PublicKeyCredential,
};

use crate::error::{Context, Error, Result};

pub trait DocumentExt<'a> {
    fn id(&self, id: &'a str) -> Result<Element>;
}

impl<'a> DocumentExt<'a> for Document {
    fn id(&self, id: &'a str) -> Result<Element> {
        let e = self
            .get_element_by_id(id)
            .context(format!("no such id {id}"))?;

        Ok(e)
    }
}

pub trait CredentialOptionsExt {
    fn set_prf_first(&self, first: &[u8]) -> Result<()>;
}

fn set_prf_first(input: &AuthenticationExtensionsClientInputs, first: &[u8]) -> Result<()> {
    let view = Uint8Array::from(first);

    let eval_obj = Object::new();
    eval_obj.set("first", &view)?;

    let prf_obj = Object::new();
    prf_obj.set("eval", &eval_obj)?;

    input.set("prf", &prf_obj)?;

    Ok(())
}

impl CredentialOptionsExt for CredentialCreationOptions {
    fn set_prf_first(&self, first: &[u8]) -> Result<()> {
        let public_key = self.get_public_key().unwrap();

        let input = public_key.get_extensions().unwrap_or_default();
        set_prf_first(&input, first)?;

        public_key.set_extensions(&input);

        Ok(())
    }
}

impl CredentialOptionsExt for CredentialRequestOptions {
    fn set_prf_first(&self, first: &[u8]) -> Result<()> {
        let public_key = self.get_public_key().unwrap();

        let input = public_key.get_extensions().unwrap_or_default();
        set_prf_first(&input, first)?;

        public_key.set_extensions(&input);

        Ok(())
    }
}

pub trait PublicKeyCredentialExt {
    fn get_prf_first(&self) -> Result<Vec<u8>>;
}

impl PublicKeyCredentialExt for PublicKeyCredential {
    fn get_prf_first(&self) -> Result<Vec<u8>> {
        let prf = self
            .get_client_extension_results()
            .get("prf")?
            .get("results")?
            .get("first")?;

        let prf = prf.cast::<ArrayBuffer>()?;
        let prf = Uint8Array::new(prf).to_vec();

        Ok(prf)
    }
}

pub trait ObjectExt<'a> {
    fn set(&self, key: &'a str, value: &JsValue) -> Result<()>;
}

impl<'a> ObjectExt<'a> for Object {
    fn set(&self, key: &'a str, value: &JsValue) -> Result<()> {
        Reflect::set(self, &JsValue::from_str(key), value).map_err(Error::from_js_value)?;
        Ok(())
    }
}

pub trait ValueExt<'a> {
    fn call0(&self, function: JsValue) -> Result<JsValue>;
    fn cast<T: JsCast>(&self) -> Result<&T>;
    fn get(&self, id: &'a str) -> Result<JsValue>;
    async fn resolve(self) -> Result<JsValue>;
}

impl<'a> ValueExt<'a> for JsValue {
    fn call0(&self, function: JsValue) -> Result<JsValue> {
        let function: &Function = function.cast()?;
        let res = function.call0(self).map_err(Error::from_js_value)?;

        Ok(res)
    }

    fn cast<T: JsCast>(&self) -> Result<&T> {
        let cast = self.dyn_ref().ok_or(Error::new("failed to cast"))?;
        Ok(cast)
    }

    fn get(&self, id: &'a str) -> Result<JsValue> {
        let v = Reflect::get(self, &JsValue::from_str(id)).map_err(Error::from_js_value)?;

        if v.is_undefined() {
            Err(Error::new("value is undefined"))
        } else if v.is_null() {
            Err(Error::new("value is null"))
        } else {
            Ok(v)
        }
    }

    async fn resolve(self) -> Result<JsValue> {
        let promise = self.dyn_into::<Promise>().map_err(Error::from_js_value)?;
        let result = JsFuture::from(promise)
            .await
            .map_err(Error::from_js_value)?;

        Ok(result)
    }
}
