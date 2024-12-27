use std::io::Read as _;

struct Callbacks;
impl vt100_yh::Callbacks for Callbacks {}

fn read_frames() -> impl Iterator<Item = Vec<u8>> {
    (1..=7625).map(|i| {
        let mut file =
            std::fs::File::open(format!("tests/data/crawl/crawl{i}"))
                .unwrap();
        let mut frame = vec![];
        file.read_to_end(&mut frame).unwrap();
        frame
    })
}

fn process_frames(frames: &[Vec<u8>]) {
    let mut parser = vt100_yh::Parser::default();
    for frame in frames {
        parser.process_cb(frame, &mut Callbacks);
    }
}

fn main() {
    let frames: Vec<Vec<u8>> = read_frames().collect();
    let start = std::time::Instant::now();
    let mut i = 0;
    loop {
        i += 1;
        process_frames(&frames);
        if (std::time::Instant::now() - start).as_secs() >= 30 {
            break;
        }
    }
    eprintln!("{i} iterations");
}
