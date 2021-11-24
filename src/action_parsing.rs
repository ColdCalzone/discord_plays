pub mod parsing {
    use std::{
        collections::HashMap,
        fs::{File, OpenOptions},
        io::{prelude::*, BufReader},
        path::Path,
    };

    // Scripting stuff
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    // Opcodes for actions
    #[derive(Clone, PartialEq, Debug)]
    pub enum Token {
        MouseMove {
            direction: Direction,
            distance: i32,
        },
        Key {
            button: enigo::Key,
            release: bool,
        },
        Click {
            button: enigo::MouseButton,
            release: bool,
        },
        Wait(u64),
        Type(String),
        Call(String),
        End,
    }

    #[derive(Debug, Clone)]
    pub struct Action {
        pub name: Option<String>,
        pub instructions: Vec<Token>,
    }

    pub fn parse_action_file() -> HashMap<String, Action> {
        let mut actions: HashMap<String, Action> = HashMap::new();
        {
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
            
            
            // Check entire file for actions before compiling actions
            let first_pass = BufReader::new(file);
            for line in first_pass.lines() {
                if let Ok(the_line) = line {
                    let no_comments = the_line.split("//").collect::<Vec<&str>>()[0].to_string();

                    let raw_instruction: Vec<&str> =
                        no_comments.split_whitespace().collect::<Vec<&str>>();

                    let trimmed_line = raw_instruction.join(" ");

                    if trimmed_line == "" {
                        continue;
                    }

                    if trimmed_line.ends_with(":") {
                        actions.insert(trimmed_line.split(":").collect::<Vec<&str>>()[0].to_string(), Action {
                            name: Some(trimmed_line.split(":").collect::<Vec<&str>>()[0].to_string()),
                            instructions: vec![]
                        });
                    }
                }
            }
        }
        let file : File = OpenOptions::new().read(true).open("actions.txt").unwrap();
        let reader = BufReader::new(file);

        let mut action: Action = Action {
            name: None,
            instructions: vec![],
        };

        let line_num: u64 = 0;
        for line in reader.lines() {
            // An example of my incredibly sophisticated naming system
            if let Ok(the_line) = line {
                let no_comments = the_line.split("//").collect::<Vec<&str>>()[0].to_string();

                let mut raw_instruction: Vec<&str> =
                    no_comments.split_whitespace().collect::<Vec<&str>>();

                let trimmed_line = raw_instruction.join(" ");

                if trimmed_line == "" {
                    continue;
                }

                if trimmed_line.ends_with(":") {
                    action.name =
                        Some(trimmed_line.split(":").collect::<Vec<&str>>()[0].to_string());
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
                            format!("Invalid distance 'move' instruction, line {}", line_num)
                                .as_str(),
                        ),
                    },
                    "press" | "hold" | "release" => {
                        let mut mouse: bool = false;

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
                                                "Invalid mouse button in '{}' instruction, line {}",
                                                raw_instruction[0],
                                                line_num
                                            )
                                        }
                                    },
                                    release: raw_instruction[0] == "release",
                                }
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
                                    // If there's a better way to check if a character is a key I'd love to hear it
                                    "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k"
                                        | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v"
                                        | "w" | "x" | "y" | "z" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "`" | "-" | "=" | "[" | "]" | "\\" | ";" | "'" | "," | "." | "/"  => enigo::Key::Layout(
                                        raw_instruction[1].chars().next().expect(
                                            format!(
                                                "Invalid character in '{}' at {} (This shouldn't happen)",
                                                raw_instruction[0],
                                                line_num
                                            )
                                            .as_str(),
                                        ),
                                    ),
                                    _ => {
                                        panic!(
                                            "Invalid key name '{}' in '{}' instruction, line {}\n{}",
                                            raw_instruction[1],
                                            raw_instruction[0],
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
                                release: raw_instruction[0] == "release",
                            },
                        };
                        if raw_instruction[0] == "hold" {
                            action.instructions.push(token.clone());
                            action.instructions.push(Token::Wait(
                                (if mouse {
                                    raw_instruction[3]
                                } else {
                                    raw_instruction[2]
                                })
                                .parse::<u64>()
                                .expect(
                                    format!(
                                        "Invalid time in 'hold' instruction, line {}",
                                        line_num
                                    )
                                    .as_str(),
                                ),
                            ));
                            match token {
                                Token::Key { button, release } => Token::Key {
                                    button: button,
                                    release: !release,
                                },
                                Token::Click { button, release } => Token::Click {
                                    button: button,
                                    release: !release,
                                },
                                _ => {
                                    panic!(
                                        "Invalid token in '{}' instruction. This shouldn't happen.",
                                        raw_instruction[0]
                                    );
                                }
                            }
                        } else {
                            token
                        }
                    }
                    "wait" => Token::Wait(raw_instruction[1].parse::<u64>().expect(
                        format!("Invalid time in 'wait' instruction, line {}", line_num).as_str(),
                    )),
                    "type" => {
                        raw_instruction.remove(0);
                        Token::Type(raw_instruction.join(" "))
                    }
                    "end" => Token::End,
                    _ => {
                        if actions.contains_key(&trimmed_line) {
                            Token::Call(trimmed_line)
                        } else {
                            panic!("Invalid instruction, line {}", line_num)
                        }
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
}
