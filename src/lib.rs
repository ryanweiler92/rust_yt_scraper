pub mod extract;
pub mod models;

pub use extract::YoutubeExtractor;
pub use models::{VideoInfo, Comment, CommentContent};

#[cfg(feature = "python")]
mod python_bindings {
    use super::{Comment, VideoInfo, YoutubeExtractor};
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[pyclass]
    #[derive(Clone)]
    struct PyVideoInfo {
        #[pyo3(get)] title: String,
        #[pyo3(get)] channel: String,
        #[pyo3(get)] channel_id: String,
        #[pyo3(get)] description: String,
        #[pyo3(get)] yt_id: String,
        #[pyo3(get)] views: u64,
        #[pyo3(get)] comment_count: u64,
        #[pyo3(get)] like_count: u64,
        #[pyo3(get)] video_thumbnail: String,
        #[pyo3(get)] upload_date: String,
        #[pyo3(get)] channel_thumbnail: String,
    }

    impl From<VideoInfo> for PyVideoInfo {
        fn from(v: VideoInfo) -> Self {
            Self {
                title: v.title,
                channel: v.channel,
                channel_id: v.channel_id,
                description: v.description,
                yt_id: v.yt_id,
                views: v.views,
                comment_count: v.comment_count,
                like_count: v.like_count,
                video_thumbnail: v.video_thumbnail,
                upload_date: v.upload_date,
                channel_thumbnail: v.channel_thumbnail,
            }
        }
    }

    #[pymethods]
    impl PyVideoInfo {
        fn to_dict<'py>(&self, py: Python<'py>) -> Bound<'py, PyDict> {
            let d = PyDict::new_bound(py);
            d.set_item("title", &self.title).unwrap();
            d.set_item("channel", &self.channel).unwrap();
            d.set_item("channel_id", &self.channel_id).unwrap();
            d.set_item("description", &self.description).unwrap();
            d.set_item("yt_id", &self.yt_id).unwrap();
            d.set_item("views", self.views).unwrap();
            d.set_item("comment_count", self.comment_count).unwrap();
            d.set_item("like_count", self.like_count).unwrap();
            d.set_item("video_thumbnail", &self.video_thumbnail).unwrap();
            d.set_item("upload_date", &self.upload_date).unwrap();
            d.set_item("channel_thumbnail", &self.channel_thumbnail).unwrap();
            d
        }

        fn __repr__(&self) -> String {
            // Rust doesn't support Python's !r; use {:?} for debug
            format!(
                "<PyVideoInfo title={:?} channel={:?} views={}>",
                self.title, self.channel, self.views
            )
        }
    }

    #[pyclass]
    #[derive(Clone)]
    struct PyComment {
        #[pyo3(get)] comment_id: String,
        #[pyo3(get)] channel_id: String,
        #[pyo3(get)] video_id: String,
        #[pyo3(get)] display_name: String,
        #[pyo3(get)] user_verified: bool,
        #[pyo3(get)] thumbnail: String,
        #[pyo3(get)] content: String,
        #[pyo3(get)] published_time: String,
        #[pyo3(get)] like_count: i32,
        #[pyo3(get)] reply_count: i32,
        #[pyo3(get)] comment_level: i32,
        #[pyo3(get)] reply_to: String,
        #[pyo3(get)] reply_order: i32,
    }

    impl From<Comment> for PyComment {
        fn from(c: Comment) -> Self {
            Self {
                comment_id: c.comment_id,
                channel_id: c.channel_id,
                video_id: c.video_id,
                display_name: c.display_name,
                user_verified: c.user_verified,
                thumbnail: c.thumbnail,
                content: c.content,
                published_time: c.published_time,
                like_count: c.like_count,
                reply_count: c.reply_count,
                comment_level: c.comment_level,
                reply_to: c.reply_to,
                reply_order: c.reply_order,
            }
        }
    }

    #[pymethods]
    impl PyComment {
        fn to_dict<'py>(&self, py: Python<'py>) -> Bound<'py, PyDict> {
            let d = PyDict::new_bound(py);
            d.set_item("comment_id", &self.comment_id).unwrap();
            d.set_item("channel_id", &self.channel_id).unwrap();
            d.set_item("video_id", &self.video_id).unwrap();
            d.set_item("display_name", &self.display_name).unwrap();
            d.set_item("user_verified", self.user_verified).unwrap();
            d.set_item("thumbnail", &self.thumbnail).unwrap();
            d.set_item("content", &self.content).unwrap();
            d.set_item("published_time", &self.published_time).unwrap();
            d.set_item("like_count", self.like_count).unwrap();
            d.set_item("reply_count", self.reply_count).unwrap();
            d.set_item("comment_level", self.comment_level).unwrap();
            d.set_item("reply_to", &self.reply_to).unwrap();
            d.set_item("reply_order", self.reply_order).unwrap();
            d
        }

        fn __repr__(&self) -> String {
            format!("<PyComment by={:?} likes={}>", self.display_name, self.like_count)
        }
    }

    fn run_async<F, T>(fut: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'static,
        T: Send + 'static,
    {
        let rt = tokio::runtime::Runtime::new()?;
        let res = rt.block_on(fut)?;
        Ok(res)
    }

    // Return dicts: (video_info: dict, comments: list[dict])
    #[pyfunction]
    #[pyo3(signature = (video, max_requests=None))]
    fn extract(py: Python<'_>, video: &str, _max_requests: Option<usize>) -> PyResult<(PyObject, Vec<PyObject>)> {
        let extractor = YoutubeExtractor::new();
        let video = video.to_owned();

        let fut = async move {
            let (info, mut comments) = extractor.extract(&video).await?;
            let py_info: PyVideoInfo = info.into();
            let py_comments: Vec<PyComment> = comments.drain(..).map(Into::into).collect();
            Ok::<_, Box<dyn std::error::Error>>((py_info, py_comments))
        };

        match run_async(fut) {
            Ok((info, comments)) => {
                let vi_dict = info.to_dict(py).into_py(py);
                let comment_dicts = comments
                    .into_iter()
                    .map(|c| c.to_dict(py).into_py(py))
                    .collect::<Vec<PyObject>>();
                Ok((vi_dict, comment_dicts))
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Extraction failed: {}", e))),
        }
    }

    #[pymodule]
    fn yt_scraper(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(extract, m)?)?;
        m.add_class::<PyVideoInfo>()?;
        m.add_class::<PyComment>()?;
        Ok(())
    }
}
