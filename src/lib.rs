//! Utilities for AWS controllers

pub mod org;

use aws_config::{default_provider::region::DefaultRegionChain, Region, SdkConfig};

/// Build `SdkConfig` by assuming a role and in the specified region
pub async fn assume_role(role: impl Into<String>, region: Option<Region>) -> SdkConfig {
  use aws_config::sts::AssumeRoleProvider;

  let rg = match region {
    Some(r) => r,
    None => DefaultRegionChain::builder().build().region().await.unwrap(),
  };

  let provider = AssumeRoleProvider::builder(role)
    .session_name(env!("CARGO_PKG_NAME"))
    .region(rg.clone())
    .build()
    .await;

  aws_config::from_env()
    .credentials_provider(provider)
    .region(rg)
    .load()
    .await
}
