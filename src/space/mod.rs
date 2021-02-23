use crate::render;

pub trait PartialDistance<D> {
	fn distance(&self, other: &Self) -> Option<D>;
}

pub trait Scaled<D> {
	fn scale(&self) -> D;
}

pub struct Filter<S> {
	space: S
}

impl<E, S: Scene<T, E>> render::Scene for Filter<S> where  {
	type Object: Sync = Self::Item;
	type Event: Sync;

	type Iter<'a>: Iterator<Item=&'a Id<Self::Object>> where Self::Object: 'a;

	fn relevant_objects<'a>(&'a self) -> Self::Iter<'a>;

	fn view(&self, object: &Self::Object) -> View;
}