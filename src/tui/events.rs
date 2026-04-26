// Key processing result — returned by process_key in mod.rs
pub enum Action {
    None,
    Quit,
    Send(String),
}
