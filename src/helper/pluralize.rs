use std::borrow::Cow;

pub fn pluralize<'a>(
    singular: impl Into<Cow<'a, str>>,
    plural: impl Into<Cow<'a, str>>,
    count: usize,
) -> Cow<'a, str> {
    if count == 1 {
        singular.into()
    } else {
        plural.into()
    }
}
