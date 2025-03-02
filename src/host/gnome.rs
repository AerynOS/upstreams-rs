// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use url::Url;

use crate::VersionMetadata;

use super::{Host, HostError};

/// GNOME host implementation
pub struct GnomeHost {
    /// The project name (i.e. "gnome-shell")
    pub project: String,

    /// The complete URL
    pub url: Url,
}

impl GnomeHost {
    /// Creates a new GnomeHost instance from a URL
    ///
    /// # Arguments
    /// * `url` - Reference to the URL to create the host from
    ///
    /// # Returns
    /// A new GnomeHost instance with the path and URL information
    pub fn from_url(url: &Url) -> Result<Self, HostError> {
        let parts = url
            .path_segments()
            .ok_or(HostError::InvalidUrl("invalid URL format".into()))?;
        let path = parts.filter(|p| !p.is_empty()).collect::<Vec<&str>>();
        let source_path = path.first().unwrap_or(&(""));
        if *source_path != "sources" {
            return Err(HostError::InvalidUrl("invalid URL format".into()));
        }
        let project = path
            .get(1)
            .ok_or(HostError::InvalidUrl("invalid URL format".into()))?;
        Ok(Self {
            project: project.to_string(),
            url: url.clone(),
        })
    }
}

#[async_trait]
impl Host for GnomeHost {
    /// Not implemented for GnomeHost - returns unimplemented error
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError> {
        Err(HostError::Unsupported(
            "gnome URL hosts do not support version listing".into(),
        ))
    }
}
