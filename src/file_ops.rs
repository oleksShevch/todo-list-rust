use rusqlite::{Connection, params};
use anyhow::{Context, Result as AnyResult};
use std::fs::File;
use std::path::Path;
use std::io::{Write};
use crate::task::{Task, TaskData};

pub fn save_tasks_to_json(conn: &mut Connection, user_id: i32) -> AnyResult<()> {
    let tasks = get_tasks(conn, user_id)?;

    // Перетворюємо Task на TaskData для серіалізації
    let task_data: Vec<TaskData> = tasks.into_iter()
        .map(|task| TaskData {
            description: task.description,
            completed: task.completed,
        })
        .collect();

    let json = serde_json::to_string_pretty(&task_data)
        .context("Серіалізація завдань у JSON не вдалася")?;

    let filename = prompt("Введіть ім'я файлу для збереження (наприклад, tasks.json): ")?;

    match File::create(&filename) {
        Ok(mut file) => {
            file.write_all(json.as_bytes())
                .context("Запис JSON у файл не вдалося")?;
            println!("Завдання успішно збережено у файл '{}'.", filename);
        }
        Err(e) => {
            println!("Помилка при створенні файлу: {}", e);
        }
    }

    Ok(())
}

pub fn load_tasks_from_json(conn: &mut Connection, user_id: i32) -> AnyResult<()> {
    let filename = prompt("Введіть ім'я файлу для завантаження (наприклад, tasks.json): ")?;

    if !Path::new(&filename).exists() {
        println!("Файл '{}' не знайдено.", filename);
        return Ok(());
    }

    let file = File::open(&filename)
        .context("Відкриття файлу не вдалося")?;
    let tasks: Vec<TaskData> = serde_json::from_reader(file)
        .context("Десеріалізація JSON не вдалася")?;

    let tx = conn.transaction()
        .context("Створення транзакції не вдалося")?;

    for task in tasks {
        // Вставляємо нове завдання з поточним user_id
        tx.execute(
            "INSERT INTO tasks (user_id, description, completed) VALUES (?1, ?2, ?3)",
            params![user_id, &task.description, task.completed as i32],
        ).context("Вставка завдання з JSON не вдалася")?;
    }

    tx.commit()
        .context("Коміт транзакції не вдалося")?;

    println!("Завдання успішно завантажено з файлу '{}'.", filename);
    Ok(())
}

fn get_tasks(conn: &Connection, user_id: i32) -> AnyResult<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, description, completed FROM tasks WHERE user_id = ?1 ORDER BY id",
    ).context("Підготовка запиту для отримання завдань не вдалася")?;

    let task_iter = stmt.query_map(params![user_id], |row| {
        Ok(Task {
            id: row.get(0)?,
            user_id: row.get(1)?,
            description: row.get(2)?,
            completed: row.get(3)?,
        })
    }).context("Виконання запиту для отримання завдань не вдалося")?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task.context("Отримання завдання не вдалося")?);
    }

    Ok(tasks)
}

fn prompt(message: &str) -> AnyResult<String> {
    print!("{}", message);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
