use std::io::{stdout, Write};
use tokio::time::{sleep, Duration};

use crate::assets::{INVADER_BOTTOM_BORDER, INVADER_SIDE_BORDER, INVADER_TOP_BORDER};

#[derive(Clone, Copy)]
pub struct Invader {
    pub value: u8,
    pub row: u16,
    pub col: u16,
    pub visible: bool,
}

impl Invader {
    pub fn new(row: u16, col: u16, value: u8) -> Self {
        Self {
            value,
            row,
            col,
            visible: true,
        }
    }

    pub fn draw(&self) {
        if self.visible {
            if self.row > 1 {
                print!("\x1b[{};{}H", self.row - 2, self.col);
                print!("    ");
            }
            if self.row == 3 {
                print!("\x1b[{};{}H", 1, self.col);
                print!("━━━━");
            }
            print!("\x1b[{};{}H", self.row - 1, self.col);
            print!("{}", INVADER_TOP_BORDER);
            print!("\x1b[{};{}H", self.row, self.col);
            if self.value < 0x10 {
                print!(
                    "\x1b[1m{}0{:X}{}\x1b[0m",
                    INVADER_SIDE_BORDER, self.value, INVADER_SIDE_BORDER
                )
            } else {
                print!(
                    "\x1b[1m{}{:X}{}\x1b[0m",
                    INVADER_SIDE_BORDER, self.value, INVADER_SIDE_BORDER
                )
            }
            print!("\x1b[{};{}H", self.row + 1, self.col);
            print!("{}", INVADER_BOTTOM_BORDER);
        }
    }

    pub fn move_forward(&mut self) {
        if self.visible {
            self.row += 1;
        }
    }

    pub async fn remove(&mut self) {
        self.visible = false;
        print!("\x1b[{};{}H", self.row - 2, self.col);
        print!("    ");
        print!("\x1b[{};{}H", self.row - 1, self.col);
        print!(" ╳╳ ");
        print!("\x1b[{};{}H", self.row, self.col);
        print!("    ");
        self.flush_stdout_with_sleep().await;

        print!("\x1b[{};{}H", self.row - 2, self.col - 1);
        print!("╲    ╱");
        print!("\x1b[{};{}H", self.row - 1, self.col);
        print!("    ");
        print!("\x1b[{};{}H", self.row, self.col - 1);
        print!("╱    ╲");
        self.flush_stdout_with_sleep().await;

        print!("\x1b[{};{}H", self.row - 2, self.col - 1);
        print!("      ");
        print!("\x1b[{};{}H", self.row - 1, self.col);
        print!("    ");
        print!("\x1b[{};{}H", self.row, self.col - 1);
        print!("      ");
    }

    async fn flush_stdout_with_sleep(&self) {
        stdout()
            .flush()
            .expect("Could not flush the stream to stdout.");
        sleep(Duration::from_millis(50)).await;
    }
}
