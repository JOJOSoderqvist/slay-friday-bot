use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Поддерживаются следующие команды:"
)]
#[derive(Debug)]
pub enum Command {
    #[command(description = "Показать это сообщение.")]
    Help,
    #[command(description = "Показать, сколько осталось до нефорской пятницы.")]
    Friday,
    #[command(description = "Показать, какая модель сгенерировала сообщние (из последних 10)")]
    Model,
    #[command(description = "Отправить стикер с определенным названием.\nНапример, /sticker xdd или /get xdd",
    aliases = ["get"])]
    Sticker(String),
    #[command(rename = "list_stickers", description = "Показать доступные стикеры", aliases = ["list"])]
    ListStickers,

    #[command(rename="add_sticker", description = "Добавляет новый стикер.\nНапример, /add_sticker xdd или /add xdd.",
    aliases = ["add"])]
    AddSticker(String),

    #[command(description = "Отмена операции в рамках диалога")]
    Cancel,
}
