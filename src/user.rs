use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::io::{self, Write};
use rpassword::read_password;

pub fn register_user(conn: &Connection) -> Result<()> {
    let username = prompt("Введіть ім'я користувача: ")?;

    // Перевірка, чи існує вже користувач з таким ім'ям
    let exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username = ?1",
        params![&username],
        |row| row.get(0),
    ).context("Помилка при перевірці існування користувача")?;

    if exists > 0 {
        println!("Помилка: Ім'я користувача '{}' вже існує. Будь ласка, виберіть інше.", username);
        return Ok(());
    }

    let password = prompt_password("Введіть пароль: ")?;
    let password_hash = hash(&password, DEFAULT_COST).context("Хешування пароля не вдалося")?;

    match conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
        params![&username, &password_hash],
    ) {
        Ok(_) => {
            println!("Реєстрація успішна!");
            Ok(())
        }
        Err(e) => {
            println!("Помилка при реєстрації: {}", e);
            Ok(())
        }
    }
}

pub fn login_user(conn: &Connection) -> Result<Option<i32>> {
    let username = prompt("Введіть ім'я користувача: ")?;
    let password = prompt_password("Введіть пароль: ")?;

    let mut stmt = conn.prepare("SELECT id, password_hash FROM users WHERE username = ?1")
        .context("Підготовка запиту для авторизації користувача не вдалася")?;
    let user_iter = stmt.query_map(params![&username], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    }).context("Виконання запиту для авторизації користувача не вдалося")?;

    for user in user_iter {
        let (id, password_hash) = user.context("Отримання даних користувача не вдалося")?;
        if verify(&password, &password_hash).unwrap_or(false) {
            println!("Вхід успішний!");
            return Ok(Some(id));
        }
    }

    println!("Невірне ім'я користувача або пароль.");
    Ok(None)
}

// Допоміжні функції
fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_password(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let password = read_password().context("Читання пароля не вдалося")?;
    Ok(password.trim().to_string())
}
