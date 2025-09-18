pub mod extract;
pub mod models;

pub use extract::YoutubeExtractor;
pub use models::{VideoInfo, Comment, CommentContent};
#[cfg(feature = "python")]
mod python_bindings {
    use super::{YoutubeExtractor};
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    // tiny async runner
    fn run_async<F, T>(fut: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'static,
        T: Send + 'static,
    {
        let rt = tokio::runtime::Runtime::new()?;
        Ok(rt.block_on(fut)?)
    }

    /// Returns: (video_info: dict, comments: list[dict])
    #[pyfunction]
    #[pyo3(signature = (video, max_requests=None))]
    fn extract(py: Python<'_>, video: &str, _max_requests: Option<usize>) -> PyResult<(PyObject, Vec<PyObject>)> {
        let video = video.to_owned();
        // Call your async Rust extractor
        let (info, mut comments) = run_async(async move {
            let extractor = YoutubeExtractor::new();
            let (info, comments) = extractor.extract(&video).await?;
            Ok::<_, Box<dyn std::error::Error>>((info, comments))
        }).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Extraction failed: {}", e)))?;

        // Build video_info dict
        let vi = PyDict::new(py);
        vi.set_item("title", &info.title)?;
        vi.set_item("channel", &info.channel)?;
        vi.set_item("channel_id", &info.channel_id)?;
        vi.set_item("description", &info.description)?;
        vi.set_item("yt_id", &info.yt_id)?;
        vi.set_item("views", info.views)?;
        vi.set_item("comment_count", info.comment_count)?;
        vi.set_item("like_count", info.like_count)?;
        vi.set_item("video_thumbnail", &info.video_thumbnail)?;
        vi.set_item("upload_date", &info.upload_date)?;
        vi.set_item("channel_thumbnail", &info.channel_thumbnail)?;
        let vi_obj = vi.into_py(py);

        // Build comments: list[dict]
        let mut out = Vec::with_capacity(comments.len());
        for c in comments.drain(..) {
            let d = PyDict::new(py);
            d.set_item("comment_id", &c.comment_id)?;
            d.set_item("channel_id", &c.channel_id)?;
            d.set_item("video_id", &c.video_id)?;
            d.set_item("display_name", &c.display_name)?;
            d.set_item("user_verified", c.user_verified)?;
            d.set_item("thumbnail", &c.thumbnail)?;
            d.set_item("content", &c.content)?;
            d.set_item("published_time", &c.published_time)?;
            d.set_item("like_count", c.like_count)?;
            d.set_item("reply_count", c.reply_count)?;
            d.set_item("comment_level", c.comment_level)?;
            d.set_item("reply_to", &c.reply_to)?;
            d.set_item("reply_order", c.reply_order)?;
            out.push(d.into_py(py));
        }

        Ok((vi_obj, out))
    }

    #[pymodule]
    fn yt_scraper(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(extract, m)?)?;
        Ok(())
    }
}
