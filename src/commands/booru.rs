use crate::{InteractionReturn, CONFIG};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    }
};
use booru_rs::client::gelbooru::GelbooruClient;
use booru_rs::client::generic::{BooruClient, BooruBuilder};

pub async fn run(options: &[CommandDataOption], allow_nsfw: bool, allow_ecchi: bool) -> InteractionReturn {

    // Generate blacklist
    let taglists = &CONFIG.read().await.booru_config;
    let tag_blacklist = &taglists.blacklist;
    let tag_blocklist = &taglists.force_block;
    let tag_nsfwlist = &taglists.nsfw_tags;

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
                        let mut blocked_tags: Vec<&str> = Vec::new();

                        for tag in tag_blacklist.iter() {
                            if input.contains(tag) {
                                illegal_tags.push(tag)
                            }
                        }

                        if !illegal_tags.is_empty() {
                            return InteractionReturn::SilentMessage(format!("Your search includes tags that would violate discord's ToS: `{}`", illegal_tags.join("` `")));
                        }

                        for tag in tag_blocklist.iter() {
                            if input.contains(tag) {
                                blocked_tags.push(tag)
                            }
                        }

                        if !blocked_tags.is_empty() {
                            return InteractionReturn::SilentMessage(format!("Your search includes tags that have been blocked by the bot operator: {}\nContact them if you believe this is in error.", blocked_tags.join("` `")));
                        }

                        if !allow_nsfw {
                            let mut blocked_nsfw_tags: Vec<String> = Vec::new();
                            for tag in tag_nsfwlist.iter() {
                                if input.contains(tag) {
                                    // Was complaining, added `.to_owned` and now it shuts up; no
                                    // fucking clue why and I'm not figuring it out.
                                    blocked_nsfw_tags.push(tag.to_owned());
                                }
                            }

                            if !blocked_nsfw_tags.is_empty() {
                                return InteractionReturn::SilentMessage(format!("You're using NSFW tags in a SFW channel: {}\nEither move to a NSFW channel or remove those tags", blocked_nsfw_tags.join("` `")));
                            }
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
        true => {
            invert_tags_and_stringify(tag_blacklist.clone())
        }
        false => {
            if ! allow_ecchi {
                match user_rating.as_str() {
                    "rating:sensitive" => return InteractionReturn::SilentMessage(String::from("Rating `sensitive` has been disabled in SFW channels, please vote here https://discord.com/channels/1106263385588387850/1106263386376904777/1106271638191865906 if you want a say in this.")),
                    "rating:questionable" | "rating:explicit" => return InteractionReturn::SilentMessage(String::from("Ratings `Questionable` and `Explicit` are only allowed in NSFW channels!")),
                    _ => (),
                }
            }
            let exclude_tags = invert_tags_and_stringify(tag_blacklist.clone());
            let exclude_nsfw = invert_tags_and_stringify(tag_nsfwlist.clone());
            let exclude_ecchi = invert_tags_and_stringify(tag_nsfwlist.clone());

            match allow_ecchi {
                true => exclude_tags + " " + &exclude_nsfw,
                false => exclude_tags + " " + &exclude_nsfw + " " + &exclude_ecchi,
            }
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

    println!("{}", &master_exclude);

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
pub async fn help() -> String {
    let blocked_tags = &CONFIG.read().await.booru_config.force_block.join(" ");
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
Additionally we block all images tagged with `{}` from our results due to discord ToS and community guidelines, this can not be disabled.", blocked_tags)
}

// Utility functions

/// Used for creating a negative tag string from a vec with tags
fn invert_tags_and_stringify(tags: Vec<String>) -> String {
    let mut negative_tags = Vec::new();

    for mut tag in tags {
        tag.insert(0, '-');
        negative_tags.push(tag);
    }

    negative_tags.join(" ")
}
