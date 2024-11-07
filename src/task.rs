// src/task.rs

use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result as AnyResult};
use crate::file_ops::{save_tasks_to_json, load_tasks_from_json};
use crate::utils;
use std::collections::HashMap;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct TaskData {
    pub description: String,
    pub completed: bool,
}

#[derive(Debug)]
pub struct Task {
    pub id: i32,
    pub user_id: i32,
    pub description: String,
    pub completed: bool,
}

pub fn user_menu(conn: &mut Connection, user_id: i32) -> AnyResult<()> {
    loop {
        utils::clear_console(); // Очищення консолі перед виводом меню
        println!("=== Меню користувача ===");
        println!("1. Додати завдання");
        println!("2. Переглянути завдання");
        println!("3. Редагувати завдання");
        println!("4. Видалити завдання");
        println!("5. Позначити завдання як виконане");
        println!("6. Зберегти завдання у JSON");
        println!("7. Завантажити завдання з JSON");
        println!("8. Вийти з облікового запису");
        print!("Виберіть опцію: ");
        std::io::stdout().flush()?;

        let choice = utils::read_input();

        match choice.as_str() {
            "1" => {
                add_task(conn, user_id)?;
                utils::pause();
            }
            "2" => {
                view_tasks(conn, user_id)?;
                utils::pause();
            }
            "3" => {
                edit_task(conn, user_id)?;
                utils::pause();
            }
            "4" => {
                delete_task(conn, user_id)?;
                utils::pause();
            }
            "5" => {
                mark_task_completed(conn, user_id)?;
                utils::pause();
            }
            "6" => {
                save_tasks_to_json(conn, user_id)?;
                utils::pause();
            }
            "7" => {
                load_tasks_from_json(conn, user_id)?;
                utils::pause();
            }
            "8" => {
                println!("Вихід з облікового запису.");
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

pub fn add_task(conn: &Connection, user_id: i32) -> AnyResult<()> {
    let description = prompt("Введіть опис завдання: ")?;

    conn.execute(
        "INSERT INTO tasks (user_id, description, completed) VALUES (?1, ?2, 0)",
        params![user_id, &description],
    ).context("Додавання завдання не вдалося")?;

    println!("Завдання додано.");
    Ok(())
}

pub fn view_tasks(conn: &Connection, user_id: i32) -> AnyResult<()> {
    let mut stmt = conn.prepare(
        "SELECT id, description, completed FROM tasks WHERE user_id = ?1 ORDER BY id",
    ).context("Підготовка запиту для перегляду завдань не вдалася")?;

    let task_iter = stmt.query_map(params![user_id], |row| {
        Ok(Task {
            id: row.get(0)?,
            user_id: user_id,
            description: row.get(1)?,
            completed: row.get(2)?,
        })
    }).context("Виконання запиту для перегляду завдань не вдалося")?;

    let tasks: Vec<Task> = task_iter.collect::<Result<_, _>>()
        .context("Отримання завдань не вдалося")?;

    if tasks.is_empty() {
        println!("\nУ вас немає завдань.\n");
        return Ok(());
    }

    println!("\nВаші завдання:");
    // Створюємо мапінг між порядковим номером та глобальним ID
    let mut task_map = HashMap::new();
    for (index, task) in tasks.iter().enumerate() {
        let display_number = index + 1;
        task_map.insert(display_number, task.id);
        let status = if task.completed { "✅" } else { "❌" };
        println!("{}. [{} ] {}", display_number, status, task.description);
    }
    println!();

    Ok(())
}

pub fn edit_task(conn: &Connection, user_id: i32) -> AnyResult<()> {
    // Спочатку переглядаємо завдання, щоб показати користувачу номери
    view_tasks(conn, user_id)?;

    let task_number_str = prompt("Введіть номер завдання для редагування: ")?;
    let task_number: usize = match task_number_str.parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Невірний номер завдання.");
            return Ok(());
        }
    };

    // Отримуємо глобальний ID завдання за номером
    let task_id = get_task_id_by_number(conn, user_id, task_number)?;
    if task_id.is_none() {
        println!("Завдання з таким номером не існує.");
        return Ok(());
    }
    let task_id = task_id.unwrap();

    let new_description = prompt("Введіть новий опис завдання: ")?;

    let rows_updated = conn.execute(
        "UPDATE tasks SET description = ?1 WHERE id = ?2 AND user_id = ?3",
        params![&new_description, task_id, user_id],
    ).context("Оновлення завдання не вдалося")?;

    if rows_updated > 0 {
        println!("Завдання оновлено.");
    } else {
        println!("Не вдалося оновити завдання.");
    }

    Ok(())
}

pub fn delete_task(conn: &Connection, user_id: i32) -> AnyResult<()> {
    view_tasks(conn, user_id)?;

    let task_number_str = prompt("Введіть номер завдання для видалення: ")?;
    let task_number: usize = match task_number_str.parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Невірний номер завдання.");
            return Ok(());
        }
    };

    let task_id = get_task_id_by_number(conn, user_id, task_number)?;
    if task_id.is_none() {
        println!("Завдання з таким номером не існує.");
        return Ok(());
    }
    let task_id = task_id.unwrap();

    let rows_deleted = conn.execute(
        "DELETE FROM tasks WHERE id = ?1 AND user_id = ?2",
        params![task_id, user_id],
    ).context("Видалення завдання не вдалося")?;

    if rows_deleted > 0 {
        println!("Завдання видалено.");
    } else {
        println!("Завдання не знайдено або не належить вам.");
    }

    Ok(())
}

pub fn mark_task_completed(conn: &Connection, user_id: i32) -> AnyResult<()> {
    view_tasks(conn, user_id)?;

    let task_number_str = prompt("Введіть номер завдання для позначення як виконане: ")?;
    let task_number: usize = match task_number_str.parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Невірний номер завдання.");
            return Ok(());
        }
    };

    let task_id = get_task_id_by_number(conn, user_id, task_number)?;
    if task_id.is_none() {
        println!("Завдання з таким номером не існує.");
        return Ok(());
    }
    let task_id = task_id.unwrap();

    let rows_updated = conn.execute(
        "UPDATE tasks SET completed = 1 WHERE id = ?1 AND user_id = ?2",
        params![task_id, user_id],
    ).context("Позначення завдання як виконане не вдалося")?;

    if rows_updated > 0 {
        println!("Завдання позначено як виконане.");
    } else {
        println!("Завдання не знайдено або не належить вам.");
    }

    Ok(())
}

// Допоміжна функція для мапінгу номеру завдання до його ID
fn get_task_id_by_number(conn: &Connection, user_id: i32, task_number: usize) -> AnyResult<Option<i32>> {
    if task_number == 0 {
        return Ok(None);
    }

    let mut stmt = conn.prepare(
        "SELECT id FROM tasks WHERE user_id = ?1 ORDER BY id LIMIT 1 OFFSET ?2",
    ).context("Підготовка запиту для отримання завдання за номером не вдалася")?;

    let task_id: Option<i32> = stmt.query_row(
        params![user_id, task_number - 1],
        |row| row.get(0),
    ).optional().context("Виконання запиту для отримання завдання за номером не вдалося")?;

    Ok(task_id)
}

// Допоміжна функція
fn prompt(message: &str) -> AnyResult<String> {
    print!("{}", message);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
