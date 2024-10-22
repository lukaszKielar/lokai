# LokAI TUI

LokAI is a local AI assistant in your terminal.

## Demo

![Demo](demo/lokai-demo.gif)

## Running app

Before we get started make sure you have following tools installed:

-   [Rust](https://www.rust-lang.org/tools/install)

To run LokAI type:

```bash
cargo run  # for more configuration flags see CLI section
```

### Persistence

Downloaded LLM models, logs, database and conversations are saved into `~/.lokai` directory.

### CLI

LokAI allow you to set some options through CLI.

-   `database-url` - defines location of SQLite database. Example values: "sqlite::memory:" (in-memory), "sqlite://db.slite3" (persistent), "db.sqlite3" (persitent)
-   `enable-transcription` - transcribes voice into prompt

To use one, many or all options type:

```bash
cargo run -- --database-url sqlite::memory: --enable-transcription
```

To print help type:

```bash
cargo run -- --help
```

## Shortcuts

| Shortcut                          | Action                       | App Context          |
| --------------------------------- | ---------------------------- | -------------------- |
| <kbd>Ctrl</kbd> + <kbd>c</kbd>    | Exit                         | Global               |
| <kbd>Ctrl</kbd> + <kbd>n</kbd>    | Add new conversation         | Global               |
| <kbd>Tab</kbd>                    | Next focus                   | Global               |
| <kbd>Shift</kbd> + <kbd>Tab</kbd> | Previous focus               | Global               |
| <kbd>Delete</kbd>                 | Delete selected conversation | Conversation sidebar |
| <kbd>↑</kbd>/<kbd>↓</kbd>         | Switch between conversations | Conversation sidebar |
| <kbd>↑</kbd>/<kbd>↓</kbd>         | Scroll up/down               | Chat/Prompt          |
| <kbd>Esc</kbd>                    | Cancel action                | Popups               |
