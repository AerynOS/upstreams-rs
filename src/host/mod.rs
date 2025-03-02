// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use github::GithubHost;
use plain::PlainHost;
use thiserror::Error;
use url::Url;

use async_trait::async_trait;

use crate::VersionMetadata;

pub mod github;
pub mod plain;

/// Common trait implemented by all repository host types
#[async_trait]
pub trait Host {
    /// Fetches all available versions for this repository
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError>;
}

/// Errors that can occur when interacting with repository hosts
#[derive(Error, Debug)]
pub enum HostError {
    /// The URL provided was not in a valid format for this host
    #[error("invalid URL format: {0}")]
    InvalidUrl(String),

    /// Failed to parse repository information from a valid URL
    #[error("failed to parse repository info: {0}")]
    ParseError(String),

    /// Failed to fetch data from the host's API
    #[error("API request failed: {context}")]
    ApiRequest {
        context: String,
        #[source]
        source: reqwest::Error,
    },

    /// Failed to parse data received from the API
    #[error("failed to parse API response: {context}")]
    ApiResponse {
        context: String,
        #[source]
        source: reqwest::Error,
    },

    /// The requested operation is not supported by this host
    #[error("operation not supported: {0}")]
    Unsupported(String),
}

pub fn from_url(url: &Url) -> Result<Box<dyn Host>, HostError> {
    match url.host_str() {
        Some("github.com") => Ok(Box::new(GithubHost::from_url(url)?)),
        _ => Ok(Box::new(PlainHost::from_url(url))),
    }
}
