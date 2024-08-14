// get the width and height of the text
//                                  (width, height)
pub fn get_text_size(text: &str) -> (usize, usize) {
    let lines = text.lines();
    let mut max_width = 0;
    let mut height = 0;
    for line in lines {
        if line.len() > max_width {
            max_width = line.len();
        }
        height += 1;
    }
    (max_width, height)
}
