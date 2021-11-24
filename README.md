# Discord Plays
 A Discord bot allowing Discord to play games on your computer.
# Actions
This bot has a simple scripting language allowing you to create "actions," or simple commands for your chat to use.
```p
// actions.txt
// This is how to define an action. 
// There must be a new line after the :
move left: // Discord can call this action by typing MoveLeft
hold left 2000 // hold the left arrow key for 2 seconds (2000 milliseconds)
end // All actions must have an "end" statement to declare them as over.
move right:
hold right 2000 
end
Jump:
hold space 750 // hold space for 750 milliseconds (0.75 seconds)
end
Talk:
hold enter 100 // hold enter for 100 milliseconds (0.1 seconds)
// basically, tap enter
end
ComplexAction:
press mouse left // Press and hold the left mouse button
move left 500 // Move the cursor 500 pixels to the left
release mouse left // Release the left mouse button
press ctrl
hold c 500
release ctrl
end
```
# So how do I use it?
Due to how this bot works - directly controlling your computer - this bot could not be hosted publically (Also, hosting is expensive!)
Instead, you have to make a Discord bot, ~~install~~ compile the bot software on your computer [(See below)](https://github.com/ColdCalzone/discord_plays#compiling-the-program), and set up a `token.txt`and `actions.txt`file where the executable is. You can also compile it yourself, ~~however this likely won't be necessary.~~ which until I can compile for other platforms, will be the only option.
## Making a Discord bot
First, [Go here.](https://discord.com/developers/applications) Click the blue button in the top right (pictured below)
![Example](https://coldcalzone.github.io/pictures/Screenshot%20from%202021-11-22%2019-01-56.png)
Then, Name your project (Do whatever, ideally name it some variant "Discord Plays").
After that, fill out the information presented to you, most importantly the Name and Avatar.
Then, click `Bot` on the left.
![Example](https://coldcalzone.github.io/pictures/Screenshot%20from%202021-11-22%2019-06-48.png)
Click Add Bot.
Fill out the information again (This time it actually matters)
Scroll down and enable all three buttons below `Privileged Gateway Intents`.
![Example](https://coldcalzone.github.io/pictures/Screenshot%20from%202021-11-22%2019-11-45.png)
Scroll up and copy the token - keep this under lock and key, with it anyone can use your bot - and paste it into the tokens.txt file.
Finally, click `OAuth2`, then click `URL Generator` below that. You will be presented with several checkboxes, hit the `bot` box (seen below)
![Example](https://coldcalzone.github.io/pictures/Screenshot%20from%202021-11-22%2019-16-09.png)
Ignore the next set of checkboxes, copy the url below it, but replace `permissions=0` with `permissions=67225664`
#### Huzzah! You have an invite link for your very own Discord bot!
##### But it's not doing anything!
Next you have to ~~run the program you downloaded on your computer,~~ compile the program. Assuming everything goes well the bot will whirr to life and you'll have your very own discord bot, which controls your computer.
## Compiling The Program
This is where a first-time user may have some problems, but don't worry! It won't be very hard.
First open up your terminal / command prompt, [head here](https://www.rust-lang.org/learn/get-started) and follow the instructions to install `rustup`. Then, move your terminal to where you put the source code (What you downloaded above) and execute `cargo build --release`. You can now close the terminal. Open up that folder with a regular file explorer and go to `target/release/`. Copy the `discord_plays` file out of there. This file is what you will run to start the bot. I recommend placing this in a folder named `Discord Plays` and deleting the folder the code was in.
##### That's it! Now you have a usable Discord bot!
