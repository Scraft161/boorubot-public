use crate::InteractionReturn;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    }
};
use booru_rs::client::gelbooru::GelbooruClient;
use booru_rs::client::generic::{BooruClient, BooruBuilder};

lazy_static! {
    /// A set of tags that will help aleviate Discord ToS violating content; we rely on gelbooru's
    /// tagging for this so it may not be 100% accurate.
    static ref DISCORD_GLOBAL_EXCLUDE: String = String::from("-loli -shota -gore");

    /// A set of tags that will help exclude NSFW content whet it is not allowed.
    static ref EXCLUDE_NSFW: String = String::from("-nude -completely_nude -rating:questionable -rating:explicit");

    /// A list of tags that should never be used in searches.
    /// This is primarily a mirror of DISCORD_GLOBAL_EXCLUDE; but may also include additional tags.
    static ref BLACKLISTED_TAGS: Vec<&'static str> = vec!["loli", "shota", "gore", "guro"];
}

pub async fn run(options: &[CommandDataOption], allow_nsfw: bool, allow_ecchi: bool) -> InteractionReturn {

    let mut user_tags = String::new();
    let mut user_exclude = String::new();
    let mut user_rating = String::new();
    let mut user_count: i64 = 1;

    for option in options {
        match option.name.as_str() {
            "tags" => {
                match &option.resolved {
                    Some(CommandDataOptionValue::String(input)) => {
                        let mut illegal_tags: Vec<&str> = Vec::new();

                        for tag in BLACKLISTED_TAGS.iter() {
                            if input.contains(tag) {
                                illegal_tags.push(tag)
                            }
                        }

                        if !illegal_tags.is_empty() {
                            return InteractionReturn::SilentMessage(format!("Your search includes tags that would violate discord's ToS: `{}`", illegal_tags.join("` `")));
                        }

                        user_tags = input.to_string()
                    },
                    Some(_) => return InteractionReturn::SilentMessage("Couldn't parse value in option `tags`, is it valid?".to_string()),
                    None => (),
                }
            },
            "exclude" => {
                match &option.resolved {
                    Some(CommandDataOptionValue::String(input)) => {
                        for item in input.to_string().split(' ') {
                            user_exclude += &(" -".to_string() + item);
                        };
                    },
                    Some(_) => return InteractionReturn::SilentMessage("Couldn't parse value in option `exclude`, is it valid?".to_string()),
                    None => (),
                }
            },
            "rating" => {
                match &option.resolved {
                    Some(CommandDataOptionValue::String(input)) => user_rating = input.to_string(),
                    Some(_) => return InteractionReturn::Message("This shouldn't happen, is discord drunk?".to_string()),
                    None => (),
                }
            },
            "count" => {
                match &option.resolved {
                    Some(CommandDataOptionValue::Integer(input)) => user_count = input.to_owned(),
                    Some(_) => return InteractionReturn::Message("what?".to_string()),
                    None => (),
                }
            },
            _ => return InteractionReturn::Message("Invalid option".to_string()),
        }
    }

    let master_exclude = match allow_nsfw {
        true => DISCORD_GLOBAL_EXCLUDE.to_string(),
        false => {
            if ! allow_ecchi {
                match user_rating.as_str() {
                    "rating:sensitive" => return InteractionReturn::SilentMessage(String::from("Rating `sensitive` has been disabled in SFW channels, please vote here https://discord.com/channels/1106263385588387850/1106263386376904777/1106271638191865906 if you want a say in this.")),
                    "rating:questionable" | "rating:explicit" => return InteractionReturn::SilentMessage(String::from("Ratings `Questionable` and `Explicit` are only allowed in NSFW channels!")),
                    _ => (),
                }
            }
            DISCORD_GLOBAL_EXCLUDE.to_string() + " " + &EXCLUDE_NSFW
        },
    };

    println!(
        "User command: Booru
- Tags:         {user_tags}
- Exclude:      {user_exclude}
- Rating:       {user_rating}
- NSFW channel: {allow_nsfw}"
    );

    let user_input = user_tags + &user_exclude + " " + &user_rating;

    let posts = match GelbooruClient::builder()
        .tag(user_input)        // User tags
        .tag(master_exclude)    // Avoid ToS violation
        .limit(user_count.try_into().unwrap())
        .random(true)
        .get()
        .await {
            Ok(data) => data,
            Err(_) => return InteractionReturn::SilentMessage("Error in trying to get post data, check your tags and try again".to_string()),
        };

    //dbg!(&posts);

    println!("Gelbooru posts:");

    let mut post_log = String::new();
    let mut response = String::new();

    for post in posts {
        post_log += &format!("- Post:
  + created:  {}
  + post url: https://gelbooru.com/index.php?page=post&s=view&id={}
  + file url: {}
  + tags:     {}
  + rating:   {}\n",
        post.created_at,
        post.id,
        post.file_url,
        post.tags,
        post.rating
        );

        response += format!("Post: <https://gelbooru.com/index.php?page=post&s=view&id={}>
Created at: {}
Rating: {}
{}\n\n",
            post.id,
            post.created_at,
            post.rating,
            post.file_url
        ).as_str();
    }

    print!("{}", post_log);

    InteractionReturn::Message(response)
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("booru")
        .description("Get a booru image")
        .create_option(|option| {
            option
                .name("tags")
                .description("tags to search for")
                .kind(CommandOptionType::String)
        })
        .create_option(|option| {
            option
                .name("exclude")
                .description("tags to exclude")
                .kind(CommandOptionType::String)
        })
        .create_option(|option| {
            option
                .name("rating")
                .description("rating")
                .kind(CommandOptionType::String)
                .add_string_choice("general", "rating:general")
                .add_string_choice("sensitive", "rating:sensitive")
                .add_string_choice("questionable", "rating:questionable")
                .add_string_choice("explicit", "rating:explicit")
        })
        .create_option(|option| {
            option
                .name("count")
                .description("amount of images")
                .kind(CommandOptionType::Integer)
        })
}

#[allow(dead_code)]
pub fn help() -> String {
    format!("`/booru` - Get an image from gelbooru

Options:
- tags: a space separated list of tags to search for.
    any non blacklisted gelbooru tags are supported.
- exclude: a space separated list of tags to exclude.
    any gelbooru tag is supported
- rating: what rating the image should be.
    + `general` -> SFW
    + `sensitive` -> Ecchi/risqué (panty-shots, breasts, ass, ...)
    + `questionable` -> Non-genital nudity and suggestive content
    + `explicit` -> NSFW
    Read more here: <https://gelbooru.com/index.php?page=wiki&s=&s=view&id=2535>
- count: how many images to post
    accepts any number, max of 5 due to discord's link embed limit.

Note for moderators:
This command may produce sexual explicit material in SFW channels, while we try to do our best to filter this out; gelbooru's tag system is not perfect.
If ecchi/suggestive content (rating:`sensitive`) is desirable in a SFW channel either add `ecchi` in the channel name or `boorubot: ecchi` in the channel description, this will allow ecchi content while filtering NSFW.
Additionally we block all images tagged with `{}` from our results due to discord ToS and community guidelines, this can not be disabled.", *DISCORD_GLOBAL_EXCLUDE)
}
