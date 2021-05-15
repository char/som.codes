mod errors;

mod assets;
mod blog;
mod error_pages;
mod main_page;
mod nav;
mod node_worker;

use std::{
    convert::TryInto,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use node_worker::NodeWorker;
use once_cell::sync::Lazy;
use siru::boilerplate::hooks::*;
use siru::prelude::*;

use crate::{blog::list_blog_posts, nav::Nav};

pub struct BuildContext {
    source_dir: PathBuf,
    output_dir: PathBuf,
    write_pipeline: WritePipeline,
    resources: Resources,
}

impl SiruFS for BuildContext {
    fn get_source_dir(&self) -> &PathBuf {
        &self.source_dir
    }

    fn get_output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    fn get_write_pipeline(&self) -> &WritePipeline {
        &self.write_pipeline
    }
}

const MIN_HTML_HEADER: &'static str = concat!(
    "<!-- This file is minified, but you can find this site's original source at ",
    "https://github.com/videogame-hacker/som.codes/",
    " -->\n",
);

static WORKER: Lazy<Mutex<NodeWorker>> = Lazy::new(|| Mutex::new(NodeWorker::new()));

fn highlight_hook(path: PathBuf, contents: Vec<u8>) -> std::io::Result<Vec<(PathBuf, Vec<u8>)>> {
    if path
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| s.ends_with(".html"))
        .is_some()
    {
        let contents = String::from_utf8_lossy(&contents);
        let highlighted_html = WORKER.lock().unwrap().highlight(&contents)?;

        return Ok(vec![(path, highlighted_html.into_bytes())]);
    }

    Ok(vec![(path, contents)])
}

fn minify_hook(path: PathBuf, contents: Vec<u8>) -> std::io::Result<Vec<(PathBuf, Vec<u8>)>> {
    if path
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| s.ends_with(".html"))
        .is_some()
    {
        let contents = String::from_utf8_lossy(&contents);
        let mut minified_html = WORKER.lock().unwrap().minify(&contents)?;
        minified_html.insert_str(0, MIN_HTML_HEADER);

        return Ok(vec![(path, minified_html.into_bytes())]);
    }

    Ok(vec![(path, contents)])
}

fn main() {
    let mut ctx = BuildContext {
        source_dir: "src".try_into().unwrap(),
        output_dir: "dist".try_into().unwrap(),
        write_pipeline: WritePipeline::new(),
        resources: Resources::new(),
    };

    ctx.write_pipeline.push(mkdirs_hook);
    ctx.write_pipeline.push(highlight_hook);
    ctx.write_pipeline.push(minify_hook);
    ctx.write_pipeline.push(logging_hook);

    let nav = Nav(vec![("Blog".to_string(), "/blog/".to_string())]);
    ctx.resources.add(nav);
    ctx.resources.add(list_blog_posts(&ctx).unwrap());

    let ctx = Arc::new(ctx);
    [
        assets::copy_assets,
        main_page::main_page,
        blog::blog_list,
        blog::blog_posts,
        error_pages::error_pages,
    ]
    .iter()
    .map(|f| {
        let ctx = Arc::clone(&ctx);
        std::thread::spawn(move || f(&ctx).unwrap())
    })
    .for_each(|t| t.join().unwrap());
}
