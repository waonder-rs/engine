use scene::{
	Scene,
	Id
};

pub trait PointOfView<T, E> {
	type Iter<'a>: Iterator<Item=&'a Id<T>> where T: 'a;

	fn cycle(&mut self, scene: &Scene<T, E>);

	fn visible_objects<'a>(&'a self) -> Self::Iter<'a>;
}