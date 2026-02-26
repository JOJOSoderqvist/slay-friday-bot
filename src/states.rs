use strum::Display;
use teloxide::types::{Message, UserId};

#[derive(Clone, Display)]
pub enum State {
    TriggeredAddCmd,
    PerformAdd {
        sticker_name: String,
    },
    TriggeredRenameCmd,
    PerformRename {
        old_name: String,
    },

    TriggerDeleteCmd,

    ShowInline {
        user_id: UserId,
        original_msg: Box<Message>,
    },
}
