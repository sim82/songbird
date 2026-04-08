use songbird::Engine;

fn main() {
    // Create a new synthesis engine at 44.1 kHz
    let _engine = Engine::new(44100);

    println!("Songbird Ambient Sound Synthesis Engine");
    println!("Framework initialized successfully!");
}
