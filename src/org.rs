//! Utilities for discovering AWS accounts

use aws_sdk_organizations::operation::list_accounts::ListAccountsError;
use aws_sdk_organizations::operation::list_tags_for_resource::ListTagsForResourceError;
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use aws_smithy_runtime_api::client::result::SdkError;
use thiserror::Error;
use tokio::task::JoinSet;

#[derive(Error, Debug)]
pub enum OrgError {
  #[error("failed to list accounts: {0}")]
  ListAccounts(#[from] SdkError<ListAccountsError, HttpResponse>),
  #[error("failed to list tags: {0}")]
  ListTags(#[from] SdkError<ListTagsForResourceError, HttpResponse>),
  #[error("failed to extract accounts")]
  BadAccounts,
  #[error("failed to extract account ID")]
  BadAccountId,
  #[error("failed to extract tags")]
  BadTags,
}

/// An AWS account for Controlant
#[derive(Debug, Clone, PartialEq, Eq)]
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
pub async fn discover_accounts(config: aws_sdk_organizations::Config) -> Result<Vec<Account>, OrgError> {
  let client = aws_sdk_organizations::Client::from_conf(config);
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
    match tag.key.as_str() {
      "catapult.controlant.com/environment" => environment = Some(tag.value),
      "catapult.controlant.com/domain" => domain = Some(tag.value),
      "catapult.controlant.com/tier" => tier = Some(tag.value),
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
