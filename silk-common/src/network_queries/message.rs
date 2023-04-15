use serde::Deserialize;

pub trait Message:
    for<'a> Deserialize<'a> + std::default::Default + Send + Sync + 'static
{
}
