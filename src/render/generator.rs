use crate::View;

pub trait Generator<T> {
	fn view(&self, object: &T) -> View;
}