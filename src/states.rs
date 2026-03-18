use strum::Display;

#[derive(Clone, Display)]
pub enum State {
    TriggeredAddCmd,
    PerformAdd { media_entry_name: String },
    TriggeredRenameCmd,
    PerformRename { old_name: String },

    TriggerDeleteCmd,
}
