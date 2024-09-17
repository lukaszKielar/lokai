# LokAI TUI

## Roadmap

-   [ ] Dedicated settings tab, allowing to change things like:
    -   default LLM model
    -   ollama URL
-   [ ] Settings persistance - save TOML file in user's dir
-   [ ] Better error handling - new Result and Error structs allowing for clear distinction between critical and non-critical errors
-   [ ] Conversations
    -   [ ] Adding new conversation - design dedicated pop up
    -   [ ] Deleting conversation
    -   [ ] Changing settings for conversations, e.g. LLM model
-   [ ] Chat
    -   [ ] Highlighting code snippets returned by LLM
    -   [ ] Ability to copy chat or selected messages to clipboard
-   [ ] Prompt
    -   [ ] Set prompt's border to different colors depending on the factors like: empty prompt, LLM still replying, error
-   [ ] Ollama
    -   [ ] Downloading models (in the background)
    -   [ ] Polling Ollama Server to get the status - presenting status to users
    -   [ ] Present all available local models
-   [ ] Pop up or bottom bar presenting shortcuts
-   [ ] Bar that presents sliding messages (iterator for a piece of text that moves from right to left)
-   [ ] Tracing - log events for file
-   [ ] Tests
    -   [ ] Improve unit test coverage
    -   [ ] Create integration tests
    -   [ ]
-   [ ] Documentation improvements
-   [ ] Release tool to crates.io
-   [ ]
