use core::panic;
use ctrlc;
use std::{
    io::{self, Read},
    os::fd::AsRawFd,
    sync::mpsc::{channel, Receiver},
    thread,
};
use terminal_size::{terminal_size, Height, Width};
use termios::{tcsetattr, Termios};

mod arrow;
mod assets;
mod game;
mod invader;
mod score_board;

pub struct InitData {
    is_multiplayer: bool,
    game_rows: u16,
    game_cols: u16,
    score_board_cols: u16,
    kb_rx: Receiver<u8>,
    invader_distance: u16,
}

fn main() {
    let (ctrlc_tx, ctrlc_rx) = channel();
    let (kb_tx, kb_rx) = channel::<u8>();

    let mut init_data = InitData {
        is_multiplayer: true,
        game_rows: 0,
        game_cols: 0,
        score_board_cols: 0,
        kb_rx,
        invader_distance: 2,
    };

    let term_size = terminal_size();
    if let Some((Width(screen_cols), Height(screen_rows))) = term_size {
        // game takes ~ 70% of the screen,
        // score board ~ 30% of the screen.
        if init_data.is_multiplayer {
            init_data.game_cols = (screen_cols * 70) / 100;
            init_data.score_board_cols = screen_cols - init_data.game_cols;
        } else {
            init_data.game_cols = screen_cols;
        }
        init_data.game_rows = screen_rows
    } else {
        panic!("Unable to get terminal size.");
    }

    {
        let ctrlc_tx = ctrlc_tx.clone();
        ctrlc::set_handler(move || {
            ctrlc_tx
                .send(())
                .expect("Could not send signal on ctrlc channel.")
        })
        .expect("Error setting Ctrl-C handler.");
    }

    thread::spawn(move || {
        if init_data.game_cols < 80 || init_data.game_rows < 10 {
            panic!("The screen should at least contain 80 columns and 10 rows");
        }
        let mut game = game::Game::new(init_data);
        game.start();
    });

    thread::spawn(move || {
        let stdin_fd = io::stdin().as_raw_fd();
        let mut termios = Termios::from_fd(stdin_fd).expect("Termios failed.");

        termios.c_lflag &= !termios::ECHO;
        termios.c_lflag &= !termios::ICANON;

        tcsetattr(stdin_fd, termios::TCSANOW, &mut termios).expect("tcsetattr failed.");

        for raw_key in io::stdin().bytes() {
            if let Ok(raw_key) = raw_key {
                match raw_key {
                    b'q' | b'Q' => ctrlc_tx
                        .send(())
                        .expect("Could not send signal on ctrlc channel."),

                    b'1'..=b'8' | b' ' => kb_tx
                        .send(raw_key)
                        .expect("Could not send signal on kb channel."),

                    _ => (),
                }
            }
        }
    });

    ctrlc_rx
        .recv()
        .expect("Could not receive from ctrlc channel.");
    println!("\x1b[?25h\n"); // unhide the cursor
}
