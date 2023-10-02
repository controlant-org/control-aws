//! Utilities for AWS controllers

pub mod org;

use aws_config::{default_provider::region::DefaultRegionChain, SdkConfig};
use aws_types::region::Region;
use std::sync::Arc;

/// Build `SdkConfig` by assuming a role and in the specified region
pub async fn assume_role(role: impl Into<String>, region: Option<Region>) -> SdkConfig {
  use aws_config::{default_provider::credentials::DefaultCredentialsChain, sts::AssumeRoleProvider};

  let rg = match region {
    Some(r) => r,
    None => DefaultRegionChain::builder().build().region().await.unwrap(),
  };

  let provider = AssumeRoleProvider::builder(role)
    .session_name(env!("CARGO_PKG_NAME"))
    .region(rg.clone())
    .build(Arc::new(DefaultCredentialsChain::builder().build().await) as Arc<_>);

  aws_config::from_env()
    .credentials_provider(provider)
    .region(rg)
    .load()
    .await
}
