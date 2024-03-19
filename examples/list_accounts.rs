use anyhow::Result;

use control_aws::org;

#[tokio::main]
async fn main() -> Result<()> {
  let ref c = aws_config::load_from_env().await;
  let accounts = org::discover_accounts(org::config(c)).await?;

  println!("{:?}", accounts);

  Ok(())
}
