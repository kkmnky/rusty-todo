use goose::prelude::*;

async fn access_root(user: &mut GooseUser) -> TransactionResult {
    let _response = user.get("/").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let host =
        std::env::var("LOADTEST_HOST").unwrap_or_else(|_| "http://localhost:8080".to_string());

    GooseAttack::initialize()?
        .set_default(GooseDefault::Host, host.as_str())?
        .register_scenario(scenario!("AccessRoot").register_transaction(transaction!(access_root)))
        .execute()
        .await?;

    Ok(())
}
