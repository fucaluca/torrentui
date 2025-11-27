#[derive(Default)]
pub struct App {
    should_quit: bool,
}
impl App {
    pub fn new() -> Self {
        Self::default()
    }
}
