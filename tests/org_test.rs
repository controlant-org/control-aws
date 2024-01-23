use aws_config::{BehaviorVersion, Region};
use aws_sdk_organizations as aws_org;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;

use control_aws::org::{discover_accounts, Account};

#[tokio::test]
async fn test_discover_accounts() {
  let replay_client = StaticReplayClient::new(vec![
    ReplayEvent::new(
      http::Request::builder()
        .method("GET")
        .uri("https://organizations.us-east-1.amazonaws.com/")
        .header("X-Amz-Target", "AWSOrganizationsV20161128.ListAccounts")
        .body(SdkBody::from("{}"))
        .unwrap(),
      http::Response::builder()
        .status(200)
        .body(SdkBody::from(include_str!("./replays/list_accounts.json")))
        .unwrap(),
    ),
    ReplayEvent::new(
      http::Request::builder()
        .method("POST")
        .uri("https://organizations.us-east-1.amazonaws.com/")
        .header("X-Amz-Target", "AWSOrganizationsV20161128.ListTagsForResource")
        .body(SdkBody::from(r#"{"ResourceId": "11111"}"#))
        .unwrap(),
      http::Response::builder()
        .status(200)
        .body(SdkBody::from(include_str!("./replays/tags_11111.json")))
        .unwrap(),
    ),
    ReplayEvent::new(
      http::Request::builder()
        .method("POST")
        .uri("https://organizations.us-east-1.amazonaws.com/")
        .header("X-Amz-Target", "AWSOrganizationsV20161128.ListTagsForResource")
        .body(SdkBody::from(r#"{"ResourceId": "22222"}"#))
        .unwrap(),
      http::Response::builder()
        .status(200)
        .body(SdkBody::from(include_str!("./replays/tags_22222.json")))
        .unwrap(),
    ),
  ]);

  let config = aws_org::Config::builder()
    .behavior_version(BehaviorVersion::latest())
    .credentials_provider(aws_org::config::Credentials::for_tests())
    .region(Region::new("us-east-1"))
    .http_client(replay_client.clone())
    .build();

  let accounts = discover_accounts(config).await.unwrap();

  assert_eq!(
    accounts,
    vec![
      Account {
        id: "11111".to_string(),
        environment: Some("development".to_string()),
        domain: Some("testing".to_string()),
        tier: Some("development".to_string()),
      },
      Account {
        id: "22222".to_string(),
        environment: None,
        domain: None,
        tier: None,
      }
    ]
  );

  replay_client.assert_requests_match(&[]);
}
