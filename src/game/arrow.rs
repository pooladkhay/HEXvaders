use crate::invader::Invader;

pub struct Arrow {
    total_rows: u16,
    pub current_row: u16,
    pub visible: bool,
    pub target: Invader,
}

impl Arrow {
    pub fn new(total_rows: u16, target: Invader) -> Self {
        Arrow {
            total_rows,
            current_row: total_rows - 3,
            visible: true,
            target,
        }
    }

    pub fn draw(&mut self) {
        print!("\x1b[{};{}H", self.current_row, self.target.col + 1);
        print!("↑↑");

        if self.current_row < self.total_rows - 4 {
            print!("\x1b[{};{}H", self.current_row + 1, self.target.col + 1);
            print!("  ");
        }
        if self.current_row == self.total_rows - 3 {
            print!("\x1b[{};{}H", self.current_row, self.target.col + 1);
            print!("^^");
        }
    }

    pub fn remove(&mut self) {
        self.visible = false;
        print!("\x1b[{};{}H", self.current_row, self.target.col + 1);
        print!("  ");
        // print!("\x1b[{};{}H", self.current_row + 1, self.target.col + 1);
        // print!("  ");
    }

    pub fn move_forward(&mut self) {
        self.current_row -= 1;
    }
}
