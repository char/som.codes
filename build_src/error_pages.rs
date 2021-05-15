use crate::{errors::Result, nav::Nav, BuildContext};
use siru::prelude::*;

#[derive(Template)]
#[template(path = "error_page.html.j2")]
struct ErrorPageTemplate<'a> {
    title: &'a str,
    description: Option<&'a str>,
    error_code: u32,
    error_description: &'a str,
    nav: &'a Vec<(String, String)>,
}

const ERRORS: &[(u32, &'static str)] = &[(404, "Page not found")];

pub fn error_pages(ctx: &BuildContext) -> Result<()> {
    log_info("Rendering error pages…");

    let nav: &Nav = ctx.resources.get();

    for (error_code, error_description) in ERRORS {
        let template = ErrorPageTemplate {
            title: error_description,
            description: None,
            error_code: *error_code,
            error_description,
            nav: &nav.0,
        };

        ctx.write(format!("{}.html", error_code), template.render()?)?
    }

    Ok(())
}
