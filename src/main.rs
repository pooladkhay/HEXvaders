use core::panic;
use ctrlc;
use std::{
    io::{self, Read},
    os::fd::AsRawFd,
    sync::mpsc::channel,
    thread,
};
use terminal_size::{terminal_size, Height, Width};
use termios::{tcsetattr, Termios};

mod arrow;
mod assets;
mod game;
mod invader;

fn main() {
    let (ctrlc_tx, ctrlc_rx) = channel();
    {
        let ctrlc_tx = ctrlc_tx.clone();
        ctrlc::set_handler(move || {
            ctrlc_tx
                .send(())
                .expect("Could not send signal on ctrlc channel.")
        })
        .expect("Error setting Ctrl-C handler.");
    }

    let (kb_tx, kb_rx) = channel::<usize>();

    thread::spawn(move || {
        let size = terminal_size();
        if let Some((Width(screen_cols), Height(screen_rows))) = size {
            if screen_cols < 80 || screen_rows < 10 {
                panic!("The screen should at least contain 80 columns and 10 rows");
            }
            let mut game = game::Game::new(screen_rows, screen_cols, kb_rx, 2);
            game.start();
        } else {
            panic!("Unable to get terminal size.");
        };
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

                    b'1'..=b'8' => kb_tx
                        .send((raw_key - b'1').into())
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
