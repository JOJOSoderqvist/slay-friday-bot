#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveSticker {
        name: String,
    },
    ReceiveNewName {
        old_name: String,
    },
}
