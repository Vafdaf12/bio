use crossterm::style::{Attribute, Color, ContentStyle, Stylize};
use regex::Regex;
use serde::Deserialize;

fn split_match<'a, 'b>(text: &'a str, re: &'b Regex) -> Vec<(&'a str, bool)> {
    let mut matched = Vec::new();
    let mut last = 0;

    let mut loc = re.capture_locations();
    re.captures_read(&mut loc, text);

    for (i, matched_text) in re.find_iter(text).map(|x| (x.start(), x.as_str())) {
        if last != i {
            matched.push((&text[last..i], false));
        }
        matched.push((matched_text, true));
        last = i + matched_text.len();
    }

    if last < text.len() {
        matched.push((&text[last..], false));
    }

    matched
}

#[derive(Deserialize)]
pub struct StyleRule {
    #[serde(with = "serde_regex")]
    pub pattern: Regex,

    pub background: Option<Color>,
    pub foreground: Option<Color>,
    pub attributes: Option<Vec<Attribute>>,
}

impl StyleRule {
    pub fn content_style(&self) -> ContentStyle {
        let mut style = ContentStyle::new();
        style.background_color = self.background;
        style.foreground_color = self.foreground;
        if let Some(attrs) = self.attributes.as_ref() {
            for attr in attrs.iter() {
                style = style.attribute(*attr);
            }
        }

        style
    }
}

#[derive(Deserialize)]
pub struct RegexStyle {
    #[serde(default = "Vec::new")]
    stdout: Vec<StyleRule>,

    #[serde(default = "Vec::new")]
    stderr: Vec<StyleRule>,
}

impl RegexStyle {
    fn style_with(text: &str, style: &Vec<StyleRule>) -> String {
        let mut text = text.to_owned();
        for rule in style.iter() {
            text = split_match(&text, &rule.pattern)
                .into_iter()
                .map(|(text, matched)| match matched {
                    true => rule.content_style().apply(text),
                    false => text.stylize(),
                })
                .map(|s| s.to_string())
                .collect()
        }

        text
    }

    pub fn style_stdout(&self, text: &str) -> String {
        Self::style_with(text, &self.stdout)
    }
    pub fn style_stderr(&self, text: &str) -> String {
        Self::style_with(text, &self.stderr)
    }
}
