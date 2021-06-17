# Twitch Chat TUI

A simple terminal interface to read Twitch chat.

![](./readme-assets/example.gif)

## Features

* Read-only chat
* Customisable icons for 🌟subscriber, 🗡moderator, 💎vip, 🥇founder
* Works on Mac, Linux, Windows

## Quickstart

```bash
cargo install --git https://github.com/stuck-overflow/twitch-chat-tui.git --branch main

TWITCH_CHANNEL=<channel-to-join> twitch-chat-tui
```

## Configuration

Check [`twitch-chat-tui.toml`](./twitch-chat-tui.toml) for instructions and for all the available configuration parameters.

If you want to use an alternative `.toml` configuration file, run the tool with the `--config <your-file>.toml` flag.

## Contributing

### Bug Reports & Feature Requests

Please use the [issue tracker](https://github.com/stuck-overflow/twitch-chat-tui/issues) to report any bugs or file feature requests.

### Developing

PRs are welcome.
