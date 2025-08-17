pub struct Cursor(pub String);

impl Cursor {
    pub fn new(token: String) -> Cursor {
        Cursor(token)
    }
}
