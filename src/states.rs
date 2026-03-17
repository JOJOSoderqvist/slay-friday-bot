use strum::Display;

#[derive(Clone, Display)]
pub enum State {
    TriggeredAddCmd,
    PerformAdd { sticker_name: String },
    TriggeredRenameCmd,
    PerformRename { old_name: String },

    TriggerDeleteCmd,
}
