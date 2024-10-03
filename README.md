# LokAI TUI

LokAI is a local AI assistant in your terminal.

## Running app

Before we get started make sure you have following tools installed:

-   [Ollama](https://ollama.com/download)
-   [Rust](https://www.rust-lang.org/tools/install)

Ollama server has to be up and running before we start LokAI.

```bash
ollama serve
```

Open separate terminal and pull down your favourite model. Default model used by LokAI is `phi3.5:3.8b`, but you can use any of available [models](https://ollama.com/library).

```bash
ollama pull phi3.5:3.8b
```

Once you have the model you can run LokAI app.

```bash
cargo run  # for more configuration flags see CLI section
```

### CLI

LokAI allow you to set some options through CLI.

-   `ollama-url` [default: `http://localhost:11434`] - if you run LokAI in docker you may need to use `http://host.docker.internal:11434`
-   `default-llm-model` [default: `phi3.5:3.8b`] - the model you would like to use for all of your conversations. You can pass any model [supported](https://ollama.com/library) by Ollama. **Make sure you have it downloaded before you start LokAI**.
-   `database-url` [default: `sqlite::memory:`] - default value spins new in-memory instance that won't persist conversations between restarts. Example value for persistent database `sqlite://db.sqlite3`

To use one, many or all options simply type:

```bash
cargo run -- --database-url <DB_URL> --ollama-url <OLLAMA_URL>
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

-   [ ] ? Dedicated settings tab, allowing to change things like:
    -   default LLM model
    -   ollama URL
-   [ ] ? Settings persistance - save TOML file in user's dir
-   [ ] Better error handling - new Result and Error structs allowing for clear distinction between critical and non-critical errors
-   [ ] If nothing is presented in Chat area print shortcuts and welcoming graphics (logo)
    -   [ ] Create logo
-   [ ] Conversations
    -   [x] Adding new conversation - design dedicated pop up
    -   [x] Deleting conversation
    -   [ ] ? Changing settings for conversations, e.g. LLM model
-   [ ] Chat
    -   [ ] Highlighting code snippets returned by LLM
    -   [ ] Ability to copy chat or selected messages to clipboard
-   [ ] Prompt
    -   [ ] Set prompt's border to different colors depending on the factors like: empty prompt, LLM still replying, error
-   [ ] ? Ollama
    -   [ ] Downloading models (in the background)
    -   [ ] Polling Ollama Server to get the status - presenting status to users
    -   [ ] Present all available local models
-   [ ] Popup or presenting shortcuts
-   [ ] Bar that presents sliding messages (iterator for a piece of text that moves from right to left)
-   [ ] Tracing
-   [ ] Tests
    -   [ ] Improve unit test coverage
    -   [ ] Create integration tests
-   [ ] Documentation improvements
-   [ ] Use `kalosm` instead of Ollama
