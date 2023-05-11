#[macro_use]
extern crate lazy_static;

mod commands;

use std::env;

use serenity::async_trait;
use serenity::{
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType}
        },
        channel::Channel,
        gateway::Ready,
    },
    builder::CreateEmbed,
    prelude::*,
};

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

        if let Interaction::ApplicationCommand(command) = interaction {
            // Some commands may take longer to process so let's give discord a heads up.
            //command.create_interaction_response(ctx, |b|
            //    b.interaction_response_type(InteractionResponseType::DeferredChannelMessageWithSource)).await.unwrap();

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
                        if value.contains("boorubot: ecchi") {
                            true
                        } else {
                            false
                        }
                    },
                    None => false,
                }
            };

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                "booru" => commands::booru::run(&command.data.options, is_channel_nsfw, enable_ecchi).await,
                _ => InteractionReturn::Message("not implemented :(".to_string()),
            };

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
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("[INFO]: Connected as {}", ready.user.name);

        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
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
    // Load .env
    match dotenv::dotenv() {
        Ok(_) => println!("[INFO]: Loaded environment from `.env`"),
        Err(why) => println!("[WARN]: Couldn't load `.env`: {}", why),
    }
    let token = env::var("DISCORD_TOKEN").expect("[ERR]: Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("[ERR]: Error creating client");

    if let Err(why) = client.start().await {
        println!("[ERR]: Client error: {:?}", why);
    }
}
