# 🛰 Sentinel (Rust Monitoring Bot)

![Rust](https://img.shields.io/badge/Language-Rust-orange?style=flat-square)
![Docker](https://img.shields.io/badge/Deploy-Docker-blue?style=flat-square)
![Status](https://img.shields.io/badge/Status-Production--Ready-green?style=flat-square)

**Sentinel** — это высокопроизводительный, асинхронный Telegram-бот для мониторинга доступности веб-ресурсов.

Написан на **Rust** с использованием экосистемы **Tokio**. Способен отслеживать состояние сотен сайтов одновременно благодаря конкурентной обработке запросов, потребляя при этом минимум системных ресурсов (15-30 MB RAM).

---

## ⚡️ Ключевые возможности

*   **🚀 Параллельные проверки:** Использует `futures::join_all` для одновременного опроса всех целей. Время проверки 100 сайтов равно времени ответа самого медленного из них.
*   **🛡 Smart Retry System:** Защита от ложных срабатываний ("flapping"). Бот отправляет алерт только после 3-х неудачных попыток подряд.
*   **💾 Persistent Storage:** Все данные хранятся в **SQLite**. При использовании Docker база вынесена в Volume и переживает перезапуски.
*   **📱 Интерактивные алерты:** Уведомления приходят с кнопкой `🔄 Перепроверить`, позволяющей мгновенно проверить статус упавшего сайта вручную.
*   **🐳 Docker Ready:** Полностью готов к деплою (Multi-stage build, маленький размер образа).

---

## 🛠 Технический стек

| Компонент | Технология | Описание |
| :--- | :--- | :--- |
| **Язык** | Rust 🦀 | Безопасность памяти, быстродействие |
| **Runtime** | Tokio | Асинхронное ядро приложения |
| **Bot Framework** | Teloxide | Взаимодействие с Telegram API |
| **Database** | SQLx + SQLite | Асинхронная работа с БД, миграции |
| **HTTP Client** | Reqwest | Сетевые запросы с пулингом соединений |
| **Container** | Docker | Debian Slim base image |

---

## 🚀 Установка и запуск

### Вариант 1: Docker (Рекомендуется)

1.  **Клонируйте репозиторий:**
    ```bash
    git clone https://github.com/your-username/sentinel.git
    cd sentinel
    ```

2.  **Создайте файл `.env`:**
    ```bash
    TELOXIDE_TOKEN=ваш_токен_от_botfather
    DATABASE_URL=sqlite:data/sentinel.db
    RUST_LOG=info
    ```

3.  **Запустите через Docker Compose:**
    ```bash
    docker-compose up -d --build
    ```
    *Бот запустится, база данных автоматически создастся в папке `sentinel_data`.*

### Вариант 2: Локальный запуск (Rust)

1.  Убедитесь, что установлен **Rust** и **Cargo**.
2.  Настройте `.env` (путь к БД измените на локальный):
    ```bash
    DATABASE_URL=sqlite:sentinel.db
    ```
3.  Запустите:
    ```bash
    cargo run --release
    ```

---

## 🎮 Команды бота

| Команда | Описание | Пример |
| :--- | :--- | :--- |
| `/start` | Инициализация и приветствие | |
| `/add <url>` | Добавить сайт на мониторинг | `/add https://google.com` |
| `/list` | Показать статус всех целей | |
| `/check` | **Принудительная проверка** всех сайтов | |
| `/remove <id>` | Удалить сайт из мониторинга | `/remove 1` |
| `/help` | Справка по командам | |

---

## 📂 Структура проекта

*   `src/main.rs` — Точка входа, инициализация `Dispatcher` и `Tokio Runtime`.
*   `src/handlers.rs` — Обработка команд `/start`, `/add` и Callback-кнопок.
*   `src/monitor.rs` — Фоновый процесс (Task B), цикл проверки и логика Retry.
*   `src/db.rs` — Слой доступа к данным (DAO), SQL-запросы.

---

*Built with ❤️ in Rust*
