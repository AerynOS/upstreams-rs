// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashMap, vec};

use async_trait::async_trait;
use serde::Deserialize;
use url::Url;

use crate::{AssetKind, VersionMetadata, VersionedAsset};

use super::{Host, HostError};

/// A Host implementation for accessing GNOME project releases
///
/// This struct provides functionality to interact with GNOME's download server
/// and retrieve version information for GNOME projects.
pub struct GnomeHost {
    /// The project name (i.e. "gnome-shell", "gtk", etc)
    pub project: String,

    /// The complete URL to the project's download location
    pub url: Url,
}

/// Metadata about a specific version of a GNOME project
///
/// This struct contains information about available download formats and checksums
/// for a specific version release. GNOME releases typically prefer the `tar.xz` format,
/// but also support `tar.gz` and `tar.bz2` as alternatives.
///
/// # Fields
///
/// * `news` - Optional URL to release notes/news
/// * `changes` - Optional URL to changelog
/// * `sha256sum` - Optional SHA256 checksum of the release files
/// * `tarxz` - Optional URL to .tar.xz archive
/// * `targz` - Optional URL to .tar.gz archive
/// * `tarbz2` - Optional URL to .tar.bz2 archive
#[derive(Deserialize)]
pub struct GnomeCacheComponentFile {
    pub news: Option<String>,
    pub changes: Option<String>,
    pub sha256sum: Option<String>,
    #[serde(rename = "tar.xz")]
    pub tarxz: Option<String>,
    #[serde(rename = "tar.gz")]
    pub targz: Option<String>,
    #[serde(rename = "tar.bz2")]
    pub tarbz2: Option<String>,
}

/// Maps version strings to component files
pub type GnomeCacheComponent = HashMap<String, GnomeCacheComponentFile>;

/// Maps component names to their versions
pub type GnomeCacheVersion = HashMap<String, GnomeCacheComponent>;

/// Response format for the GNOME cache.json API
#[derive(Deserialize)]
pub struct GnomeCacheResponse {
    pub format: u8,
    pub components: GnomeCacheVersion,
    pub versions: HashMap<String, Vec<String>>,
    pub meta: HashMap<String, Vec<String>>,
}

impl GnomeHost {
    /// Creates a new GnomeHost instance from a URL
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
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError> {
        let uri = format!(
            "https://download.gnome.org/sources/{}/cache.json",
            self.project
        );
        let response = reqwest::get(&uri)
            .await
            .map_err(|e| HostError::ApiRequest {
                context: "failed to fetch cache data".into(),
                source: e,
            })?
            .json::<GnomeCacheResponse>()
            .await
            .map_err(|e| HostError::ApiResponse {
                context: "failed to parse cache data".into(),
                source: e,
            })?;

        let mut versions_set = vec![];

        for (_component, versions) in response.components.iter() {
            for (version, files) in versions.iter() {
                let mut downloads = vec![];
                if let Some(tarxz) = files.tarxz.as_ref() {
                    downloads.push(VersionedAsset {
                        url: format!(
                            "https://download.gnome.org/sources/{}/{}",
                            self.project, tarxz
                        ),
                        kind: AssetKind::Release,
                        released_at: None,
                        updated_at: None,
                    });
                }
                if let Some(targz) = files.targz.as_ref() {
                    downloads.push(VersionedAsset {
                        url: format!(
                            "https://download.gnome.org/sources/{}/{}",
                            self.project, targz
                        ),
                        kind: AssetKind::Release,
                        released_at: None,
                        updated_at: None,
                    });
                }
                if let Some(tarbz2) = files.tarbz2.as_ref() {
                    downloads.push(VersionedAsset {
                        url: format!(
                            "https://download.gnome.org/sources/{}/{}",
                            self.project, tarbz2
                        ),
                        kind: AssetKind::Release,
                        released_at: None,
                        updated_at: None,
                    });
                }

                // TODO: fetch release notes and released_at from additional metadata
                let version = VersionMetadata {
                    version: version.clone(),
                    downloads,
                    release_notes: None,
                    released_at: None,
                };
                versions_set.push(version);
            }
        }

        Ok(versions_set)
    }
}
