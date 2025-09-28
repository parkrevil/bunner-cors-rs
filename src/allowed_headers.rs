/// Configuration for the `Access-Control-Allow-Headers` response value.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum AllowedHeaders {
    #[default]
    MirrorRequest,
    List(Vec<String>),
}

impl AllowedHeaders {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::List(values.into_iter().map(Into::into).collect())
    }
}
