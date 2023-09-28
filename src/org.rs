//! Utilities for discovering AWS accounts

use aws_sdk_organizations::operation::list_accounts::ListAccountsError;
use aws_sdk_organizations::operation::list_tags_for_resource::ListTagsForResourceError;
use aws_smithy_http::{body::SdkBody, result::SdkError};
use aws_types::SdkConfig;
use http::Response;
use thiserror::Error;
use tokio::task::JoinSet;
use tokio_stream::StreamExt;

#[derive(Error, Debug)]
pub enum OrgError {
  #[error("failed to list accounts")]
  ListAccounts(#[from] SdkError<ListAccountsError, Response<SdkBody>>),
  #[error("failed to list tags")]
  ListTags(#[from] SdkError<ListTagsForResourceError, Response<SdkBody>>),
  #[error("failed to extract accounts")]
  BadAccounts,
  #[error("failed to extract account ID")]
  BadAccountId,
  #[error("failed to extract tags")]
  BadTags,
}

/// An AWS account for Controlant
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
  /// The AWS account ID
  pub id: String,
  /// The `environment`, usually maps to account name, but this is extract from a trusted source (tag that controlled in management account)
  pub environment: Option<String>,
  /// The `tier` of an account in Controlant convention, sourced from tags
  pub tier: Option<String>,
  /// The `domain` the account belongs to, in short form (domain ID), sourced from tags
  pub domain: Option<String>,
}

/// Discover all accounts in the organization, also extract Controlant-specific tags.
///
/// To use this function the provided `SdkConfig` must have the following permissions:
/// ```json
/// {
///   Version = "2012-10-17"
///   Statement = [
///     {
///       Effect = "Allow"
///       Action = [
///         "organizations:ListAccounts",
///         "organizations:ListTagsForResource",
///       ]
///       Resource = ["*"]
///     },
///   ]
/// }
/// ```
pub async fn discover_accounts(config: SdkConfig) -> Result<Vec<Account>, OrgError> {
  let client = aws_sdk_organizations::Client::new(&config);
  let mut la_pages = client.list_accounts().into_paginator().send();

  let mut ras = JoinSet::new();
  while let Some(p) = la_pages.next().await {
    for acc in p?.accounts.ok_or(OrgError::BadAccounts)?.into_iter() {
      let id = acc.id.ok_or(OrgError::BadAccountId)?;
      let client = client.clone();
      ras.spawn(async move { read_account(client, id).await });
    }
  }

  let mut accounts = Vec::new();

  while let Some(res) = ras.join_next().await {
    let acc = res.unwrap()?;
    accounts.push(acc);
  }

  Ok(accounts)
}

async fn read_account(client: aws_sdk_organizations::Client, id: String) -> Result<Account, OrgError> {
  let tags = client
    .list_tags_for_resource()
    .resource_id(id.clone())
    .send()
    .await?
    .tags
    .ok_or(OrgError::BadTags)?;

  let mut environment = None;
  let mut domain = None;
  let mut tier = None;

  for tag in tags {
    match tag.key.as_deref() {
      Some("catapult.controlant.com/environment") => environment = tag.value,
      Some("catapult.controlant.com/domain") => domain = tag.value,
      Some("catapult.controlant.com/tier") => tier = tag.value,
      _ => (),
    };
  }

  Ok(Account {
    id,
    environment,
    tier,
    domain,
  })
}
