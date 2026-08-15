# platypus-powder-ping

Small Rust program for checking Nix Darwin packages for updates

- **Platypus** Darwin's mascot. "Hexley" I believe?
- **Powder** Snow, snowflake is Nix's icon
- **Ping** checking

This is where the name originated. Probably too long, but anything with a possible animal mascot tie-in is a bonus as far as I'm concerned.
It can easily be alias'ed.

<img width="355" height="562" alt="ppp-output" src="https://github.com/user-attachments/assets/130ec2c4-fe9b-4e25-8d74-b90efb97774c" />

## Background

I originally wanted to find a way to see if I had any updates in my [Nix Darwin](https://github.com/nix-darwin/nix-darwin) setup.
I have a _very_ simple (read: "naïve and lazy") Flake based setup that I use to manage my entire install (no profiles, no channels, no HomeManager). Works great for me!

It's trivially easy to just `nix flake update` and then `darwin-rebuild switch` get updates but... when do I have updates? 🤔

Nix has a few ways of doing something _close_ to my idea, but none of them really clicked with my pea-brain, so I decided to just start hacking around in Fish (my shell) and see what I could come up with.

Tons of searching, some asking LLMs "why is this broken?", and a few hours later, I came out with something that would give me a list of packages that had a different version in `unstable` than I had on my machine.

"Close Enough!" I said.

Then I decided I wanted the format to be tabular... okay that's doable with just `column` it turns
out... but what about... and maybe this thing... and wow this is kinda slow and hard to work on now...
and what if I want to run it every day at X time... 😅

So I decided I would write an actual program to run the Nix commands and then wrangle the data that came from them. This would hopefully make
it easier for tabular output, progress spinners, progress bars, and putting into a .plist so [Launchd](https://en.wikipedia.org/wiki/Launchd) could run it every day at noon and give me a notification if there were updates. Oh and instead of running `Nix Command for Each Package` sequentially, I could run them all in parallel (hopefully much faster) without losing my mind (yes "fork process" is a thing. "No" I don't want to use that)

"Yes, but what language?..." I thought to myself. My first though was **Go**, as its concurrency mechanisms are top-notch, however, the current meta is "Rewrite it in Rust".

So I did! 🦀🦀🦀

It will painfully clear to anyone who has ever programmed Rust that this was my first Rust project. There is at least some test coverage 🙏
(just barely over 70% according to `cargo llvm-cov`)

## Setup

There's a `flake.nix` but its purely for running a Rust dev shell (the extent of my Flake knowledge so far).  
I use `direnv` on my machine, so doing `cd`ing to the dir and then doing `direnv allow` should make everything work. YMMV depending on how you use Nix and/or if you already have a Rust dev env.

`cargo test`, `cargo run` or `cargo build --release` then running the binary should get you up and running.

Before you run the actual program (via `cargo run` or via the binary) you'll need a toml file at `./config/platypus-powder-ping/config.toml` (You can specify the file's location with the `-c` flag, but you will need a `config.toml` _somewhere_).

You can just copy the `config.toml.example` to that location and cut off the `.example` part. I could probably make a bootstrap/install sh file for that, but "maybe later".

There should be some basic help with `-h`.

### A Note on Notifications

This program can send notifications. I screwed around with several apps and Rust libraries, before I landed on using `macos-notifier`. It's a trivial small application that does what you WANT notifications to do, but aren't allowed to do outside of writing actual Apple apps (which I understand from a security standpoint, but "wow" it's annoying... kind of like some aspects of Rust, _amirite_ 😜).

[macos-notifier](https://github.com/pixelperfectat/macos-cli-notifier)  
Blog post by author [here](https://thecoder.io/blog/native-macos-notifications-from-the-command-line-without-terminal-notifier)

<img width="360" height="97" alt="macos-notifier" src="https://github.com/user-attachments/assets/5c29a571-bb3f-43d3-a8e9-9d54b0c4c95b" />


</br></br>
> That is lame and I don't want to download anything

No problem. If `macos-notifier` isn't on your path, it will notify with `osascript` which, IMHO, isn't that good looking, but "it'll work".

If you don't care about notifications, then you're looking after your mental health, and I say "good for you!".
