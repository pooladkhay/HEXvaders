use core::time;
use rand::Rng;
use std::{collections::VecDeque, io::Write, thread};
use terminal_size::{terminal_size, Height, Width};

use crate::invader::Invader;

pub struct Scene {
    invaders: VecDeque<Invader>,
    rows: u16,
    cols: u16,
}

impl Scene {
    pub fn new() -> Self {
        let size = terminal_size();
        if let Some((Width(w), Height(h))) = size {
            return Self {
                invaders: VecDeque::new(),
                rows: h,
                cols: w,
            };
        } else {
            panic!("Unable to get terminal size");
        };
    }

    fn gen_invader(&mut self) {
        loop {
            let mut rng = rand::thread_rng();
            let col: u16 = rng.gen_range(3..=self.cols - 5);
            let val: u8 = rng.gen_range(1..=0xFF);

            let mut val_unique = true;
            // let mut col_unique = true;
            // let mut enough_distance = true;

            for invader in &self.invaders {
                if !invader.visible {
                    continue;
                }
                if invader.value == val {
                    val_unique = false
                }
                // if invader.col == col {
                //     col_unique = false;
                // }
                // if invader.col.abs_diff(col) <= 1 {
                //     enough_distance = false
                // }
            }

            if val_unique
            /* && col_unique && enough_distance */
            {
                let invader = Invader::new(1, col, val);
                self.invaders.push_front(invader);
                break;
            }
        }
    }

    fn proceed(&mut self) {
        let mut to_deq = 0;
        for invader in self.invaders.iter_mut() {
            invader.proceed();
            if invader.row >= self.rows {
                to_deq += 1;
                // Not really necessary
                invader.visible = false
            }
        }
        for _ in 0..to_deq {
            self.invaders.pop_back();
        }
    }

    pub fn start(&mut self) {
        self.gen_invader();

        print!("\x1b[?25l",); // hide the cursor

        let threshold: u8 = 3;
        let mut counter: u8 = 0;

        loop {
            print!("\x1b[3J",);
            print!("\x1b[2J",);
            print!("\x1b[1;1H",);

            for row in 1..=self.rows {
                if row == 1 {
                    print!("┌{}┐", "─".repeat((self.cols - 2) as usize));
                } else if row == self.rows {
                    print!("└{}┘", "─".repeat((self.cols - 2) as usize));
                } else {
                    print!("\x1b[{};1H", row);
                    print!("│");

                    for invader in &self.invaders {
                        invader.draw();
                    }

                    print!("\x1b[{};{}H", row, self.cols);
                    print!("│");
                }
            }

            self.proceed();

            counter += 1;
            if counter == threshold {
                self.gen_invader();
                counter = 0;
            }

            std::io::stdout()
                .flush()
                .expect("Could not flush the stream to stdout.");
            thread::sleep(time::Duration::from_millis(500));
        }
    }
}
