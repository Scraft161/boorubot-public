#[macro_use]
extern crate lazy_static;

mod commands;
mod config;

use crate::config::{Config, BooruConfig};

use std::env;

use serenity::async_trait;
use serenity::{
    model::{
        application::{
            command::Command,
            interaction::Interaction
        },
        channel::Channel,
        gateway::Ready,
    },
    builder::CreateEmbed,
    prelude::*,
};

lazy_static!{
    static ref CONFIG: RwLock<Config> = RwLock::new(Config {
        token: None,
        booru_config: BooruConfig {
            blacklist: Vec::new(),
            force_block: Vec::new(),
            nsfw_tags: Vec::new(),
        }
    });
}

pub enum InteractionReturn {
    Message(String),
    SilentMessage(String),
    Embed(CreateEmbed),
    Raw(),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = &interaction {
            // Give discord a heads up that we're working so it doesn't time out the interaction.
            match command.defer(&ctx.http).await {
                Ok(_) => (),
                Err(_) => println!("[WARN]: Could not defer interaction."),
            }

            let interaction_channel = command.channel_id.to_channel(&ctx).await.unwrap();
            //dbg!(&interaction_channel);

            let is_channel_nsfw = interaction_channel.is_nsfw();

            let interaction_channel_info = match interaction_channel {
                Channel::Guild(data) => data,
                _ => {dbg!(interaction_channel); todo!()},
            };

            //dbg!(&interaction_channel_info);

            let enable_ecchi = if interaction_channel_info.name().contains("ecchi") {
                true
            //} else if interaction_channel_info.topic.unwrap().contains("boorubot: ecchi") {
            } else {
                match interaction_channel_info.topic {
                    Some(value) => {
                        value.contains("boorubot: ecchi")                    },
                    None => false,
                }
            };

            let content = match command.data.name.as_str() {
                "help" => commands::help::run(&command.data.options).await,
                "ping" => commands::ping::run(&command.data.options),
                "booru" => commands::booru::run(&command.data.options, is_channel_nsfw, enable_ecchi).await,
                _ => InteractionReturn::Message("not implemented :(".to_string()),
            };

            if let Err(why) = command
                .create_followup_message(&ctx.http, |response| {
                    match content {
                        InteractionReturn::Message(data) => response
                            .content(data),
                        InteractionReturn::SilentMessage(data) => response
                            .ephemeral(true)
                            .content(data),
                        _ => todo!("Return types raw and embed are not supported for follow up yet"),
                    }
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why)
            }

            // Old return method
            /*
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    //response
                    //    .kind(InteractionResponseType::ChannelMessageWithSource)
                    //    .interaction_response_data(|message| message.content(content))

                    match content {
                        InteractionReturn::Message(data) => response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(data)),
                        InteractionReturn::SilentMessage(data) => response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(data).ephemeral(true)),
                        InteractionReturn::Embed(data) => response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.set_embed(data)),
                        _ => todo!(),
                    }
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
            */
        }

        /*
        if let Interaction::Autocomplete(completion) = interaction {
            dbg!(&completion);

            let content = match completion.data.name.as_str() {
                "help" => commands::help::completion(&completion.data.options),
                _ => panic!(),
            };

            dbg!(&content);
        }
        */
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("[INFO]: Connected as {}", ready.user.name);

        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::help::register(command))
                .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::booru::register(command))
        })
        .await;

        let mut registered_commands = Vec::new();

        match commands {
            Ok(value) => {
                registered_commands = value;
                println!("Commands:\n  Name, Description, Options")
            },
            Err(why) => println!("[WARN]: No commands registered: {}", why),
        }

        for command in registered_commands {
            println!("- {}, {}, options:", command.name, command.description);

            for option in command.options {
                println!("  + {}, {}, {:#?}", option.name, option.description, option.kind);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Read config
    let config = config::read();

    // Load .env
    match dotenv::dotenv() {
        Ok(_) => println!("[INFO]: Loaded environment from `.env`"),
        Err(why) => println!("[WARN]: Couldn't load `.env`: {}", why),
    }
    //let token: String = env::var("DISCORD_TOKEN").expect("[ERR]: Expected a token in the environment");

    let token = match env::var("DISCORD_TOKEN") {
        Ok(value) => value,
        Err(_) => config.token.as_ref().unwrap().to_string(),
    };

    // Write the config
    {
        let mut w = CONFIG.write().await;
        *w = config;
    }

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("[ERR]: Error creating client");

    if let Err(why) = client.start().await {
        println!("[ERR]: Client error: {:?}", why);
    }
}
