// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use url::Url;

use super::Host;

/// GitHub host implementation.
pub struct GithubHost {
    /// The owner of the repository.
    pub owner: String,
    /// The name of the repository.
    pub repo: String,

    /// The URL of the repository.
    pub url: Url,
}

impl GithubHost {
    pub fn from_url(url: &Url) -> Result<Self, super::Error> {
        let path = url.path();
        let mut parts = path.split('/').skip(1).filter(|x| !x.is_empty());
        let owner = parts.next().ok_or(super::Error::InvalidUrl)?.to_string();
        let repo = parts.next().ok_or(super::Error::InvalidUrl)?.to_string();
        Ok(Self {
            owner,
            repo,
            url: url.clone(),
        })
    }
}

#[async_trait]
impl Host for GithubHost {
    async fn versions(&self) -> Vec<String> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
