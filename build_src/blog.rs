use std::{collections::HashMap, path::PathBuf};

use crate::{errors::Result, nav::Nav, BuildContext};

use regex::Regex;
use siru::prelude::*;

pub struct BlogPost {
    file: PathBuf,
    slug: String,
    date: String,
    title: String,
    description: String,
    unlisted: bool,
}

#[derive(Deserialize)]
struct BlogPostFrontmatter {
    title: String,
    description: String,

    #[serde(default)]
    unlisted: bool,

    #[serde(default)]
    page: HashMap<String, String>,
}

pub fn list_blog_posts(ctx: &BuildContext) -> Result<Vec<BlogPost>> {
    let mut posts = Vec::new();

    let blog_post_regex = Regex::new(r"(?P<date>\d{4}-\d{2}-\d{2})-(?P<slug>.*)\.md")?;

    for entry in ctx.read_dir("blog")? {
        let entry = entry?;

        if let Some(captures) = entry
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|n| blog_post_regex.captures(n))
        {
            let date = captures.name("date").unwrap().as_str();
            let slug = captures.name("slug").unwrap().as_str();

            let blog_post_file = ctx.resolve("blog", entry.file_name())?;
            let frontmatter: BlogPostFrontmatter = parse_frontmatter(&ctx.read(&blog_post_file)?)?;

            posts.push(BlogPost {
                file: blog_post_file,
                slug: slug.to_string(),
                date: date.to_string(),
                title: frontmatter.title,
                description: frontmatter.description,
                unlisted: frontmatter.unlisted,
            })
        }
    }

    Ok(posts)
}

#[derive(Template)]
#[template(path = "blog_list.html.j2")]
struct BlogListTemplate<'a> {
    title: &'a str,
    description: Option<&'a str>,
    nav: &'a Vec<(String, String)>,
    blog_posts: &'a Vec<BlogPost>,
}

pub fn blog_list(ctx: &BuildContext) -> Result<()> {
    log_info("Rendering blog archive…");

    let blog_posts: &Vec<BlogPost> = ctx.resources.get();
    let nav: &Nav = ctx.resources.get();

    let template = BlogListTemplate {
        title: "Blog",
        description: Some("Thoughts & projects: Technology, programming, and language."),
        nav: &nav.0,
        blog_posts,
    };

    ctx.write("blog/index.html", template.render()?)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "blog_post.html.j2")]
struct BlogPostTemplate<'a> {
    title: &'a str,
    description: Option<&'a str>,
    nav: &'a Vec<(String, String)>,
    date: &'a str,
    content: &'a str,
    page: &'a HashMap<String, String>,
}

pub fn blog_posts(ctx: &BuildContext) -> Result<()> {
    log_info("Rendering blog posts…");

    let mut markdown_options = MarkdownOptions::default();
    markdown_options.render.unsafe_ = true;
    markdown_options.extension.footnotes = true;

    let blog_posts: &Vec<BlogPost> = ctx.resources.get();
    let nav: &Nav = ctx.resources.get();

    for post in blog_posts {
        let (fm, content) = render_markdown_with_options::<BlogPostFrontmatter>(
            &ctx.read(&post.file)?,
            &markdown_options,
        )?;

        let template = BlogPostTemplate {
            title: &post.title,
            description: Some(&post.description),
            nav: &nav.0,
            date: &post.date,
            content: &content,
            page: &fm.page,
        };

        ctx.write(
            format!("blog/{}/{}/index.html", post.date, post.slug),
            template.render()?,
        )?;
    }

    Ok(())
}

// TODO: RSS Feed
