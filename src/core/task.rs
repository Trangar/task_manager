use crate::core::Context;

pub trait Task {
    fn execute(self: Box<Self>, context: &mut Context);
}

pub trait TryTask {
    type Error;
    fn try_execute(self: Box<Self>, context: &mut Context) -> Result<(), Self::Error>;
    fn on_error(error: Self::Error, context: &mut Context);
}

impl<T, E> Task for T
where
    T: TryTask<Error = E>,
{
    fn execute(self: Box<Self>, context: &mut Context) {
        if let Err(e) = self.try_execute(context) {
            Self::on_error(e, context);
        }
    }
}
