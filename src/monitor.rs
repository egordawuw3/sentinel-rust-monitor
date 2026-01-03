use crate::db::{self, Site};
use sqlx::{Pool, Sqlite};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};
use std::time::Duration;
use tokio::time;
use reqwest::Client;

const CHECK_INTERVAL: u64 = 60;
const RETRY_DELAY: u64 = 5;
const MAX_RETRIES: usize = 3;

pub async fn start_monitoring(bot: Bot, pool: Pool<Sqlite>, client: Client) {
    log::info!("🛰 Monitor Task: STARTED");
    let mut interval = time::interval(Duration::from_secs(CHECK_INTERVAL));

    loop {
        interval.tick().await;
        match db::get_all_sites(&pool).await {
            Ok(sites) => {
                for site in sites {
                    let bot = bot.clone();
                    let client = client.clone();
                    tokio::spawn(async move {
                        check_url_with_retry(bot, client, site).await;
                    });
                }
            }
            Err(e) => log::error!("❌ DB Error: {}", e),
        }
    }
}

async fn check_url_with_retry(bot: Bot, client: Client, site: Site) {
    for attempt in 1..=MAX_RETRIES {
        if client.get(&site.url).send().await.map(|r| r.status().is_success()).unwrap_or(false) {
            return;
        }
        if attempt < MAX_RETRIES {
            time::sleep(Duration::from_secs(RETRY_DELAY)).await;
        }
    }
    send_alert(&bot, &site).await;
}

async fn send_alert(bot: &Bot, site: &Site) {
    let msg = format!(
        "🚨 <b>Сайт недоступен!</b>\n\n🔗 {}\n📉 Не отвечает после 3 попыток.\n\nПроверь, работает ли сервер или DNS.", 
        site.url
    );

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("🔄 Перепроверить", format!("check:{}", site.id))
    ]]);
    
    let _ = bot.send_message(UserId(site.user_id as u64), msg)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await;
}
