use std::collections::{HashMap, hash_map::Entry};
use std::sync::{Arc, Mutex};

use api::*;
use axum::{Json, Router, extract::State, routing::post};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{Key, time::Duration},
};
use tracing::{info, instrument};
use url::Url;
use uuid::Uuid;
use webauthn_rs::{Webauthn, WebauthnBuilder, prelude::*};

use crate::app::session::{AppSessionState, Session};
use crate::error::{Context, Error, Result};

pub(crate) struct App {
    id: String,
    origin: Url,
}

struct AppState {
    database: Mutex<HashMap<Username, User>>,
    webauthn: Webauthn,
}

type Username = String;

struct User {
    keypkg: Option<[u8; 64]>,
    id: Uuid,
    passkey: Option<Passkey>,
}

mod session {

    const SESSION_KEY: &str = "app-session";

    use std::fmt::Debug;

    use axum::{
        extract::FromRequestParts,
        http::{StatusCode, request::Parts},
    };
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use webauthn_rs::prelude::*;

    use crate::error::{Context, Result};

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub(crate) struct AppSession {
        pub(crate) state: AppSessionState,
        pub(crate) user_id: Option<Uuid>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) enum AppSessionState {
        Anonymous,
        Authenticated,
        Authenticating(PasskeyAuthentication),
        Registering(PasskeyRegistration),
    }

    impl Default for AppSessionState {
        fn default() -> Self {
            Self::Anonymous
        }
    }

    pub(crate) struct Session {
        data: AppSession,
        session: tower_sessions::Session,
    }

    impl Session {
        pub(crate) async fn read(self) -> Result<AppSession> {
            let session = self
                .session
                .get(SESSION_KEY)
                .await
                .context("failed to read session")?
                .unwrap_or_default();

            Ok(session)
        }

        pub(crate) async fn write<F: FnOnce(&mut AppSession)>(self, f: F) -> Result<AppSession> {
            let mut data = self.data;
            f(&mut data);

            self.session
                .insert(SESSION_KEY, &data)
                .await
                .context("failed to update session")?;

            Ok(data)
        }
    }

    impl Debug for Session {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.data.fmt(f)
        }
    }

    impl<S> FromRequestParts<S> for Session
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        async fn from_request_parts(
            req: &mut Parts,
            state: &S,
        ) -> std::result::Result<Self, Self::Rejection> {
            let session = tower_sessions::Session::from_request_parts(req, state).await?;
            let data = session
                .get(SESSION_KEY)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to get session data",
                    )
                })?
                .unwrap_or_default();

            Ok(Self { data, session })
        }
    }
}

impl App {
    pub fn new(id: &str, endpoint: &str) -> Result<Self> {
        let url = format!("https://{endpoint}");
        let origin = Url::parse(&url).context("failed to construct rp origin url")?;

        Ok(Self {
            id: id.to_owned(),
            origin,
        })
    }

    pub fn into_router(self) -> Result<Router> {
        let database = Mutex::new(HashMap::new());

        let webauthn = WebauthnBuilder::new(&self.id, &self.origin)
            .context("invalid webauthn configuration")?
            .rp_name("WebAuthn Demo")
            .build()
            .context("failed to build webauthn")?;

        let state = Arc::new(AppState { database, webauthn });

        let session_key = Key::generate();
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(300)))
            .with_signed(session_key);

        let router = Router::new()
            .route("/auth/start", post(Self::start_authentication))
            .route("/auth/finish", post(Self::finish_authentication))
            .route("/register/start", post(Self::start_registration))
            .route("/register/finish", post(Self::finish_registration))
            .fallback_service(ServeDir::new("static"))
            .layer(TraceLayer::new_for_http())
            .layer(session_layer)
            .with_state(state);

        Ok(router)
    }

    #[instrument(skip(state))]
    async fn start_authentication(
        State(state): State<Arc<AppState>>,
        session: Session,
        Json(req): Json<StartAuthenticationRequest>,
    ) -> Result<Json<StartAuthenticationResponse>> {
        let (challenge, authentication, user_id) = {
            let database = state.database.lock().map_err(Error::from_poison)?;

            let user = database.get(&req.username).context("no such user")?;
            let passkey = user.passkey.as_ref().context("user has no passkey")?;

            let (challenge, authentication) = state
                .webauthn
                .start_passkey_authentication(&[passkey.to_owned()])
                .context("failed to start passkey authentication")?;

            let user_id = user.id;

            (challenge, authentication, user_id)
        };

        session
            .write(move |data| {
                data.user_id = Some(user_id);
                data.state = AppSessionState::Authenticating(authentication);
            })
            .await?;

        Ok(Json(StartAuthenticationResponse { challenge }))
    }

    #[instrument(skip(state))]
    async fn finish_authentication(
        State(state): State<Arc<AppState>>,
        session: Session,
        Json(req): Json<FinishAuthenticationRequest>,
    ) -> Result<Json<FinishAuthenticationResponse>> {
        let session = session.read().await?;

        if let AppSessionState::Authenticating(authentication) = session.state {
            let _auth = state
                .webauthn
                .finish_passkey_authentication(&req.credential, &authentication)
                .context("failed to finish passkey authentication")?;

            let user_id = session.user_id.context("no user id in session")?;

            Ok(Json(FinishAuthenticationResponse { user_id }))
        } else {
            Err(Error::new("invalid session state"))
        }
    }

    #[instrument(skip(state))]
    async fn start_registration(
        State(state): State<Arc<AppState>>,
        session: Session,
        Json(req): Json<StartRegistrationRequest>,
    ) -> Result<Json<StartRegistrationResponse>> {
        let user_id = Uuid::new_v4();

        info!(user_id = ?user_id, "registering user");

        let (challenge, registration) = state
            .webauthn
            .start_passkey_registration(user_id, &req.username, &req.username, None)
            .context("failed to start passkey registration")?;

        session
            .write(move |data| {
                data.user_id = Some(user_id);
                data.state = AppSessionState::Registering(registration);
            })
            .await?;

        let mut database = state.database.lock().map_err(Error::from_poison)?;

        let name = req.username.clone();

        if let Entry::Vacant(entry) = database.entry(name.clone()) {
            entry.insert(User {
                id: user_id,
                keypkg: None,
                passkey: None,
            });

            Ok(Json(StartRegistrationResponse { challenge, user_id }))
        } else {
            Err(Error::new("user already exists"))
        }
    }

    #[instrument(skip(state))]
    async fn finish_registration(
        State(state): State<Arc<AppState>>,
        session: Session,
        Json(req): Json<FinishRegistrationRequest>,
    ) -> Result<Json<FinishRegistrationResponse>> {
        let session = session.read().await?;
        let user_id = session.user_id.context("user not found")?;

        if let AppSessionState::Registering(registration) = session.state {
            let passkey = state
                .webauthn
                .finish_passkey_registration(&req.credential, &registration)
                .context("failed to finish passkey registration")?;

            let mut database = state.database.lock().map_err(Error::from_poison)?;

            database
                .values_mut()
                .find(|user| user.id == user_id)
                .context(format!("no user found for uuid {user_id}"))?
                .passkey = Some(passkey);

            info!("user is registered on the backend");

            Ok(Json(FinishRegistrationResponse { success: true }))
        } else {
            Err(Error::new("invalid session state"))
        }
    }
}
