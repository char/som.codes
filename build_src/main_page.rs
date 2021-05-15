use crate::{errors::Result, nav::Nav, BuildContext};

use siru::{logging::log_info, prelude::*};

#[derive(Serialize, Deserialize)]
struct MainPageFrontmatter {
    title: String,
    description: String,
}

#[derive(Template)]
#[template(path = "main_page.html.j2")]
struct MainPageTemplate<'a> {
    title: &'a str,
    description: Option<&'a str>,
    nav: &'a Vec<(String, String)>,
    content: &'a str,
}

pub fn main_page(ctx: &BuildContext) -> Result<()> {
    log_info("Rendering main page…");

    let mut options = MarkdownOptions::default();
    options.render.unsafe_ = true;

    let (main_frontmatter, content) =
        render_markdown_with_options::<MainPageFrontmatter>(&ctx.read("main.md")?, &options)?;

    let nav: &Nav = ctx.resources.get();

    let template = MainPageTemplate {
        title: &main_frontmatter.title,
        description: Some(&main_frontmatter.description),
        nav: &nav.0,
        content: &content,
    };

    ctx.write("index.html", template.render()?)?;

    Ok(())
}
