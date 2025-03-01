// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Upstream host-specific functionality.

use async_trait::async_trait;
use github::GithubHost;
use plain::PlainHost;
use thiserror::Error;
use url::Url;

mod github;
mod plain;

#[async_trait]
pub trait Host {
    async fn versions(&self) -> Vec<String>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to fetch versions")]
    FetchVersions,

    #[error("invalid URL")]
    InvalidUrl,
}

pub fn from_url(url: &Url) -> Result<Box<dyn Host>, Error> {
    match url.host_str() {
        Some("github.com") => Ok(Box::new(GithubHost::from_url(url)?)),
        _ => Ok(Box::new(PlainHost::from_url(url))),
    }
}
