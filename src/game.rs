use core::time;
use rand::Rng;
use std::{collections::HashMap, io::Write, sync::mpsc::Receiver, thread};

use crate::{arrow::Arrow, invader::Invader};

pub struct Game {
    screen_rows: u16,
    screen_cols: u16,
    kb_rx: Receiver<usize>,
    input_buf: Vec<u8>,
    input_value: u8,
    current_score: usize,
    // highest_score: usize,

    // from shooter
    arrows: Vec<Arrow>,

    // from invader_army
    hex_values: HashMap<u8, bool>,
    invaders: Vec<Invader>,
    invader_distance: u16,
}

impl Game {
    pub fn new(
        screen_rows: u16,
        screen_cols: u16,
        kb_rx: Receiver<usize>,
        invader_distance: u16,
    ) -> Self {
        Self {
            screen_rows,
            screen_cols,
            kb_rx,
            input_buf: vec![0; 8],
            input_value: 0,
            current_score: 0,
            // highest_score: 0,
            arrows: Vec::new(),

            hex_values: HashMap::new(),
            invaders: Vec::new(),
            invader_distance,
        }
    }

    fn draw_canvas(&self) {
        print!("\x1b[?25l",); // hide the cursor
        print!("\x1b[2J",); // clear the screen
        print!("\x1b[1;1H",); // move cursor to (1,1)

        for row in 1..=self.screen_rows {
            if row == 1 {
                print!("┏{}┓", "━".repeat((self.screen_cols - 2) as usize));
            } else if row == self.screen_rows {
                print!(
                    "┗━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷{}┛",
                    "━".repeat((self.screen_cols - 34) as usize)
                );
            } else if row == self.screen_rows - 2 {
                print!(
                    "┠───┬───┬───┬───┬───┬───┬───┬───┬{}┨",
                    "─".repeat((self.screen_cols - 34) as usize)
                );
            } else if row == self.screen_rows - 3 {
                print!("┃{}┃", "^".repeat((self.screen_cols - 2) as usize));
            } else {
                print!("\x1b[{};1H", row);
                print!("┃");
                print!("\x1b[{};{}H", row, self.screen_cols);
                print!("┃");
            }
        }
    }

    fn update_input(&mut self) {
        match self.kb_rx.try_recv() {
            Ok(key_pressed) => self.input_buf[key_pressed] = self.input_buf[key_pressed] ^ 1,
            Err(_) => (),
        };

        for &bit in self.input_buf.iter() {
            self.input_value = (self.input_value << 1) | bit
        }
    }

    fn draw_scoreboard(&self) {
        print!("\x1b[{};2H", self.screen_rows - 1);

        for i in self.input_buf.iter() {
            print!(" \x1b[1m{}\x1b[0m │", i)
        }

        print!(
            "\x1b[{};{}H",
            self.screen_rows - 2,
            (self.screen_cols / 2) - 2
        );
        print!("┰");
        print!(
            "\x1b[{};{}H",
            self.screen_rows - 2,
            ((self.screen_cols / 2) - 2) + 5
        );
        print!("┰");
        print!("\x1b[{};{}H", self.screen_rows, (self.screen_cols / 2) - 2);
        print!("┻");
        print!(
            "\x1b[{};{}H",
            self.screen_rows,
            ((self.screen_cols / 2) - 2) + 5
        );
        print!("┻");

        print!(
            "\x1b[{};{}H",
            self.screen_rows - 1,
            (self.screen_cols / 2) - 2
        );
        if self.input_value < 0x10 {
            print!("║ \x1b[1m0{:X}\x1b[0m ║", self.input_value)
        } else {
            print!("║ \x1b[1m{:X}\x1b[0m ║", self.input_value)
        }

        print!("\x1b[{};{}H", self.screen_rows - 1, self.screen_cols - 10);
        print!("\x1b[1mScore: {}\x1b[0m", self.current_score);
    }

    fn zero_input(&mut self) {
        self.input_buf = vec![0; 8];
        self.input_value = 0;
    }

    fn game_over(&self) {
        self.draw_canvas();
        self.draw_scoreboard();
        let txt = vec![
            "  _____                         ____                 ",
            " / ____|                       / __ \\                ",
            "| |  __  __ _ _ __ ___   ___  | |  | |_   _____ _ __ ",
            "| | |_ |/ _` | '_ ` _ \\ / _ \\ | |  | \\ \\ / / _ \\ '__|",
            "| |__| | (_| | | | | | |  __/ | |__| |\\ V /  __/ |   ",
            " \\_____|\\__,_|_| |_| |_|\\___|  \\____/  \\_/ \\___|_|   ",
        ];

        let mut c = 0;
        for l in txt {
            print!(
                "\x1b[{};{}H",
                (self.screen_rows / 2) - 4 + c,
                (self.screen_cols / 2) - 25
            );
            print!("{}", l);
            c += 1;
        }

        println!("\x1b[{};{}H", self.screen_rows, self.screen_cols);
    }

    fn flush_stdout(&self) {
        std::io::stdout()
            .flush()
            .expect("Could not flush the stream to stdout.");
    }

    fn arrow_move_forward(&mut self) -> Option<u8> {
        for arrow in self.arrows.iter_mut() {
            if arrow.visible {
                arrow.draw();
                arrow.move_forward();

                if arrow.current_row + 1 == arrow.target.row {
                    arrow.remove();
                    return Some(arrow.target.value);
                }
            }
        }
        None
    }

    /// Checks whether an invader with its `value` equal to `input_value` is on the screen.
    fn to_shoot(&self) -> Option<Invader> {
        for inv in self.invaders.iter() {
            if inv.visible && inv.value == self.input_value {
                return Some(*inv);
            }
        }
        None
    }

    fn shoot(&mut self, target: Invader) {
        let arrow = Arrow::new(self.screen_rows, target);
        self.arrows.push(arrow);
    }

    fn generate_invader(&mut self) {
        loop {
            let mut rng = rand::thread_rng();
            let col: u16 = rng.gen_range(3..=self.screen_cols - 5);
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
                self.invaders.push(invader);
                break;
            }
        }
    }

    /// Returns `false` as soon as an invader can longer move forward, i.e. it has reached the gun line (...`^^^^`...).
    fn invaders_move_forward(&mut self) -> bool {
        if let Some(invader) = self.invaders.last() {
            if invader.row == self.invader_distance + 4 {
                self.generate_invader();
            }
        } else {
            // invaders list is empty
            self.generate_invader();
        }

        for invader in self.invaders.iter_mut() {
            invader.draw();
            invader.move_forward();

            if invader.row > self.screen_rows - 4 {
                // GAME OVER
                return false;
            }
        }
        true
    }

    fn remove_invader(&mut self, val: u8) {
        self.hex_values.insert(val, false);

        let mut index_to_remove = 0;
        for (i, inv) in self.invaders.iter_mut().enumerate() {
            if inv.value == val {
                index_to_remove = i;
                inv.remove();
            }
        }

        self.invaders.remove(index_to_remove);
    }

    pub fn start(&mut self) {
        let mut iter_counter = 0_usize;

        self.draw_canvas();

        loop {
            if iter_counter % 800 == 0 {
                if !self.invaders_move_forward() {
                    self.game_over();
                    self.flush_stdout();
                    panic!("GAME OVER");
                }
            }

            if iter_counter % 25 == 0 {
                if let Some(shot_inv) = self.arrow_move_forward() {
                    // An invader was shot
                    self.remove_invader(shot_inv);
                }
            }

            self.update_input();
            self.draw_scoreboard();

            // Anything to shoot?
            if let Some(invader_to_shoot) = self.to_shoot() {
                self.shoot(invader_to_shoot);
                self.zero_input();

                self.current_score += 1;
            }

            self.flush_stdout();

            // We don't need to worry about overflows here.
            // Since the counter is incremented every 1ms,
            // it would roughly take 585.67 billion years
            // for an overflow to occur!
            iter_counter += 1;

            thread::sleep(time::Duration::from_millis(1));
        }
    }
}
