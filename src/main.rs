mod db;
mod user;
mod task;
mod file_ops;
mod utils;

use std::io::Write;
use db::initialize_db;
use user::{register_user, login_user};
use task::user_menu;
use rusqlite::Connection;
use anyhow::Result;

fn main() -> Result<()> {
    let mut conn = Connection::open("todo_app.db")?;
    initialize_db(&mut conn)?;

    loop {
        utils::clear_console(); // Очищення консолі перед виводом головного меню
        println!("=== ToDo App ===");
        println!("1. Реєстрація");
        println!("2. Вхід");
        println!("3. Вихід");
        print!("Виберіть опцію: ");
        std::io::stdout().flush()?;

        let choice = utils::read_input();

        match choice.as_str() {
            "1" => {
                utils::clear_console();
                if let Err(e) = register_user(&conn) {
                    println!("Помилка при реєстрації: {}", e);
                }
                utils::pause();
            }
            "2" => {
                utils::clear_console();
                match login_user(&mut conn)? {
                    Some(user_id) => {
                        user_menu(&mut conn, user_id)?;
                    }
                    None => {
                        println!("Невірне ім'я користувача або пароль.");
                        utils::pause();
                    }
                }
            }
            "3" => {
                println!("До побачення!");
                break;
            }
            _ => {
                println!("Невідома опція. Спробуйте ще раз.");
                utils::pause();
            }
        }
    }

    Ok(())
}
