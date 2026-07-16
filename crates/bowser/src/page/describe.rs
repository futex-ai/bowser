//! On-demand image description helpers.

use crate::ai::ImageFormat;
use crate::error::{Error, Result};
use crate::expand::{find_capture_element, find_element_mut};
use crate::model::{Element, ImageDescription, PageCapture};

use super::types::LivePage;

impl LivePage {
    pub(super) async fn describe_impl(&self, element_id: u32) -> Result<ImageDescription> {
        self.ensure_preview_capture().await?;
        let image = self.image_details(element_id).await?;
        if let Some(description) = image.description {
            return Ok(ImageDescription::new(
                element_id,
                image.alt,
                image.src,
                description,
            ));
        }
        let summarizer = self
            .summarizer
            .clone()
            .ok_or(Error::DescribeUnavailable { element_id })?;
        self.ensure_live_ids(element_id).await?;
        let bytes = self.screenshot_element_from_live_ids(element_id).await?;
        let description = summarizer.describe(&bytes, ImageFormat::Png).await?;
        self.cache_image_description(element_id, &description)
            .await?;
        Ok(ImageDescription::new(
            element_id,
            image.alt,
            image.src,
            description,
        ))
    }

    async fn image_details(&self, element_id: u32) -> Result<ImageState> {
        let state = self.state.lock().await;
        let capture = state
            .full_capture
            .as_ref()
            .or(state.preview_capture.as_ref())
            .ok_or(Error::ElementNotFound { element_id })?;
        match find_capture_element(capture, element_id) {
            Some(Element::Image {
                alt,
                src,
                description,
                ..
            }) => Ok(ImageState {
                alt: alt.clone(),
                src: src.clone(),
                description: description.clone(),
            }),
            Some(_) => Err(Error::DescribeNotImage { element_id }),
            None => Err(Error::ElementNotFound { element_id }),
        }
    }

    async fn cache_image_description(&self, element_id: u32, description: &str) -> Result<()> {
        let description = description.to_string();
        {
            let mut state = self.state.lock().await;
            if let Some(capture) = state.full_capture.as_mut() {
                set_image_description(capture, element_id, &description);
            }
            if let Some(capture) = state.preview_capture.as_mut() {
                set_image_description(capture, element_id, &description);
            }
            for record in &mut state.metadata_records {
                if let crate::model::MetadataRecord::Image {
                    element_id: record_id,
                    description: slot,
                    describable,
                    ..
                } = record
                    && *record_id == element_id
                {
                    *slot = Some(description.clone());
                    *describable = true;
                }
            }
        }
        self.persist_state(false).await
    }
}

#[derive(Clone)]
struct ImageState {
    alt: String,
    src: String,
    description: Option<String>,
}

fn set_image_description(capture: &mut PageCapture, element_id: u32, description: &str) {
    for children in capture.content.buckets_mut() {
        if let Some(Element::Image {
            description: slot, ..
        }) = find_element_mut(children, element_id)
        {
            *slot = Some(description.to_string());
            return;
        }
    }
}
