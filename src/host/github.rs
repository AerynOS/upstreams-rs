// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, info};
use url::Url;

use crate::{AssetKind, VersionMetadata, VersionedAsset};

use super::{Host, HostError};

/// The GitHub API version to use for requests
const GH_API_VERSION: &str = "2022-11-28";

/// GitHub host implementation for interacting with GitHub repositories.
pub struct GithubHost {
    /// The owner of the repository.
    pub owner: String,
    /// The name of the repository.
    pub repo: String,
    /// The URL of the repository.
    pub url: Url,
}

impl GithubHost {
    /// Creates a new GithubHost instance from a GitHub repository URL.
    ///
    /// # Arguments
    /// * `url` - The GitHub repository URL to parse
    ///
    /// # Returns
    /// A Result containing either the GithubHost instance or an error if the URL is invalid
    pub fn from_url(url: &Url) -> Result<Self, HostError> {
        debug!("Creating GithubHost from URL: {}", url);
        let path = url.path();
        let mut parts = path.split('/').skip(1).filter(|x| !x.is_empty());
        let owner = parts
            .next()
            .ok_or_else(|| HostError::ParseError("missing repository owner in GitHub URL".into()))?
            .to_string();
        let repo = parts
            .next()
            .ok_or_else(|| HostError::ParseError("missing repository name in GitHub URL".into()))?
            .to_string();
        info!("Created GithubHost for {}/{}", owner, repo);
        Ok(Self {
            owner,
            repo,
            url: url.clone(),
        })
    }

    fn gh_client(&self, url: &str) -> Result<reqwest::RequestBuilder, HostError> {
        debug!("Creating GitHub API client for URL: {}", url);
        let client = reqwest::Client::new();
        let client = client
            .get(url)
            .header("Accept", "application/vnd.github.v3+json".to_string())
            .header("User-Agent", "upstreams-rs".to_string())
            .header("X-GitHub-Api-Version", GH_API_VERSION);
        Ok(client)
    }

    /// Fetches tags from the GitHub REST API.
    ///
    /// # Returns
    /// A Result containing either a vector of GithubTagResponse or an error
    async fn fetch_tags(&self) -> Result<Vec<GithubTagResponse>, HostError> {
        let tag_url = format!(
            "https://api.github.com/repos/{}/{}/tags",
            self.owner, self.repo
        );
        debug!("Fetching tags from: {}", tag_url);

        let tags = self
            .gh_client(&tag_url)?
            .send()
            .await
            .map_err(|e| HostError::ApiRequest {
                context: "failed to fetch tags".into(),
                source: e,
            })?
            .json::<Vec<GithubTagResponse>>()
            .await
            .map_err(|e| HostError::ApiResponse {
                context: "failed to parse tags response".into(),
                source: e,
            })?;

        info!("Successfully fetched {} tags", tags.len());
        Ok(tags)
    }

    /// Fetches releases from the GitHub REST API.
    ///
    /// # Returns
    /// A Result containing either a vector of GithubReleaseResponse or an error
    async fn fetch_releases(&self) -> Result<Vec<GithubReleaseResponse>, HostError> {
        let releases_url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        );
        debug!("Fetching releases from: {}", releases_url);

        let releases = self
            .gh_client(&releases_url)?
            .send()
            .await
            .map_err(|e| HostError::ApiRequest {
                context: "failed to fetch releases".into(),
                source: e,
            })?
            .json::<Vec<GithubReleaseResponse>>()
            .await
            .map_err(|e| HostError::ApiResponse {
                context: "failed to parse releases response".into(),
                source: e,
            })?;

        info!("Successfully fetched {} releases", releases.len());
        Ok(releases)
    }
}

/// Response structure for the GitHub tags REST API endpoint.
#[derive(Deserialize, Debug)]
pub struct GithubTagResponse {
    /// The name of the tag
    pub name: String,
    /// URL for downloading the repository as a zip file at this tag (requires authentication)
    pub zipball_url: String,
    /// URL for downloading the repository as a tarball at this tag (requires authentication)
    pub tarball_url: String,
    /// Information about the commit this tag points to
    pub commit: GithubTagCommit,
    /// GitHub's internal node ID for this tag
    pub node_id: String,
}

/// Response structure for commit information in a GitHub tag response.
#[derive(Deserialize, Debug)]
pub struct GithubTagCommit {
    /// The SHA hash of the commit
    pub sha: String,
    /// The API URL for this commit
    pub url: String,
}

/// Response structure for the GitHub releases REST API endpoint.
#[derive(Deserialize, Debug)]
pub struct GithubReleaseResponse {
    /// The name of the tag associated with this release
    pub tag_name: String,
    /// The title of the release
    pub name: String,
    /// The description/body text of the release
    pub body: String,
    /// List of assets attached to this release
    pub assets: Vec<GithubReleaseAsset>,
    /// URL for downloading the repository as a tarball at this release
    pub tarball_url: String,
    /// URL for downloading the repository as a zip file at this release
    pub zipball_url: String,
    /// When this release was published
    pub published_at: String,
}

/// Response structure for release assets in a GitHub release response.
#[derive(Deserialize, Debug)]
pub struct GithubReleaseAsset {
    /// The filename of the asset
    pub name: String,
    /// Optional label describing the asset
    pub label: Option<String>,
    /// MIME type of the asset
    pub content_type: String,
    /// Current state of the asset (e.g. "uploaded")
    pub state: String,
    /// File size in bytes
    pub size: u64,
    /// Number of times this asset has been downloaded
    pub download_count: u64,
    /// When this asset was created
    pub created_at: String,
    /// When this asset was last updated
    pub updated_at: String,
    /// Direct download URL for the asset
    pub browser_download_url: String,
}

#[async_trait]
impl Host for GithubHost {
    /// Fetches all versions available for this repository
    ///
    /// # Returns
    /// A Result containing either a vector of VersionedAsset or an error
    async fn versions(&self) -> Result<Vec<VersionMetadata>, HostError> {
        debug!("Fetching versions for {}/{}", self.owner, self.repo);
        let tags = self.fetch_tags().await?;
        let releases = self.fetch_releases().await?;

        // Combine tags and releases into a single list of version strings
        let version_strings = tags
            .iter()
            .map(|tag| tag.name.clone())
            .chain(releases.iter().map(|release| release.tag_name.clone()))
            .collect::<BTreeSet<String>>();

        info!("Found {} unique versions", version_strings.len());
        let mut found = Vec::new();

        for version in version_strings {
            debug!("Processing version: {}", version);
            let mut downloads = BTreeSet::new();
            for tag in tags.iter().filter(|tag| tag.name == version) {
                downloads.insert(VersionedAsset {
                    url: tag.tarball_url.clone(),
                    kind: AssetKind::Autogenerated,
                });
            }
            for release in releases
                .iter()
                .filter(|release| release.tag_name == version)
            {
                downloads.insert(VersionedAsset {
                    url: release.tarball_url.clone(),
                    kind: AssetKind::Release,
                });
                for asset in release.assets.iter() {
                    // TODO: Specialise asset kind based on content type
                    let kind = AssetKind::Autogenerated;
                    downloads.insert(VersionedAsset {
                        url: asset.browser_download_url.clone(),
                        kind,
                    });
                }
            }

            // Find the release notes for this version
            let release_notes = releases
                .iter()
                .find(|release| release.tag_name == version)
                .map(|release| release.body.clone());
            found.push(VersionMetadata {
                version,
                downloads: downloads.into_iter().collect(),
                release_notes,
            });
        }

        info!("Processed {} versions with assets", found.len());
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the from_url function correctly handles valid and invalid GitHub URLs
    #[tokio::test]
    async fn test_from_url() {
        let valid_urls = [
            "https://github.com/rust-lang/rust/archive/refs/tags/1.73.0.tar.gz",
            "https://github.com/microsoft/vscode/archive/refs/tags/1.84.0.tar.gz",
            "https://github.com/torvalds/linux/archive/refs/tags/v6.6.tar.gz",
            "https://github.com/redis/redis/archive/refs/tags/7.2.1.tar.gz",
        ];

        let invalid_urls = [
            "https://github.com",
            "https://github.com/test",
            "https://github.com/",
            "https://github.com/test//",
            "https://github.com//test",
        ];

        for url in valid_urls {
            let url = Url::parse(url).unwrap();
            let l = GithubHost::from_url(&url);
            assert!(l.is_ok())
        }

        for url in invalid_urls {
            let url = Url::parse(url).unwrap();
            let l = GithubHost::from_url(&url);
            assert!(l.is_err())
        }
    }
}
