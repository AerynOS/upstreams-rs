// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use async_trait::async_trait;
use url::Url;

use crate::{versioning::VersionExtractor, AssetKind, VersionMetadata, VersionedAsset};

use super::{Host, HostError};

/// Fallback host implementation for plain URLs. Used when no other host implementation
/// matches the provided URL format. Simply stores the raw URL and path information
/// without any special handling.
pub struct PlainHost {
    /// The path portion of the URL, with leading slashes trimmed
    pub path: String,

    /// The complete URL of the repository
    pub url: Url,

    pub directory: String,
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
        // collect the path but not the last element
        let mut segments: Vec<String> = url
            .path_segments()
            .map(|segments| segments.map(String::from).collect())
            .unwrap_or_default();
        segments.pop();
        let directory = segments.join("/");

        Self {
            path: path.to_string(),
            url: url.clone(),
            directory,
        }
    }
}

#[async_trait]
impl Host for PlainHost {
    /// Not implemented for PlainHost - returns unimplemented error
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError> {
        let url = format!(
            "{}://{}/{}/",
            self.url.scheme(),
            self.url.host().map(|s| s.to_string()).unwrap_or_default(),
            self.directory
        );
        let client = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "upstreams-rs".to_string())
            .send()
            .await
            .map_err(|e| HostError::ApiRequest {
                context: "failed to fetch directory listing".into(),
                source: e,
            })?;

        let body = client.text().await.map_err(|e| HostError::ApiRequest {
            context: "failed to read directory listing".into(),
            source: e,
        })?;

        let doc = scraper::Html::parse_document(&body);
        let selector =
            scraper::Selector::parse("a").map_err(|e| HostError::ParseError(e.to_string()))?;

        let matcher = VersionExtractor::new().map_err(|e| HostError::ParseError(e.to_string()))?;
        let match_us = matcher
            .extract(self.url.as_ref())
            .map_err(|e| HostError::ParseError(e.to_string()))?;

        let mut versions = HashMap::new();
        for element in doc.select(&selector) {
            let href = element.value().attr("href").unwrap_or_default();

            let mut downloads = vec![];
            if let Ok(m) = matcher.extract(href) {
                if m.name == match_us.name {
                    // Construct full URL
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!(
                            "{}{}/{}",
                            self.url.host().map(|s| s.to_string()).unwrap_or_default(),
                            self.directory,
                            href.trim_start_matches('.')
                        )
                    };

                    downloads.push(VersionedAsset {
                        url: full_url,
                        kind: AssetKind::Release,
                        released_at: None,
                        updated_at: None,
                    });

                    versions
                        .entry(m.version)
                        .or_insert_with(Vec::new)
                        .extend(downloads);
                }
            }
        }

        let mut versions_set = vec![];
        for (version, downloads) in versions.iter() {
            let metadata = VersionMetadata {
                version: version.to_string(),
                downloads: downloads.clone(),
                release_notes: None,
                released_at: None,
            };
            versions_set.push(metadata);
        }

        Ok(versions_set)
    }
}
