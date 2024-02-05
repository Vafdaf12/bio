# Better IO
Better IO (BIO for short) is wrapper around any terminal-based app that provides several quality-of-life improvements, such as:
- **Regex Styling:** Output lines can be styled using regex, with the option to only select lines from `stderr` or `stdout`
- **No Fragmented Input:** Input is performed on its own line, and is maintained while the application produces output

# Usage
BIO looks for a `config.json` file in the current working directory to determine styling. The format for styling is as follows:
```json5
{
    "stdout": [
        {
            "pattern": "[Hh]ello",
            "foreground": "cyan",
            "background": "dark_red".
            "attributes": ["Bold", "Underlined"]
        },
        // ...
    ]
}
```
The valid usage for `foreground`, `background` and `attributes` are provided by [Crossterm](https://github.com/crossterm-rs/crossterm)
