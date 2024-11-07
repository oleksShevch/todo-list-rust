use crossterm::{execute, terminal::{Clear, ClearType}};
use std::io::{self};

// Функція для очищення консолі
pub fn clear_console() {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();
}

// Функція для паузи
pub fn pause() {
    println!("Натисніть Enter, щоб продовжити...");
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy).unwrap();
}

// Функція для читання вводу користувача
pub fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
