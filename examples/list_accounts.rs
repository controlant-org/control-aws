use anyhow::Result;

use control_aws::org;

#[tokio::main]
async fn main() -> Result<()> {
  let c = aws_config::load_from_env().await;
  let accounts = org::discover_accounts(&c).await?;

  println!("{:?}", accounts);

  Ok(())
}
