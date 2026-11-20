#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Remote {
    name: String,
    fetch_urls: Vec<String>,
    push_urls: Vec<String>,
}

impl Remote {
    pub(crate) fn new(name: String, fetch_urls: Vec<String>, push_urls: Vec<String>) -> Self {
        Self { name, fetch_urls, push_urls }
    }

    pub(crate) fn name(&self) -> &str { &self.name }
    pub(crate) fn fetch_urls(&self) -> &[String] { &self.fetch_urls }
    pub(crate) fn push_urls(&self) -> &[String] { &self.push_urls }
}
