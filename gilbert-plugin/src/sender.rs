use std::marker::PhantomData;

pub struct MapSenderTemp<'a, S, T, U, F>
where
    S: Sender<T>,
    F: Fn(U) -> T,
{
    inner: &'a S,
    mapping: F,
    p: PhantomData<(T, U)>,
}

impl<'a, S, T, U, F> Sender<U> for MapSenderTemp<'a, S, T, U, F>
where
    S: Sender<T>,
    F: Fn(U) -> T,
{
    fn send(&self, value: U) {
        self.inner.send((self.mapping)(value))
    }
}

pub trait Sender<T> {
    fn send(&self, value: T);
    fn map_temp<U, F>(&self, mapping: F) -> MapSenderTemp<'_, Self, T, U, F>
    where
        F: Fn(U) -> T,
        Self: Sized,
    {
        MapSenderTemp {
            inner: self,
            mapping,
            p: PhantomData,
        }
    }
}

impl<T> Sender<T> for tokio::sync::mpsc::UnboundedSender<T> {
    fn send(&self, value: T) {
        let _ = self.send(value);
    }
}
