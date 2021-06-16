use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use siru::{logging::log_info, prelude::SiruFS};

use crate::{
    blog::{render_blog_post, BlogPost},
    errors::Result,
    BuildContext,
};

const RSS_TEMPLATE: &'static str = include_str!("./templates/blog_rss/main.rss");
const ITEM_TEMPLATE: &'static str = include_str!("./templates/blog_rss/item.rss");

fn render_item(ctx: &BuildContext, post: &BlogPost) -> Result<String> {
    let link = &format!("https://som.codes/blog/{}/{}/", post.date, post.slug);
    let (_, content) = render_blog_post(ctx, post)?;

    let date_rfc2822 = DateTime::<Utc>::from_utc(
        NaiveDate::parse_from_str(&post.date, "%Y-%m-%d")
            .unwrap()
            .and_time(NaiveTime::from_hms(0, 0, 0)),
        Utc,
    )
    .to_rfc2822();

    Ok(ITEM_TEMPLATE
        .replace("<!-- $LINK -->", &link)
        .replace("<!-- $TITLE -->", &post.title)
        .replace("<!-- $DESCRIPTION -->", &post.description)
        .replace("<!-- $DATE -->", &date_rfc2822)
        .replace("<!-- $CONTENT -->", &content))
}

pub fn blog_rss(ctx: &BuildContext) -> Result<()> {
    log_info("Rendering blog RSS…");

    let blog_posts: &Vec<BlogPost> = ctx.resources.get();

    let rss = RSS_TEMPLATE.replace(
        "<!-- $ITEMS -->",
        &blog_posts
            .iter()
            .filter_map(|post| render_item(ctx, post).ok())
            .collect::<Vec<_>>()
            .join("\n"),
    );

    ctx.write("blog/blog.rss", &rss)?;

    Ok(())
}
