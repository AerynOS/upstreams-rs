// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use url::Url;

use crate::VersionMetadata;

use super::{Host, HostError};

/// Fallback host implementation for plain URLs. Used when no other host implementation
/// matches the provided URL format. Simply stores the raw URL and path information
/// without any special handling.
pub struct PlainHost {
    /// The path portion of the URL, with leading slashes trimmed
    pub path: String,

    /// The complete URL of the repository
    pub url: Url,
}

impl PlainHost {
    /// Creates a new PlainHost instance from a URL
    ///
    /// # Arguments
    /// * `url` - Reference to the URL to create the host from
    ///
    /// # Returns
    /// A new PlainHost instance with the path and URL information
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
    /// Not implemented for PlainHost - returns unimplemented error
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError> {
        Err(HostError::Unsupported(
            "plain URL hosts do not support version listing".into(),
        ))
    }
}
