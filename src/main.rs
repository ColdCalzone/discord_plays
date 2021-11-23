// Combination of copied from the serenity github and my own code.

use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::{File, OpenOptions},
    io::{prelude::*, BufReader, Write as FileWrite},
    path::Path,
    sync::Arc,
    thread::{sleep, spawn as thread_spawn},
    time::Duration,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands,
        macros::{check, command, group, help, hook},
        Args, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions, Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils,
};
use tokio::sync::Mutex;

use enigo::*;

// Scripting stuff
#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Opcodes for actions
#[derive(Clone, PartialEq, Debug)]
enum Token {
    MouseMove { direction: Direction, distance: i32 },
    Key { button: enigo::Key },
    KeyRelease { button: enigo::Key },
    Click { button: enigo::MouseButton },
    ClickRelease { button: enigo::MouseButton },
    Wait(u64),
    Type(String),
    End,
}

#[derive(Debug, Clone)]
struct Action {
    name: Option<String>,
    instructions: Vec<Token>,
}

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct GamerModeTracker;

impl TypeMapKey for GamerModeTracker {
    type Value = bool;
}

struct ActionTracker;

impl TypeMapKey for ActionTracker {
    type Value = HashMap<String, Action>;
}

struct CustomAbout;

impl TypeMapKey for CustomAbout {
    type Value = HashMap<String, String>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let data = ctx.data.read().await;
        let actions = data
            .get::<ActionTracker>()
            .expect("Expected ActionTracker in TypeMap.");
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("Discord plays started");
        println!(
            "Loaded {} action{}",
            actions.len(),
            if actions.len() > 1 { "s" } else { "" }
        );
    }
}

#[group]
#[only_in(guilds)]
#[commands(about, commands, latency)]
struct General;

#[group]
// Change this to fit your specific server setup
#[allowed_roles("Mods", "Admin")]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server moderators"]
#[commands(slow_mode, kill)]
struct Mods;

#[group]
// Change this to fit your specific server setup
#[allowed_roles("Mods", "Admin", "Discord Plays Manager")]
#[only_in(guilds)]
#[summary = "Commands for gaming actions"]
#[commands(
    reload_actions,
    start_discord_plays,
    stop_discord_plays,
    set_icon,
    set_title,
    set_description
)]
struct Gaming;

#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "If you need more info on a command, type `help [command]`"]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Command not found: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
#[suggestion_text = "Did you mean: {}?"]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.

async fn my_help(
    context: &Context,
    msg: &Message,
    mut args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let mut is_admin: bool = false;

    if let Some(member) = &msg.member {
        for role in &member.roles {
            // Change this to fit your specific server setup
            if role
                .to_role_cached(&context.cache)
                .await
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
                || role
                    .to_role_cached(&context.cache)
                    .await
                    .expect("Invalid Role. What")
                    .name
                    == "Mods".to_string()
                || role
                    .to_role_cached(&context.cache)
                    .await
                    .expect("Invalid Role. What")
                    .name
                    == "Admin".to_string()
            {
                is_admin = true;

                break;
            }
        }
    }
    // match being bad go brr
    let help_target: &str = &args.single::<String>().unwrap_or("".to_string());
    if help_target == "slow_mode"
        && (msg
            .author
            .has_role(&context.http, msg.guild_id.unwrap(), 759786910427381790)
            .await?
            || msg
                .author
                .has_role(&context.http, msg.guild_id.unwrap(), 759787598016151572)
                .await?
            || is_admin)
    {
        msg.channel_id.send_message(&context.http, |m| {
			m.embed(|e| {
				e.title("Slow Mode");
				e.field("For Mods.", "Allows you to enable or disable slow mode in a channel,\n which you can either select by ID or use in the target channel.", false);
				e
			});
			m
		}).await.unwrap();
    } else if help_target == "latency" {
        msg.channel_id.send_message(&context.http, |m| {
			m.embed(|e| {
				e.title("Latency")
				.field("For Anyone.", "Shows you the latency the bot is experiencing. \nYou may know this as \"ping\" in video games", false);
				e
			});
			
			m
		}).await.unwrap();
    } else if help_target == "kill" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Kill")
                        .field("For Mods.", "Kills the bot as soon as possible.", false);
                    e
                });

                m
            })
            .await
            .unwrap();
    } else if help_target == "about" {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("About").field(
                        "For Anyone",
                        "Displays information about the currently configured game.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    // set_title, set_description
    } else if help_target == "start_discord_plays" && is_admin {
        msg.channel_id.send_message(&context.http, |m| {
			m.embed(|e| {
				e.title("Start Discord Plays")
				.field("For Mods.", "Begin streaming actions from Discord to your computer.\nIt is recommended to have a failsafe and moderation ready before using, because otherwise you may have a very hard time turning it off.", false);
				e
			});
			
			m
		}).await.unwrap();
    } else if help_target == "stop_discord_plays" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Stop Discord Plays").field(
                        "For Mods.",
                        "Stops streaming actions from Discord to your computer.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    } else if help_target == "reload_actions" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Reload Actions").field(
                        "For Mods.",
                        "Reloads actions. If you made a change in the program, then it will be reflected.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    } else if help_target == "set_icon" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Set Icon").field(
                        "For Mods.",
                        "Sets the bot's avatar and the thumbnail in the about embed.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    } else if help_target == "set_title" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Set Title").field(
                        "For Mods.",
                        "Sets the title of the game being played in the about embed.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    } else if help_target == "set_description" && is_admin {
        msg.channel_id
            .send_message(&context.http, |m| {
                m.embed(|e| {
                    e.title("Set Description").field(
                        "For Mods.",
                        "Sets the description in the about emebd.",
                        false,
                    );
                    e
                });

                m
            })
            .await
            .unwrap();
    } else {
        let help_data = help_commands::create_customised_help_data(
            context,
            msg,
            &args,
            &groups,
            &owners,
            help_options,
        )
        .await;
        let mut formatted_data = HashMap::new();
        msg.channel_id.send_message(&context.http, |m| -> &mut serenity::builder::CreateMessage {
			m.embed(|e| -> &mut serenity::builder::CreateEmbed {
				e.description("If you need more info on a command, type `help [command]`\n~~Strikethrough commands~~ are unavailable because they require certain conditions, or are limited to server messages.");
				match help_data {
					help_commands::CustomisedHelpData::GroupedCommands {
						ref groups,
						help_description: _,
					} => for x in groups { formatted_data.insert(x.name, x.command_names.join("\n")); }
				    _ => {}
				}
				e.field("General", &formatted_data["General"], true);
				if is_admin {
					e.field("Mods", &formatted_data["Mods"], true);
				}
				e.field("Gaming", &formatted_data["Gaming"], true);
				e
			});
		
			m
		}).await?;
    }
    Ok(())
}

#[hook]
async fn before(ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    println!("{}", msg.content);
    let data = ctx.data.read().await;
    let mode = data
        .get::<GamerModeTracker>()
        .expect("Couldn't find Game mode tracker in TypeMap.");
    let actions = data
        .get::<ActionTracker>()
        .expect("Couldn't find actions in TypeMap.");

    if *mode {
        if actions.contains_key(&msg.content) {
            let used_action = actions[&msg.content].clone();

            thread_spawn(move || {
                let mut action_index: usize = 0;
                let mut enigo = Enigo::new();
                loop {
                    match &used_action.instructions[action_index] {
                        Token::MouseMove {
                            direction,
                            distance,
                        } => match direction {
                            Direction::Up => {
                                enigo.mouse_move_relative(0, -*distance);
                            }
                            Direction::Down => {
                                enigo.mouse_move_relative(0, *distance);
                            }
                            Direction::Left => {
                                enigo.mouse_move_relative(-*distance, 0);
                            }
                            Direction::Right => {
                                enigo.mouse_move_relative(*distance, 0);
                            }
                        },
                        Token::Key { button } => {
                            enigo.key_down(*button);
                        }
                        Token::KeyRelease { button } => {
                            enigo.key_up(*button);
                        }
                        Token::Click { button } => {
                            enigo.mouse_down(*button);
                        }
                        Token::ClickRelease { button } => {
                            enigo.mouse_up(*button);
                        }
                        Token::Wait(time) => {
                            sleep(Duration::from_millis(*time));
                        }
                        Token::Type(text) => {
                            enigo.key_sequence(&text);
                        }
                        Token::End => {
                            break;
                        }
                    }
                    action_index += 1;
                }
            })
            .join()
            .expect("Error running action");
        }
    }
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::{futures::future::BoxFuture, FutureExt};
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        &format!("Try this again in {} seconds.", info.as_secs()),
                    )
                    .await;
            }
        };
    }
    .boxed()
}

// All action parsing code

fn parse_action_file() -> HashMap<String, Action> {
    let file: File = if Path::new("actions.txt").exists() {
        OpenOptions::new().read(true).open("actions.txt").unwrap()
    } else {
        {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open("actions.txt")
                .unwrap();
        }
        println!(
            "Create new actions in actions.txt\nSee the GitHub for documentation and examples."
        );
        OpenOptions::new().read(true).open("actions.txt").unwrap()
    };
    let reader = BufReader::new(file);

    let mut actions: HashMap<String, Action> = HashMap::new();

    let mut action: Action = Action {
        name: None,
        instructions: vec![],
    };

    let mut line_num: u64 = 0;
    for line in reader.lines() {
        // An example of my incredibly sophisticated naming system
        if let Ok(the_line) = line {
            line_num += 1;
            if the_line == "" {
                continue;
            }
            let mut raw_instruction: Vec<&str> = the_line.split_whitespace().collect::<Vec<&str>>();

            let mut comment: bool = false;
            let mut index: usize = 0;
            let mut all_comment: bool = false;
            while index < raw_instruction.len() {
                if comment {
                    raw_instruction.remove(index);
                } else if raw_instruction[index].contains("//") {
                    comment = true;
                    if raw_instruction[index].starts_with("//") {
                        raw_instruction.remove(index);
                        if index == 0 {
                            all_comment = true;
                            break;
                        }
                    } else {
                        raw_instruction[index] =
                            raw_instruction[index].split("//").collect::<Vec<&str>>()[0];
                        index += 1;
                    }
                } else {
                    index += 1;
                }
            }
            if all_comment {
                continue;
            }
            if raw_instruction[0].ends_with(":") {
                action.name =
                    Some(raw_instruction[0].split(":").collect::<Vec<&str>>()[0].to_string());
                continue;
            }

            let instruction: Token = match raw_instruction[0] {
                "move" => Token::MouseMove {
                    direction: match raw_instruction[1] {
                        "up" => Direction::Up,
                        "down" => Direction::Down,
                        "left" => Direction::Left,
                        "right" => Direction::Right,
                        _ => {
                            panic!(
                                "Invalid mouse move direction in 'move' instruction, line {}",
                                line_num
                            )
                        }
                    },
                    distance: raw_instruction[2].parse::<i32>().expect(
                        format!("Invalid distance 'move' instruction, line {}", line_num).as_str(),
                    ),
                },
                "press" => match raw_instruction[1] {
                    "mouse" => Token::Click {
                        button: match raw_instruction[2] {
                            "left" => enigo::MouseButton::Left,
                            "middle" => enigo::MouseButton::Middle,
                            "right" => enigo::MouseButton::Right,
                            _ => {
                                panic!(
                                    "Invalid mouse button in 'press' instruction, line {}",
                                    line_num
                                )
                            }
                        },
                    },
                    // Kill
                    _ => Token::Key {
                        button: match raw_instruction[1] {
                            "alt" => enigo::Key::Alt,
                            "backspace" | "back" => enigo::Key::Backspace,
                            "caps_lock" => enigo::Key::CapsLock,
                            "control" | "ctrl" => enigo::Key::Control,
                            "del" | "delete" => enigo::Key::Delete,
                            "down" => enigo::Key::DownArrow,
                            "end" => enigo::Key::End,
                            "esc" | "escape" => enigo::Key::Escape,
                            "f1" => enigo::Key::F1,
                            "f10" => enigo::Key::F10,
                            "f11" => enigo::Key::F11,
                            "f12" => enigo::Key::F12,
                            "f2" => enigo::Key::F2,
                            "f3" => enigo::Key::F3,
                            "f4" => enigo::Key::F4,
                            "f5" => enigo::Key::F5,
                            "f6" => enigo::Key::F6,
                            "f7" => enigo::Key::F7,
                            "f8" => enigo::Key::F8,
                            "f9" => enigo::Key::F9,
                            "home" => enigo::Key::Home,
                            "left" => enigo::Key::LeftArrow,
                            "win" | "windows" | "meta" | "command" | "super" => enigo::Key::Meta,
                            "option" => enigo::Key::Option,
                            "pgdown" | "pg_down" | "page_down" => enigo::Key::PageDown,
                            "pgup" | "pg_up" | "page_up" => enigo::Key::PageUp,
                            "return" | "enter" => enigo::Key::Return,
                            "right" => enigo::Key::RightArrow,
                            "shift" => enigo::Key::Shift,
                            "space" => enigo::Key::Space,
                            "tab" => enigo::Key::Tab,
                            "up" => enigo::Key::UpArrow,
                            "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k"
                            | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v"
                            | "w" | "x" | "y" | "z" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "`" | "-" | "=" | "[" | "]" | "\\" | ";" | "'" | "," | "." | "/" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "`" | "-" | "=" | "[" | "]" | "\\" | ";" | "'" | "," | "." | "/" => enigo::Key::Layout(
                                raw_instruction[1].chars().next().expect(
                                    format!(
                                        "Invalid character in press at {} (This shouldn't happen)",
                                        line_num
                                    )
                                    .as_str(),
                                ),
                            ),
                            _ => {
                                panic!(
                                    "Invalid key name '{}' in 'press' instruction, line {}\n{}",
                                    raw_instruction[1],
                                    line_num,
                                    if raw_instruction[1] == "right"
                                        || raw_instruction[1] == "left"
                                        || raw_instruction[1] == "middle"
                                    {
                                        "Perhaps you meant to press a mouse button?"
                                    } else {
                                        ""
                                    }
                                )
                            }
                        },
                    },
                },
                "release" => match raw_instruction[1] {
                    "mouse" => Token::ClickRelease {
                        button: match raw_instruction[2] {
                            "left" => enigo::MouseButton::Left,
                            "middle" => enigo::MouseButton::Middle,
                            "right" => enigo::MouseButton::Right,
                            _ => {
                                panic!(
                                    "Invalid mouse button in 'release' instruction, line {}",
                                    line_num
                                )
                            }
                        },
                    },
                    // Kill
                    _ => Token::KeyRelease {
                        button: match raw_instruction[1] {
                            "alt" => enigo::Key::Alt,
                            "backspace" | "back" => enigo::Key::Backspace,
                            "caps_lock" => enigo::Key::CapsLock,
                            "control" | "ctrl" => enigo::Key::Control,
                            "del" | "delete" => enigo::Key::Delete,
                            "down" => enigo::Key::DownArrow,
                            "end" => enigo::Key::End,
                            "esc" | "escape" => enigo::Key::Escape,
                            "f1" => enigo::Key::F1,
                            "f10" => enigo::Key::F10,
                            "f11" => enigo::Key::F11,
                            "f12" => enigo::Key::F12,
                            "f2" => enigo::Key::F2,
                            "f3" => enigo::Key::F3,
                            "f4" => enigo::Key::F4,
                            "f5" => enigo::Key::F5,
                            "f6" => enigo::Key::F6,
                            "f7" => enigo::Key::F7,
                            "f8" => enigo::Key::F8,
                            "f9" => enigo::Key::F9,
                            "home" => enigo::Key::Home,
                            "left" => enigo::Key::LeftArrow,
                            "win" | "windows" | "meta" | "command" | "super" => enigo::Key::Meta,
                            "option" => enigo::Key::Option,
                            "pgdown" | "pg_down" | "page_down" => enigo::Key::PageDown,
                            "pgup" | "pg_up" | "page_up" => enigo::Key::PageUp,
                            "return" | "enter" => enigo::Key::Return,
                            "right" => enigo::Key::RightArrow,
                            "shift" => enigo::Key::Shift,
                            "space" => enigo::Key::Space,
                            "tab" => enigo::Key::Tab,
                            "up" => enigo::Key::UpArrow,
                            "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k"
                            | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v"
                            | "w" | "x" | "y" | "z" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "`" | "-" | "=" | "[" | "]" | "\\" | ";" | "'" | "," | "." | "/" => enigo::Key::Layout(
                                raw_instruction[1].chars().next().expect(
                                    format!(
                                        "Invalid character in release at {} (This shouldn't happen)",
                                        line_num
                                    )
                                    .as_str(),
                                ),
                            ),
                            _ => {
                                panic!(
                                    "Invalid key name '{}' in 'release' instruction, line {}\n{}",
                                    raw_instruction[1],
                                    line_num,
                                    if raw_instruction[1] == "right"
                                        || raw_instruction[1] == "left"
                                        || raw_instruction[1] == "middle"
                                    {
                                        "Perhaps you meant to press a mouse button?"
                                    } else {
                                        ""
                                    }
                                )
                            }
                        },
                    },
                },
                "wait" => Token::Wait(raw_instruction[1].parse::<u64>().expect(
                    format!("Invalid time in 'wait' instruction, line {}", line_num).as_str(),
                )),
                "type" => {
                    raw_instruction.remove(0);
                    Token::Type(raw_instruction.join(" "))
                },
                "end" => Token::End,
                // instructions that get compiled to more complex tokens
                // Somewhat hacky implementation, but it works perfectly fine.
                // Also code copying lmao
                "hold" => {
                    let mut mouse : bool = false;
                    let token = match raw_instruction[1] {
                        "mouse" => {
                            mouse = true;
                            Token::Click {
                            button: match raw_instruction[2] {
                                "left" => enigo::MouseButton::Left,
                                "middle" => enigo::MouseButton::Middle,
                                "right" => enigo::MouseButton::Right,
                                _ => {
                                    panic!(
                                        "Invalid mouse button in 'press' instruction, line {}",
                                        line_num
                                    )
                                }
                            },
                        }},
                        // Kill
                        _ => Token::Key {
                            button: match raw_instruction[1] {
                                "alt" => enigo::Key::Alt,
                                "backspace" | "back" => enigo::Key::Backspace,
                                "caps_lock" => enigo::Key::CapsLock,
                                "control" | "ctrl" => enigo::Key::Control,
                                "del" | "delete" => enigo::Key::Delete,
                                "down" => enigo::Key::DownArrow,
                                "end" => enigo::Key::End,
                                "esc" | "escape" => enigo::Key::Escape,
                                "f1" => enigo::Key::F1,
                                "f10" => enigo::Key::F10,
                                "f11" => enigo::Key::F11,
                                "f12" => enigo::Key::F12,
                                "f2" => enigo::Key::F2,
                                "f3" => enigo::Key::F3,
                                "f4" => enigo::Key::F4,
                                "f5" => enigo::Key::F5,
                                "f6" => enigo::Key::F6,
                                "f7" => enigo::Key::F7,
                                "f8" => enigo::Key::F8,
                                "f9" => enigo::Key::F9,
                                "home" => enigo::Key::Home,
                                "left" => enigo::Key::LeftArrow,
                                "win" | "windows" | "meta" | "command" | "super" => enigo::Key::Meta,
                                "option" => enigo::Key::Option,
                                "pgdown" | "pg_down" | "page_down" => enigo::Key::PageDown,
                                "pgup" | "pg_up" | "page_up" => enigo::Key::PageUp,
                                "return" | "enter" => enigo::Key::Return,
                                "right" => enigo::Key::RightArrow,
                                "shift" => enigo::Key::Shift,
                                "space" => enigo::Key::Space,
                                "tab" => enigo::Key::Tab,
                                "up" => enigo::Key::UpArrow,
                                "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k"
                                | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v"
                                | "w" | "x" | "y" | "z" => enigo::Key::Layout(
                                    raw_instruction[1].chars().next().expect(
                                        format!(
                                            "Invalid character in press at {} (This shouldn't happen)",
                                            line_num
                                        )
                                        .as_str(),
                                    ),
                                ),
                                _ => {
                                    panic!(
                                        "Invalid key name '{}' in 'press' instruction, line {}\n{}",
                                        raw_instruction[1],
                                        line_num,
                                        if raw_instruction[1] == "right"
                                            || raw_instruction[1] == "left"
                                            || raw_instruction[1] == "middle"
                                        {
                                            "Perhaps you meant to press a mouse button?"
                                        } else {
                                            ""
                                        }
                                    )
                                }
                            },
                        },
                    };
                    action.instructions.push(token.clone());
                    action.instructions.push(Token::Wait((if mouse {raw_instruction[3]} else {raw_instruction[2]}).parse::<u64>().expect(format!("Invalid time in 'hold' instruction, line {}", line_num).as_str())));
                    match token {
                        Token::Key { button } => Token::KeyRelease { button: button},
                        Token::Click { button } => Token::ClickRelease { button: button},
                        _ => { panic!("Invalid token in 'hold' instruction. This shouldn't happen!")}
                    }
                },
                _ => {
                    panic!("Invalid instruction, line {}", line_num)
                }
            };
            if instruction == Token::End {
                // I hate this and everything about this.
                action.instructions.push(instruction);
                println!("{:#?}", action);
                actions.insert(
                    action
                        .name
                        .as_ref()
                        .expect("No name for action")
                        .to_string(),
                    action,
                );
                action = Action {
                    name: None,
                    instructions: vec![],
                };
            } else {
                action.instructions.push(instruction);
            }
        }
    }
    return actions;
}

#[tokio::main]
async fn main() {
    let mut token: String = String::new();
    {
        // Configure the client with your Discord bot token in the file.
        let mut file: File;
        if Path::new("token.txt").exists() {
            file = OpenOptions::new().read(true).open("token.txt").unwrap();
        } else {
            {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("token.txt")
                    .unwrap();
            }
            panic!("Put your Discord Bot Token in the token.txt file");
        }
        file.read_to_string(&mut token).unwrap();
        print!("{}", token);
        token = token.replace("\n", "");
    }
    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefixes(["d!", "D!", "dp!", "Dp!", "DP!"])
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // Avoid using this to determine whether a specific command should be
        // executed. Instead, prefer using the `#[check]` macro which
        // gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your
        // application if you are fine using nightly Rust.
        // If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
        // Optionally `await_ratelimits` will delay until the command can be executed instead of
        // cancelling the command invocation.=
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MODS_GROUP)
        .group(&GAMING_GROUP);
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<ActionTracker>(parse_action_file());
        data.insert::<GamerModeTracker>(false);
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<CustomAbout>(HashMap::from([("title".to_string(), "Sample title".to_string()), ("description".to_string(), "Sample description. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string())]))
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        panic!("Client error: {:?}", why);
    }
}

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
#[check]
#[name = "Owner"]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Replace 7 with your ID to make this check pass.
    //
    // 1. If you want to pass a reason alongside failure you can do:
    // `Reason::User("Lacked admin permission.".to_string())`,
    //
    // 2. If you want to mark it as something you want to log only:
    // `Reason::Log("User lacked admin permission.".to_string())`,
    //
    // 3. If the check's failure origin is unknown you can mark it as such:
    // `Reason::Unknown`
    //
    // 4. If you want log for your system and for the user, use:
    // `Reason::UserAndLog { user, log }`
    if msg.author.id != 7 {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}

#[command]
// Limits the usage of this command to roles named:
async fn about_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let potential_role_name = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache).await {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.role_by_name(potential_role_name) {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, &format!("Role-ID: {}", role.id))
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    msg.channel_id
        .say(
            &ctx.http,
            format!("Could not find role named: {:?}", potential_role_name),
        )
        .await?;

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let fields = data
        .get::<CustomAbout>()
        .expect("Expected Actions in TypeMap.");
    let user = ctx.cache.current_user().await;

    let url = match user.avatar_url() {
        Some(url) => url,
        None => "".to_string(),
    };
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                // ???
                e.title(&fields[&"title".to_string()]);
                e.description(&fields[&"description".to_string()]);
                e.thumbnail(url);
                e
            });
            m
        })
        .await
        .unwrap();

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency))
        .await?;

    Ok(())
}

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) = msg
            .channel_id
            .edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds))
            .await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!(
                "Failed to set slow mode to `{}` seconds.",
                slow_mode_rate_seconds
            )
        } else {
            format!(
                "Successfully set slow mode rate to `{}` seconds.",
                slow_mode_rate_seconds
            )
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache).await
    {
        format!(
            "Current slow mode rate is `{}` seconds.",
            channel.slow_mode_rate.unwrap_or(0)
        )
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

#[command]
async fn kill(ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            std::process::exit(0);
        }
    };
    shard_manager.lock().await.shutdown_all().await;
    std::process::exit(0);
}

#[command]
async fn reload_actions(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let actions = data
        .get_mut::<ActionTracker>()
        .expect("Expected Actions in TypeMap.");
    *actions = parse_action_file();
    msg.react(&ctx.http, '✅').await?;
    Ok(())
}

#[command]
async fn start_discord_plays(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mode = data
        .get_mut::<GamerModeTracker>()
        .expect("Expected Game Tracker in TypeMap.");
    *mode = true;
    msg.react(&ctx.http, '✅').await?;
    Ok(())
}

#[command]
async fn stop_discord_plays(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mode = data
        .get_mut::<GamerModeTracker>()
        .expect("Expected Game Tracker in TypeMap.");
    *mode = false;
    msg.react(&ctx.http, '✅').await?;
    Ok(())
}

#[command]
async fn set_icon(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    //https://docs.rs/serenity/0.9.0/serenity/model/channel/struct.Attachment.html#method.download
    if msg.attachments.len() > 0 {
        let content = match msg.attachments[0].download().await {
            Ok(content) => content,
            Err(why) => {
                println!("Error downloading attachment: {:?}", why);
                let _ = msg.channel_id.say(&ctx, "Error downloading image").await;
                return Ok(());
            }
        };
        let mut file = match File::create("./avatar.png") {
            Ok(file) => file,
            Err(why) => {
                println!("Error creating file: {:?}", why);
                let _ = msg.channel_id.say(&ctx, "Error creating file").await;
                return Ok(());
            }
        };

        if let Err(why) = file.write(&content) {
            println!("Error writing to file: {:?}", why);

            return Ok(());
        }
    } else {
        msg.channel_id
            .say(
                &ctx,
                "No file provided; Please upload an image to change the avatar",
            )
            .await?;
        return Ok(());
    }

    let base64 = utils::read_image("./avatar.png").expect("Failed to read image");

    let mut user = ctx.cache.current_user().await;
    let _ = user.edit(&ctx, |p| p.avatar(Some(&base64))).await;
    msg.channel_id
        .say(&ctx, "Avatar successfully changed.")
        .await?;
    Ok(())
}

#[command]
async fn set_title(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let about = data
        .get_mut::<CustomAbout>()
        .expect("Expected CustomAbout in TypeMap.");
    about.insert("title".to_string(), args.rest().to_string());
    Ok(())
}

#[command]
async fn set_description(ctx: &Context, _msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let about = data
        .get_mut::<CustomAbout>()
        .expect("Expected CustomAbout in TypeMap.");
    about.insert("description".to_string(), args.rest().to_string());
    Ok(())
}
