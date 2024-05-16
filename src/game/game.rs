use core::time;
use rand::Rng;
use std::{collections::HashMap, io::Write, sync::mpsc::Receiver, thread};

use crate::{
    arrow::Arrow,
    assets::{
        BITS_BOTTOM_BORDER, BITS_TOP_BORDER, BIT_SEP_TOP, BOTTOM_BOARD_L, BOTTOM_BOARD_R,
        BOTTOM_L_CORNER, BOTTOM_R_CORNER, GAME_OVER_TEXT, HORIZONTAL_LINE_BOLD,
        HORIZONTAL_LINE_REGULAR, RESTART_MESSAGE, SEP_BOTTOM, SEP_SCORE_DOWN_L, SEP_SCORE_DOWN_R,
        SEP_SCORE_UP_L, SEP_SCORE_UP_R, SEP_TOP, SHOOTER, START_MESSAGE, TOP_L_CORNER,
        TOP_R_CORNER, VERTICAL_LINE_BOLD, VERTICAL_LINE_DOUBLE, VERTICAL_LINE_REGULAR,
    },
    invader::Invader,
};

pub struct Game {
    screen_rows: u16,
    game_cols: u16,
    score_board_cols: u16,
    kb_rx: Receiver<u8>,
    input_buf: Vec<u8>,
    input_value: u8,
    current_score: usize,
    // highest_score: usize,
    arrows: Vec<Arrow>,
    hex_values: HashMap<u8, bool>,
    invaders: Vec<Invader>,
    invader_distance: u16,
}

impl Game {
    pub fn new(
        screen_rows: u16,
        game_cols: u16,
        score_board_cols: u16,
        kb_rx: Receiver<u8>,
        invader_distance: u16,
    ) -> Self {
        Self {
            screen_rows,
            game_cols,
            score_board_cols,
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

    fn clear_data(&mut self) {
        self.input_buf = vec![0; 8];
        self.input_value = 0;
        self.current_score = 0;
        self.arrows = Vec::new();
        self.hex_values = HashMap::new();
        self.invaders = Vec::new();
    }

    fn draw_score_board(&self) {
        print!("\x1b[1;{}H", self.game_cols); // move cursor to (1,self.game_cols)

        for row in 1..=self.screen_rows {
            if row == 1 {
                print!("\x1b[{};{}H", row, self.game_cols);
                print!(
                    "{}{}{}",
                    SEP_TOP,
                    HORIZONTAL_LINE_BOLD.repeat((self.score_board_cols - 1) as usize),
                    TOP_R_CORNER
                );
                self.flush_stdout();
            } else if row == 3 {
                print!("\x1b[{};{}H", row, self.game_cols);
                print!(
                    "{}{}{}",
                    SEP_SCORE_UP_L,
                    HORIZONTAL_LINE_REGULAR.repeat((self.score_board_cols - 1) as usize),
                    SEP_SCORE_UP_R
                );
                self.flush_stdout();
            } else if row == self.screen_rows {
                print!("\x1b[{};{}H", row, self.game_cols);
                print!(
                    "{}{}{}",
                    SEP_BOTTOM,
                    HORIZONTAL_LINE_BOLD.repeat((self.score_board_cols - 1) as usize),
                    BOTTOM_R_CORNER
                );
                self.flush_stdout();
            } else if row == self.screen_rows - 2 {
                print!("\x1b[{};{}H", row, self.game_cols);
                print!(
                    "{}{}{}",
                    SEP_SCORE_DOWN_L,
                    HORIZONTAL_LINE_REGULAR.repeat((self.score_board_cols - 1) as usize),
                    SEP_SCORE_DOWN_R
                );
                self.flush_stdout();
            } else {
                print!("\x1b[{};{}H", row, self.game_cols);
                print!("{}", VERTICAL_LINE_DOUBLE);
                self.flush_stdout();
                print!("\x1b[{};{}H", row, self.game_cols + self.score_board_cols);
                print!("{}", VERTICAL_LINE_BOLD);
                self.flush_stdout();
            }
        }
    }

    fn draw_game(&self) {
        print!("\x1b[1;1H",); // move cursor to (1,1)

        for row in 1..=self.screen_rows {
            if row == 1 {
                print!("\x1b[{};1H", row);
                print!(
                    "{}{}{}",
                    TOP_L_CORNER,
                    HORIZONTAL_LINE_BOLD.repeat((self.game_cols - 2) as usize),
                    TOP_R_CORNER
                );
                self.flush_stdout();
            } else if row == self.screen_rows {
                print!("\x1b[{};1H", row);
                print!(
                    "{}{}{}{}",
                    BOTTOM_L_CORNER,
                    BITS_BOTTOM_BORDER,
                    HORIZONTAL_LINE_BOLD.repeat(
                        ((self.game_cols as usize)
                            - BITS_BOTTOM_BORDER.chars().collect::<Vec<char>>().len()
                            - 2) as usize
                    ),
                    BOTTOM_R_CORNER
                );
                self.flush_stdout();
            } else if row == self.screen_rows - 2 {
                print!("\x1b[{};1H", row);
                print!(
                    "{}{}{}{}",
                    BOTTOM_BOARD_L,
                    BITS_TOP_BORDER,
                    HORIZONTAL_LINE_REGULAR.repeat(
                        ((self.game_cols as usize)
                            - BITS_TOP_BORDER.chars().collect::<Vec<char>>().len()
                            - 2) as usize
                    ),
                    BOTTOM_BOARD_R
                );
                self.flush_stdout();
            } else if row == self.screen_rows - 3 {
                print!("\x1b[{};1H", row);
                print!(
                    "{}{}{}",
                    VERTICAL_LINE_BOLD,
                    SHOOTER.repeat((self.game_cols - 2) as usize),
                    VERTICAL_LINE_BOLD
                );
                self.flush_stdout();
            } else {
                print!("\x1b[{};1H", row);
                print!("{}", VERTICAL_LINE_BOLD);
                self.flush_stdout();
                print!("\x1b[{};{}H", row, self.game_cols);
                print!("{}", VERTICAL_LINE_BOLD);
                self.flush_stdout();
            }
        }
    }

    fn draw_canvas(&self) {
        print!("\x1b[?25l",); // hide the cursor
        print!("\x1b[2J",); // clear the screen

        self.draw_game();
        self.draw_score_board();
    }

    fn print_start_message(&self) {
        print!(
            "\x1b[{};{}H",
            self.screen_rows / 2,
            ((self.game_cols / 2) as usize) - (START_MESSAGE.len() / 2)
        );
        print!("{}", START_MESSAGE);
    }

    fn update_input(&mut self) {
        match self.kb_rx.try_recv() {
            Ok(raw_key) => {
                let key_pressed: usize = (raw_key - b'1').into();
                self.input_buf[key_pressed] = self.input_buf[key_pressed] ^ 1
            }
            Err(_) => (),
        };

        for &bit in self.input_buf.iter() {
            self.input_value = (self.input_value << 1) | bit
        }
    }

    fn draw_bottom_board(&self) {
        print!("\x1b[{};2H", self.screen_rows - 1);

        for i in self.input_buf.iter() {
            print!(" \x1b[1m{}\x1b[0m {}", i, VERTICAL_LINE_REGULAR)
        }

        print!(
            "\x1b[{};{}H",
            self.screen_rows - 2,
            (self.game_cols / 2) - 2
        );
        print!("{}", BIT_SEP_TOP);
        print!(
            "\x1b[{};{}H",
            self.screen_rows - 2,
            ((self.game_cols / 2) - 2) + 5
        );
        print!("{}", BIT_SEP_TOP);
        print!("\x1b[{};{}H", self.screen_rows, (self.game_cols / 2) - 2);
        print!("{}", SEP_BOTTOM);
        print!(
            "\x1b[{};{}H",
            self.screen_rows,
            ((self.game_cols / 2) - 2) + 5
        );
        print!("{}", SEP_BOTTOM);

        print!(
            "\x1b[{};{}H",
            self.screen_rows - 1,
            (self.game_cols / 2) - 2
        );
        if self.input_value < 0x10 {
            print!(
                "{} \x1b[1m0{:X}\x1b[0m {}",
                VERTICAL_LINE_DOUBLE, self.input_value, VERTICAL_LINE_DOUBLE
            )
        } else {
            print!(
                "{} \x1b[1m{:X}\x1b[0m {}",
                VERTICAL_LINE_DOUBLE, self.input_value, VERTICAL_LINE_DOUBLE
            )
        }

        print!("\x1b[{};{}H", self.screen_rows - 1, self.game_cols - 10);
        print!("\x1b[1mScore: {}\x1b[0m", self.current_score);
    }

    fn zero_input(&mut self) {
        self.input_buf = vec![0; 8];
        self.input_value = 0;
    }

    fn game_over(&self) {
        self.draw_canvas();
        self.draw_bottom_board();

        let mut c = 0;
        for l in GAME_OVER_TEXT {
            print!(
                "\x1b[{};{}H",
                (self.screen_rows / 2) - 6 + c,
                (self.game_cols / 2) - 25
            );
            print!("{}", l);
            c += 1;
        }

        print!(
            "\x1b[{};{}H",
            (self.screen_rows / 2) - 4 + c,
            ((self.game_cols / 2) as usize) - (RESTART_MESSAGE.len() / 2)
        );
        print!("{}", RESTART_MESSAGE);
        print!("\x1b[{};{}H", self.screen_rows, self.game_cols);
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
            let col: u16 = rng.gen_range(3..=self.game_cols - 5);
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
        self.draw_bottom_board();
        self.print_start_message();
        self.flush_stdout();

        loop {
            if let Ok(key_pressed) = self.kb_rx.recv() {
                if key_pressed == b' ' {
                    self.draw_canvas();
                    self.flush_stdout();
                } else {
                    continue;
                }
            }

            loop {
                if iter_counter % 650 == 0 {
                    if !self.invaders_move_forward() {
                        // GAME OVER
                        self.game_over();
                        self.clear_data();
                        self.flush_stdout();
                        break;
                    }
                }

                if iter_counter % 25 == 0 {
                    if let Some(shot_inv) = self.arrow_move_forward() {
                        // An invader was shot
                        self.remove_invader(shot_inv);
                    }
                }

                self.update_input();
                self.draw_bottom_board();

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
}
