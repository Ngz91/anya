<div align="center">
    <h1>🍱 Anya 🍱</h1>
    <h3> Terminal based HTTP client application </h3>
</div>

![](https://raw.githubusercontent.com/Ngz91/anya/master/screenshots/anya_example2.png)

Anya is a lightweight TUI HTTP client application. Test GET and POST methods with or without a JSON body from your terminal.

# Installation

- [Install](https://www.rust-lang.org/tools/install) Rust if it's not already installed in your system.
- Clone the repository.
- Run `cargo build --release`
- The created binary should be located at `./target/release/anya.*` in the cloned repository folder.

# Usage

Use the following keymaps to make a request to the endpoint of choice.

<b>Note:</b> Since the project is in it's early stages only GET and POST methods are supported.

| Keymap | Action |
------|------
| Ctrl + x | Change between layouts
| Ctrl + g | GET request
| Ctrl + h | POST request
| Esc or Ctrl + q | Exit
