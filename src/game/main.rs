use std::os::fd::{AsFd, AsRawFd};
use terminal_size::{terminal_size, Height, Width};
use termios::{tcsetattr, Termios};
use tokio::{
    io::{self, AsyncReadExt, BufReader},
    signal,
    sync::mpsc::UnboundedReceiver,
};

mod arrow;
mod assets;
mod game;
mod invader;
mod score_board;

// pub struct Window {
//     rows: u16,
//     cols: u16,
// }

pub struct GameData {
    is_multiplayer: bool,
    game_rows: u16,
    game_cols: u16,
    score_board_cols: u16,
    kb_rx: UnboundedReceiver<u8>,
    invader_distance: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Event channel
    let (kb_tx, kb_rx) = tokio::sync::mpsc::unbounded_channel::<u8>();
    let (ctrlc_tx, mut ctrlc_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    // Configuring Terminal
    let stdin_fd = io::stdin().as_fd().as_raw_fd();
    let mut termios = Termios::from_fd(stdin_fd)?;
    termios.c_lflag &= !termios::ECHO;
    termios.c_lflag &= !termios::ICANON;
    tcsetattr(stdin_fd, termios::TCSANOW, &mut termios).expect("tcsetattr failed.");

    // Getting
    let term_size = terminal_size();
    let (Width(screen_cols), Height(screen_rows)) =
        term_size.expect("Unable to get terminal size.");

    //
    let mut game_data = GameData {
        is_multiplayer: true,
        game_rows: 0,
        game_cols: 0,
        score_board_cols: 0,
        kb_rx,
        invader_distance: 2,
    };

    // game takes ~ 70% of the screen,
    // score board ~ 30% of the screen.
    if game_data.is_multiplayer {
        game_data.game_cols = (screen_cols * 70) / 100;
        game_data.score_board_cols = screen_cols - game_data.game_cols;
    } else {
        game_data.game_cols = screen_cols;
    }
    game_data.game_rows = screen_rows;

    tokio::spawn(async move {
        if game_data.game_cols < 80 || game_data.game_rows < 10 {
            panic!("The screen should at least contain 80 columns and 10 rows");
        }
        let mut game = game::Game::new(game_data);
        game.start().await;
    });

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);

        loop {
            let raw_key = reader.read_u8().await.expect("failed to read u8");
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
    });

    tokio::select! {
        _ = signal::ctrl_c() => shutdown(screen_rows),
        _ = ctrlc_rx.recv()=> shutdown(screen_rows),
    }

    // match signal::ctrl_c().await {
    //     Ok(()) => shutdown(screen_rows),
    //     Err(err) => {
    //         eprintln!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     }
    // }

    Ok(())
}

fn shutdown(screen_rows: u16) {
    print!("\x1b[?25h"); // unhide the cursor
    print!("\x1b[{};1H", screen_rows);
    println!("\n\nPress Enter to exit.");
}
