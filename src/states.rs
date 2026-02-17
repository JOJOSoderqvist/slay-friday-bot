use teloxide::types::UserId;

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
    ShowInline {
        user_id: UserId
    },
    ShowOutline {
        user_id: UserId
    },
}
