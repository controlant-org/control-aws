use anyhow::Result;

use control_aws;

#[tokio::main]
async fn main() -> Result<()> {
  let ref config = aws_config::load_from_env().await;
  let accounts = control_aws::org::discover_accounts(config.into()).await?;

  println!("{:?}", accounts);

  Ok(())
}
