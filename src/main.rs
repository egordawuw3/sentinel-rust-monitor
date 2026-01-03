mod db;
mod handlers;
mod monitor;

use dotenv::dotenv;
use teloxide::prelude::*;
use std::env;
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("🚀 Sentinel System Starting...");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&db_url).await?;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("SentinelBot/Pro")
        .build()?;

    let bot = Bot::from_env();

    // Background Monitor
    let bot_clone = bot.clone();
    let pool_clone = pool.clone();
    let client_clone = client.clone();
    tokio::spawn(async move {
        monitor::start_monitoring(bot_clone, pool_clone, client_clone).await;
    });

    log::info!("✅ Bot Listener online");

    // Routing
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<handlers::Command>()
                .endpoint(handlers::answer)
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(handlers::callback_handler)
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("🛑 Shutting down.");
    Ok(())
}
