use core::time;
use rand::Rng;
use std::{
    collections::{HashMap, VecDeque},
    io::Write,
    thread,
};
use terminal_size::{terminal_size, Height, Width};

use crate::invader::Invader;

pub struct Scene {
    hex_values: HashMap<u8, bool>,
    invaders: VecDeque<Invader>,
    rows: u16,
    cols: u16,
}

impl Scene {
    pub fn new() -> Self {
        let size = terminal_size();
        if let Some((Width(w), Height(h))) = size {
            return Self {
                hex_values: HashMap::new(),
                invaders: VecDeque::new(),
                rows: h,
                cols: w,
            };
        } else {
            panic!("Unable to get terminal size");
        };
    }

    fn generate_invader(&mut self) {
        loop {
            let mut rng = rand::thread_rng();
            let col: u16 = rng.gen_range(3..=self.cols - 5);
            let val: u8 = rng.gen_range(1..=0xFF);

            let val_is_unique = (self.hex_values.contains_key(&val)
                && !self
                    .hex_values
                    .get(&val)
                    .expect("hex_values does not contain the specified key."))
                || (!self.hex_values.contains_key(&val));

            if val_is_unique {
                self.hex_values.insert(val, true);

                let invader = Invader::new(1, col, val);
                self.invaders.push_front(invader);
                break;
            }
        }
    }

    fn draw_invaders(&self) {
        for invader in &self.invaders {
            invader.draw();
        }
    }

    fn march(&mut self, with_distance: u16) {
        if let Some(invader) = self.invaders.front() {
            if invader.row == with_distance + 4 {
                self.generate_invader();
            }
        } else {
            // invaders list is empty
            self.generate_invader();
        }

        let mut deq = false;

        for invader in self.invaders.iter_mut() {
            invader.move_forward();

            if invader.row > self.rows {
                invader.remove();
                self.hex_values.insert(invader.value, false);
                deq = true;
            }
        }

        if deq {
            self.invaders.pop_back();
        }
    }

    fn draw_canvas(&self) {
        print!("\x1b[?25l",); // hide the cursor
        print!("\x1b[2J",); // clear the screen
        print!("\x1b[1;1H",); // move cursor to (1,1)

        for row in 1..=self.rows {
            if row == 1 {
                print!("┌{}┐", "─".repeat((self.cols - 2) as usize));
            } else if row == self.rows {
                print!("└{}┘", "─".repeat((self.cols - 2) as usize));
            } else {
                print!("\x1b[{};1H", row);
                print!("│");
                print!("\x1b[{};{}H", row, self.cols);
                print!("│");
            }
        }
    }

    pub fn start(&mut self, march_distance: u16) {
        self.draw_canvas();

        // main loop
        loop {
            self.draw_invaders();

            self.march(march_distance);

            std::io::stdout()
                .flush()
                .expect("Could not flush the stream to stdout.");
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
