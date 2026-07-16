//! Interactive command dispatch.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bowser::{BrowserEngine, PageEngine};

use crate::StructuredFormat;
use crate::commands::page::render_page_summaries;
use crate::commands::repl::{HelperState, ReplCommand, update_page_state};
use crate::commands::{render_element, render_image_description, render_metadata};
use crate::error::Result;

use super::capture::capture_page_management_and_print;
use super::page_actions::{
    clear, click, go_back, go_forward, navigate, new_page, press_keys, print_yaml, reload,
    screenshot, select_option, submit, type_text,
};
use super::state::{load_stored_page_summaries, refresh_page_state};

pub(super) async fn dispatch_command(
    browser: &dyn BrowserEngine,
    page: &mut Box<dyn PageEngine>,
    config: &bowser::BrowserConfig,
    session_id: &str,
    command: ReplCommand,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    match command {
        ReplCommand::Goto(url) => navigate(page, &url, helper_state).await,
        ReplCommand::Back => go_back(page, helper_state).await,
        ReplCommand::Forward => go_forward(page, helper_state).await,
        ReplCommand::Reload => reload(page, helper_state).await,
        ReplCommand::Pages => print_pages(config, session_id, helper_state).await,
        ReplCommand::Page(page_id) => {
            select_page(browser, page, config, session_id, &page_id, helper_state).await
        }
        ReplCommand::NewPage(url) => {
            new_page(
                browser,
                page,
                config,
                session_id,
                url.as_deref(),
                helper_state,
            )
            .await
        }
        ReplCommand::ClosePage(page_id) => {
            close_page_command(
                browser,
                page,
                config,
                session_id,
                page_id.as_deref(),
                helper_state,
            )
            .await
        }
        ReplCommand::Yaml(id) => print_yaml(&**page, id, helper_state).await,
        ReplCommand::Click(id) => click(&**page, id, helper_state).await,
        ReplCommand::KeyPress(keys) => press_keys(&**page, &keys, helper_state).await,
        ReplCommand::Type(id, value) => type_text(&**page, id, &value, helper_state).await,
        ReplCommand::Clear(id) => clear(&**page, id, helper_state).await,
        ReplCommand::Select(id, value) => select_option(&**page, id, &value, helper_state).await,
        ReplCommand::Submit(id) => submit(&**page, id, helper_state).await,
        ReplCommand::Scroll(target) => {
            page.scroll(target).await?;
            Ok(())
        }
        ReplCommand::Screenshot(target, path) => screenshot(&**page, target, path).await,
        ReplCommand::Wait(selector) => {
            page.wait_for_selector(&selector, Duration::from_secs(10))
                .await?;
            Ok(())
        }
        ReplCommand::Refresh => super::capture::capture_and_print(&**page, helper_state).await,
        ReplCommand::Expand(id) => {
            println!(
                "{}",
                render_element(&page.expand(id).await?, StructuredFormat::Yaml)?
            );
            Ok(())
        }
        ReplCommand::Meta(id) => {
            println!(
                "{}",
                render_metadata(&page.metadata(id).await?, StructuredFormat::Yaml)?
            );
            Ok(())
        }
        ReplCommand::Describe(id) => {
            println!(
                "{}",
                render_image_description(&page.describe(id).await?, StructuredFormat::Yaml)?
            );
            Ok(())
        }
        ReplCommand::Session => {
            println!("{session_id}");
            Ok(())
        }
        ReplCommand::Url => {
            println!("{}", page.url().await?);
            Ok(())
        }
        ReplCommand::Title => {
            println!("{}", page.title().await?);
            Ok(())
        }
        ReplCommand::Html => {
            println!("{}", page.rendered_html().await?);
            Ok(())
        }
        ReplCommand::Js(expression) => {
            println!("{}", page.evaluate_js(&expression).await?);
            Ok(())
        }
        ReplCommand::Help => {
            crate::commands::repl::print_help();
            Ok(())
        }
        ReplCommand::Quit => Ok(()),
    }
}

async fn print_pages(
    config: &bowser::BrowserConfig,
    session_id: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let pages = load_stored_page_summaries(config, session_id).await?;
    update_page_state(helper_state, &pages);
    print!("{}", render_page_summaries(&pages));
    Ok(())
}

async fn select_page(
    browser: &dyn BrowserEngine,
    page: &mut Box<dyn PageEngine>,
    config: &bowser::BrowserConfig,
    session_id: &str,
    page_id: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    *page = browser.select_page(page_id).await?;
    refresh_page_state(config, session_id, helper_state).await?;
    capture_page_management_and_print(&**page, config, session_id, helper_state).await
}

async fn close_page_command(
    browser: &dyn BrowserEngine,
    page: &mut Box<dyn PageEngine>,
    config: &bowser::BrowserConfig,
    session_id: &str,
    page_id: Option<&str>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    *page = browser.close_page(page_id).await?;
    refresh_page_state(config, session_id, helper_state).await?;
    capture_page_management_and_print(&**page, config, session_id, helper_state).await
}
