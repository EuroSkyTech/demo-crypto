use api::*;
use gloo_net::http::Request;

use crate::error::{Context, Result};

pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(endpoint: &str) -> Self {
        Client {
            endpoint: endpoint.to_owned(),
        }
    }

    pub async fn auth_start(
        &self,
        req: StartAuthenticationRequest,
    ) -> Result<StartAuthenticationResponse> {
        let req = Request::post(&self.url("auth/start"))
            .json(&req)
            .context("failed to serialize authentication start request")?
            .send()
            .await
            .context("failed to send authentication start request")?;

        let res = req
            .json()
            .await
            .context("failed to parse authentication start response")?;

        Ok(res)
    }

    pub async fn auth_finish(
        &self,
        req: FinishAuthenticationRequest,
    ) -> Result<FinishAuthenticationResponse> {
        let req = Request::post(&self.url("auth/finish"))
            .json(&req)
            .context("failed to serialize authentication finish request")?
            .send()
            .await
            .context("failed to send authentication finish request")?;

        let res = req
            .json()
            .await
            .context("failed to parse authentication finish response")?;

        Ok(res)
    }

    pub async fn register_start(
        &self,
        req: StartRegistrationRequest,
    ) -> Result<StartRegistrationResponse> {
        let req = Request::post(&self.url("register/start"))
            .json(&req)
            .context("failed to serialize registration start request")?
            .send()
            .await
            .context("failed to send registration start request")?;

        let res = req
            .json()
            .await
            .context("failed to parse registration start response")?;

        Ok(res)
    }

    pub async fn register_finish(
        &self,
        req: FinishRegistrationRequest,
    ) -> Result<FinishRegistrationResponse> {
        let req = Request::post(&self.url("register/finish"))
            .json(&req)
            .context("failed to serialize registration finish request")?
            .send()
            .await
            .context("failed to send registration finish request")?;

        let res = req
            .json()
            .await
            .context("failed to parse registration finish response")?;

        Ok(res)
    }

    fn url(&self, path: &str) -> String {
        format!("https://{}/{}", self.endpoint, path)
    }
}
