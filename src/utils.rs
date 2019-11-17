use std::collections::HashMap;
use crate::games_model::Category;
use serenity::utils::MessageBuilder;
use chrono::Local;

pub fn format_post(categories: &HashMap<String, Category>) -> String {
    if categories.is_empty() {
        return String::from("Please add a category.");
    }

    let mut mb = MessageBuilder::new();

    categories.values().for_each(|cat| {
        mb.push_bold_line(cat.name());

        let games = cat.games();
        if games.is_empty() {
            mb.push("Please add a game.");
        } else {
            games.values().for_each(|game| {
                mb.push("<:")
                    .push(&game.emoji().name)
                    .push(":")
                    .push(&game.emoji().id)
                    .push(">");

                mb.push(" - ")
                    .push(game.name())
                    .push("\n");
            });
        }

        mb.push("\n");
    });

    mb.push("(Last updated on ")
        .push(Local::now().to_rfc2822())
        .push(")")
        .build()
}