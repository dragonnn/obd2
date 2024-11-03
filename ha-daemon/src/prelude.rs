pub trait Log {
    fn log(&self);
    fn let_log(self) -> Self
    where
        Self: Sized,
    {
        self.log();
        self
    }
}

impl<T, E: std::error::Error> Log for Result<T, E> {
    fn log(&self) {
        if let Err(e) = self {
            error!("{}", e);
        }
    }
}

pub trait AnyhowLog {
    fn log(&self);
    fn let_log(self) -> Self
    where
        Self: Sized,
    {
        self.log();
        self
    }
}

impl<T> AnyhowLog for Result<T, anyhow::Error> {
    fn log(&self) {
        if let Err(e) = self {
            error!("{}", e);
        }
    }
}
