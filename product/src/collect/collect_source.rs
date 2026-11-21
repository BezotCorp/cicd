#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CollectSource {
    Git,
    Ci,
}
