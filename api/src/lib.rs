use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs_proto::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StartRegistrationRequest {
    pub did: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartRegistrationResponse {
    pub challenge: CreationChallengeResponse,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishRegistrationRequest {
    pub credential: RegisterPublicKeyCredential,
}

// TODO: Drop
#[derive(Debug, Serialize, Deserialize)]
pub struct FinishRegistrationResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartAuthenticationRequest {
    pub did: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartAuthenticationResponse {
    pub challenge: RequestChallengeResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishAuthenticationRequest {
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishAuthenticationResponse {
    pub user_id: Uuid,
}
