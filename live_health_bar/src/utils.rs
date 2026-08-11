pub trait Wrap
where
    Self: Sized,
{
    #[inline]
    fn wrap<R, F: FnOnce(Self) -> R>(self, f: F) -> R {
        f(self)
    }

    #[inline]
    fn wrap_ok<E>(self) -> Result<Self, E> {
        self.wrap(Ok)
    }

    #[inline]
    fn wrap_err<T>(self) -> Result<T, Self> {
        self.wrap(Err)
    }

    #[inline]
    fn wrap_err_unit(self) -> Result<(), Self> {
        self.wrap(Err)
    }

    #[inline]
    fn wrap_some(self) -> Option<Self> {
        self.wrap(Some)
    }
}

impl<T: Sized> Wrap for T {}
