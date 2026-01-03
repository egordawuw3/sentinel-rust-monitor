use crate::db;
use sqlx::{Pool, Sqlite};
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};
use url::Url;
use reqwest::Client;
use futures::future::join_all;
use std::time::Duration;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды бота:")]
pub enum Command {
    #[command(description = "Начать работу")]
    Start,
    #[command(description = "Помощь")]
    Help,
    #[command(description = "Добавить сайт")]
    Add(String),
    #[command(description = "Удалить сайт")]
    Remove(i64),
    #[command(description = "Мои сайты")]
    List,
    #[command(description = "Проверить сейчас")]
    Check,
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command, pool: Pool<Sqlite>, client: Client) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            let text = "👋 <b>Привет! Я бот-мониторинг.</b>\n\n\
            Я буду проверять твои сайты каждую минуту. Если какой-то из них перестанет работать, я сразу пришлю уведомление.\n\n\
            <b>Как пользоваться:</b>\n\
            1. Добавь сайт: <code>/add ссылка</code>\n\
            2. Посмотри список: <code>/list</code>\n\
            3. Если нужно, удали: <code>/remove id</code>";
            
            bot.send_message(msg.chat.id, text).parse_mode(ParseMode::Html).await?;
        }

        Command::Help => {
            let desc = Command::descriptions().to_string();
            bot.send_message(msg.chat.id, format!("📄 <b>Список команд:</b>\n\n{}", desc))
                .parse_mode(ParseMode::Html).await?;
        }

        Command::Add(url_str) => {
            let url = match Url::parse(&url_str) {
                Ok(u) if u.scheme().starts_with("http") => u,
                _ => {
                    bot.send_message(msg.chat.id, "❌ <b>Ошибка:</b> Ссылка должна начинаться с <code>http://</code> или <code>https://</code>").parse_mode(ParseMode::Html).await?;
                    return Ok(());
                }
            };
            let chat_id = msg.chat.id.0;
            if let Ok(id) = db::add_site(&pool, chat_id, url.to_string()).await {
                bot.send_message(msg.chat.id, format!("✅ <b>Сайт добавлен!</b>\n🆔 ID: <code>{}</code>\n🔗 {}", id, url)).parse_mode(ParseMode::Html).await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Ошибка базы данных.").await?;
            }
        }

        Command::List => {
            let chat_id = msg.chat.id.0;
            if let Ok(sites) = db::get_user_sites(&pool, chat_id).await {
                if sites.is_empty() {
                    bot.send_message(msg.chat.id, "📭 <b>Список пуст.</b>\nДобавьте сайт командой <code>/add</code>").parse_mode(ParseMode::Html).await?;
                } else {
                    let mut text = "📋 <b>Ваши сайты на мониторинге:</b>\n\n".to_string();
                    
                    // FIX: Добавил '&' перед sites, чтобы перебирать ссылки, а не перемещать данные
                    for site in &sites {
                        text.push_str(&format!("🔹 <b>ID: {}</b> — {}\n", site.id, site.url));
                    }
                    // Теперь sites.len() работает, потому что массив не был уничтожен
                    text.push_str(&format!("\n<i>Всего сайтов: {}</i>", sites.len()));
                    
                    bot.send_message(msg.chat.id, text).parse_mode(ParseMode::Html).disable_web_page_preview(true).await?;
                }
            }
        }

        Command::Remove(id) => {
             let chat_id = msg.chat.id.0;
             if db::remove_site(&pool, id, chat_id).await.unwrap_or(false) {
                 bot.send_message(msg.chat.id, "🗑 <b>Сайт удален из списка.</b>").parse_mode(ParseMode::Html).await?;
             } else {
                 bot.send_message(msg.chat.id, "⚠️ Сайт с таким ID не найден.").await?;
             }
        }

        Command::Check => {
            let chat_id = msg.chat.id.0;
            bot.send_message(msg.chat.id, "🔎 <b>Проверяю доступность...</b>").parse_mode(ParseMode::Html).await?;

            if let Ok(sites) = db::get_user_sites(&pool, chat_id).await {
                if sites.is_empty() { return Ok(()); }

                let tasks = sites.iter().map(|site| {
                    let client = client.clone();
                    let url = site.url.clone();
                    async move {
                        let result = tokio::time::timeout(
                            Duration::from_secs(5), 
                            client.get(&url).send()
                        ).await;
                        (url, result)
                    }
                });

                let results = join_all(tasks).await;

                let mut report = String::from("📊 <b>Результат проверки:</b>\n\n");
                for (url, res) in results {
                    let status_icon = match res {
                        Ok(Ok(resp)) if resp.status().is_success() => "🟢 Работает".to_string(),
                        Ok(Ok(resp)) => format!("🔴 Ошибка {}", resp.status().as_u16()),
                        Ok(Err(_)) => "❌ Недоступен".to_string(),
                        Err(_) => "⏱ Таймаут".to_string(),
                    };
                    report.push_str(&format!("{} — {}\n", status_icon, url));
                }
                
                bot.send_message(msg.chat.id, report).parse_mode(ParseMode::Html).disable_web_page_preview(true).await?;
            }
        }
    };
    Ok(())
}

pub async fn callback_handler(bot: Bot, q: CallbackQuery, pool: Pool<Sqlite>, client: Client) -> ResponseResult<()> {
    if let Some(data) = q.data {
        if let Some(id_str) = data.strip_prefix("check:") {
            if let Ok(id) = id_str.parse::<i64>() {
                if let Ok(site) = db::get_site(&pool, id).await {
                    let status_text = match client.get(&site.url).timeout(Duration::from_secs(5)).send().await {
                        Ok(res) if res.status().is_success() => "✅ Доступен (200 OK)".to_string(),
                        Ok(res) => format!("⚠️ Всё еще ошибка (Код: {})", res.status()),
                        Err(_) => "❌ Не открывается".to_string(),
                    };
                    
                    bot.answer_callback_query(q.id).text("Проверка завершена").await?;
                    
                    if let Some(msg) = q.message {
                        bot.send_message(msg.chat.id, format!("🔍 <b>Ручная проверка:</b>\n\nСайт: {}\nСтатус: {}", site.url, status_text))
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                } else {
                    bot.answer_callback_query(q.id).text("Сайт не найден").await?;
                }
            }
        }
    }
    Ok(())
}
