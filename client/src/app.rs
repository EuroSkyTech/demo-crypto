use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use api::{
    FinishAuthenticationRequest, FinishRegistrationRequest, StartAuthenticationRequest,
    StartRegistrationRequest,
};
use gloo_events::EventListener;
use tracing::{error, info, instrument};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CredentialCreationOptions, CredentialRequestOptions, Document, HtmlInputElement,
    PublicKeyCredential, Window,
};

use crate::client::Client;
use crate::error::{Context, Error, Result};
use crate::util::{CredentialOptionsExt, DocumentExt, PublicKeyCredentialExt, ValueExt};

#[wasm_bindgen]
pub(crate) struct Application {
    client: Rc<Client>,
    document: Document,
    listeners: OnceCell<Vec<EventListener>>,
    window: Window,
}

impl Application {
    pub(crate) fn new(window: Option<Window>) -> Result<Rc<Self>> {
        let window = window.context("window not available")?;
        let document = window.document().context("document not available")?;

        let register = document.id("b2c3d4e5-f6g7-8901-bcde-f12345678901")?;
        let login = document.id("5c492801-6acb-4657-a000-4ce99d5540a3")?;

        let endpoint = "localhost:9999";

        let webauthn = Rc::new(Self {
            client: Rc::new(Client::new(endpoint)),
            document,
            listeners: OnceCell::new(),
            window,
        });

        let register = EventListener::new(&register, "click", {
            let webauthn = webauthn.clone();
            move |_event| {
                wasm_bindgen_futures::spawn_local({
                    let webauthn = webauthn.clone();

                    async move {
                        if let Err(err) = webauthn.register_user().await {
                            error!(err = ?err, "failure while registering user");
                            webauthn
                                .update_status("Registration failed", "error")
                                .unwrap_throw();
                        }
                    }
                });
            }
        });

        let login = EventListener::new(&login, "click", {
            let webauthn = webauthn.clone();
            move |_event| {
                wasm_bindgen_futures::spawn_local({
                    let webauthn = webauthn.clone();
                    async move {
                        if let Err(err) = webauthn.login_user().await {
                            error!(err = ?err, "failure while logging in user");
                            webauthn
                                .update_status("Login failed", "error")
                                .unwrap_throw();
                        }
                    }
                });
            }
        });

        webauthn
            .listeners
            .set(vec![register, login])
            .map_err(|_| Error::new("failed to setup listeners"))?;

        Ok(webauthn)
    }

    async fn capabilities(&self) -> Result<HashMap<String, bool>> {
        let class = self
            .window
            .get("PublicKeyCredential")
            .context("PublicKeyCredential not available")?;

        let func = class.get("getClientCapabilities")?;
        let res = class.call0(func)?.resolve().await?;

        let capabilities = serde_wasm_bindgen::from_value(res)
            .map_err(|_| Error::new("failed to convert client capabilities"))?;

        info!(
            capabilities = ?capabilities,
            "retrieve client webauthn capabilities"
        );

        Ok(capabilities)
    }

    pub async fn has_prf_support(&self) -> Result<bool> {
        let caps = self.capabilities().await?;
        Ok(caps.contains_key("extension:prf"))
    }

    #[instrument(skip(self))]
    async fn login_user(self: &Rc<Self>) -> Result<()> {
        let username = self
            .document
            .id("dceaf2f7-75b8-4e61-88d0-99d32797af8b")?
            .cast::<HtmlInputElement>()?
            .value();

        info!(username = %username, "Login started");

        let res = self
            .client
            .auth_start(StartAuthenticationRequest { username })
            .await?;

        let challenge = res.challenge;

        info!(challenge = ?challenge, "Got login challenge");

        let options: CredentialRequestOptions = challenge.into();
        options.set_prf_first(b"hello world".as_ref())?;

        info!(options = ?options, "Got login options");

        let promise = self
            .window
            .navigator()
            .credentials()
            .get_with_options(&options)
            .map_err(Error::from_js_value)?;

        let credential = JsFuture::from(promise)
            .await
            .map_err(Error::from_js_value)?;

        let credential = PublicKeyCredential::from(credential);
        let prf = credential.get_prf_first()?;

        info!(credential = ?credential, prf = ?prf, "Created credential, finishing login");

        self.client
            .auth_finish(FinishAuthenticationRequest {
                credential: credential.into(),
            })
            .await?;

        self.update_status("Login successful", "success")?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn register_user(self: &Rc<Self>) -> Result<()> {
        let username = self
            .document
            .id("dceaf2f7-75b8-4e61-88d0-99d32797af8b")?
            .cast::<HtmlInputElement>()?
            .value();

        info!(username = %username, "Starting registration");

        let res = self
            .client
            .register_start(StartRegistrationRequest { username })
            .await?;

        let challenge = res.challenge;

        info!(challenge = ?challenge, "Got registration challenge");

        let options: CredentialCreationOptions = challenge.into();
        options.set_prf_first(b"hello world".as_ref())?;

        info!(options = ?options, "Got registration options");

        let promise = self
            .window
            .navigator()
            .credentials()
            .create_with_options(&options)
            .map_err(Error::from_js_value)?;

        let credential = JsFuture::from(promise)
            .await
            .map_err(Error::from_js_value)?;

        let credential = PublicKeyCredential::from(credential);
        let prf = credential.get_prf_first()?;

        info!(credential = ?credential, prf = ?prf, "Created credential, finishing registration");

        self.client
            .register_finish(FinishRegistrationRequest {
                credential: credential.into(),
            })
            .await?;

        self.update_status("Registration successful", "success")?;

        Ok(())
    }

    fn update_status(self: &Rc<Self>, msg: &str, status_type: &str) -> Result<()> {
        let div = self.document.id("d4e5f6g7-h8i9-0123-def0-234567890123")?;

        div.set_class_name(&format!("status {status_type}"));
        div.set_inner_html(msg);

        Ok(())
    }
}
