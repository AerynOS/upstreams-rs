// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use url::Url;

use super::Host;

/// Fallback host implementation for plain URLs.
pub struct PlainHost {
    pub path: String,

    /// The URL of the repository.
    pub url: Url,
}

impl PlainHost {
    pub fn from_url(url: &Url) -> Self {
        let path = url.path();
        let path = path.trim_start_matches('/');
        Self {
            path: path.to_string(),
            url: url.clone(),
        }
    }
}

#[async_trait]
impl Host for PlainHost {
    async fn versions(&self) -> Vec<String> {
        unimplemented!()
    }
}
