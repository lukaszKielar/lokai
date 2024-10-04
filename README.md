# LokAI TUI

LokAI is a local AI assistant in your terminal.

## Running app

Before we get started make sure you have following tools installed:

-   [Rust](https://www.rust-lang.org/tools/install)

To run LokAI type:

```bash
cargo run  # for more configuration flags see CLI section
```

### CLI

LokAI allow you to set some options through CLI.

-   `database-url` [default: `sqlite::memory:`] - default value spins new in-memory instance that won't persist conversations between restarts. Example value for persistent database `sqlite://db.sqlite3`

To use one, many or all options type:

```bash
cargo run -- --database-url <DB_URL>
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
| <kbd>↑</kbd>/<kbd>↓</kbd>         | Switch between conversations | Conversation sidebar |
| <kbd>Delete</kbd>                 | Delete selected conversation | Conversation sidebar |
| <kbd>↑</kbd>/<kbd>↓</kbd>         | Scroll up/down               | Chat/Prompt          |
| <kbd>Esc</kbd>                    | Cancel action                | Popups               |

## Roadmap

-   [ ] ? Settings persistance - save TOML file in user's dir
-   [ ] Better error handling - new Result and Error structs allowing for clear distinction between critical and non-critical errors
-   [ ] If nothing is presented in Chat area print shortcuts and welcoming graphics (logo)
    -   [ ] Create logo
-   [ ] Conversations
    -   [x] Adding new conversation - design dedicated pop up
    -   [x] Deleting conversation
    -   [ ] Add `session_path` column to `conversations` table - it should store local path to chat session `LOKAI_DIR/chats/{chat_id}`
-   [ ] Chat
    -   [ ] Highlighting code snippets returned by LLM
    -   [ ] Ability to copy chat or selected messages to clipboard
    -   [ ] Save/load Kalosm Chat history to/from disk
    -   [ ] Create simple cache (or reuse some tool) to store Chat sessions to avoid constant reading/writing from/to disk
-   [ ] Prompt
    -   [ ] Set prompt's border to different colors depending on the factors like: empty prompt, LLM still replying, error
    -   [ ] Improve prompt transcription process. Currently there is no way to turn off microphone, and the app constantly listens until its killed. I need to toggle it on/off on demand.
-   [ ] Popup or presenting shortcuts
-   [ ] Implement `AppState` for sharing things like DB pool, Whisper, Llama, app config, lokai dir (app config is actually dependent on lokai dir)
-   [ ] Bar that presents sliding messages (iterator for a piece of text that moves from right to left)
-   [ ] Tracing
-   [ ] Tests
    -   [ ] Improve unit test coverage
    -   [ ] Create integration tests
-   [ ] Documentation improvements
