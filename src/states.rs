#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveSticker {
        name: String,
    },
}
