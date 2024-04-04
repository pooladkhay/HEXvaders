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
            print!("\x1b[{};{}H", self.row - 1, self.col);
            print!("┌──┐");
            print!("\x1b[{};{}H", self.row, self.col);
            if self.value < 0x10 {
                print!("\x1b[1m│0{:X}│\x1b[0m", self.value)
            } else {
                print!("\x1b[1m│{:X}│\x1b[0m", self.value)
            }
            print!("\x1b[{};{}H", self.row + 1, self.col);
            print!("└──┘");
        }
    }
    pub fn proceed(&mut self) {
        self.row += 1;
    }
}
