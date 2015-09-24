extern crate rodio;

use std::io::BufReader;

fn main() {
    let endpoint = rodio::get_default_endpoint().unwrap();

    let file = std::fs::File::open("examples/beep.wav").unwrap();
    let mut beep1 = rodio::play_once(&endpoint, BufReader::new(file));
    beep1.set_volume(0.2);

    std::thread::sleep_ms(1000);

    let file = std::fs::File::open("examples/beep2.wav").unwrap();
    rodio::play_once(&endpoint, BufReader::new(file));

    std::thread::sleep_ms(1000);
    let file = std::fs::File::open("examples/beep3.ogg").unwrap();
    rodio::play_once(&endpoint, file);

    std::thread::sleep_ms(1000);
    beep1.stop();

    std::thread::sleep_ms(7000);
}