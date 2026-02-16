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
    #[command(description = "Показать, какая модель сгенерировала сообщние (из последних 20)")]
    Model,
    #[command(description = "Отправить стикер с определенным названием.\nНапример, /sticker xdd или /get xdd",
    aliases = ["get"])]
    Sticker(String),
    #[command(rename = "list_stickers", description = "Показать доступные стикеры", aliases = ["list"])]
    ListStickers,

    #[command(rename="add_sticker", description = "Добавляет новый стикер.\nНапример, /add_sticker xdd или /add xdd.",
    aliases = ["add"])]
    AddSticker(String),

    #[command(rename="rename_sticker", description = "Переименовывает существующий стикер.\nНапример, /rename_sticker xdd или /rename xdd.",
    aliases = ["rename"])]
    RenameSticker(String),

    #[command(rename="delete_sticker", description = "Удаляет существующий стикер.\nНапример, /delete_sticker xdd или /delete xdd.",
    aliases = ["delete", "remove_sticker", "remove"])]
    DeleteSticker(String),

    #[command(description = "Отмена операции в рамках диалога")]
    Cancel,
}
