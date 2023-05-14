use crate::InteractionReturn;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    }
};

pub async fn run(options: &[CommandDataOption]) -> InteractionReturn {
    for option in options {
        match option.name.as_str() {
            "module" => {
                match &option.resolved {
                    Some(CommandDataOptionValue::String(input)) => {
                        match input.as_str() {
                            "help" => return InteractionReturn::SilentMessage(help()),
                            "ping" => return InteractionReturn::SilentMessage(String::from("No help for this module.")),
                            "booru" => return InteractionReturn::SilentMessage(crate::commands::booru::help().await),
                            _ => return InteractionReturn::SilentMessage(String::from("Invalid module name."))
                        };
                    }
                    Some(_) => {},
                    None => {},
                }
            },
            _ => return InteractionReturn::SilentMessage(String::from("Invalid argument.")),
        }
    }

    InteractionReturn::Message("What module do you want help with?

modules:
- help
- ping
- booru".to_string())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("help")
        .description("Get help")
        .create_option(|option| {
            option
                .name("module")
                .description("Module for help")
                .kind(CommandOptionType::String)
                //.set_autocomplete(true)
        })
}

pub fn help() -> String {
    String::from("`/help` - Bot help and info

Run `/help module` to get started")
}

#[allow(dead_code)]
pub fn completion(options: &[CommandDataOption]) {
    for option in options {
        match option.name.as_str() {
            "module" => {},
            _ => panic!(),
        }
    }
}
