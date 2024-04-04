use ctrlc;
use std::{sync::mpsc::channel, thread};

mod invader;
mod scene;

fn main() {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    thread::spawn(move || {
        let mut scene = scene::Scene::new();
        scene.start(2);
    });

    rx.recv().expect("Could not receive from channel.");
    println!("\x1b[?25h",); // unhide the cursor
}
// ↑
