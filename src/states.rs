use teloxide::types::{Message, MessageId, UserId};

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
        user_id: UserId,
        original_msg: Message,
    },
    ShowOutline {
        user_id: UserId,
    },
}
