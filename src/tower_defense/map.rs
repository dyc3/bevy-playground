use bevy::prelude::*;

#[derive(Component)]
pub struct Path {
	pub id: u64,
	nodes: Vec<PathNode>,
}

#[derive(Debug, Clone, Copy)]
struct PathNode {
	pub(crate) point: Vec3,
	/// At what percent of the path is this point
	pub(crate) percent: f32,
}

impl Path {
	pub fn new(id: u64, points: Vec<Vec3>) -> Self {
		let mut path = Path {
			id,
			nodes: points.into_iter().map(|point| PathNode { point, percent: 0.0 }).collect()
		};
		path.update_path_percents();
		path
	}

	pub fn points(&self) -> Vec<Vec3> {
		self.nodes.iter().map(|node| node.point).collect()
	}

	pub fn total_length(&self) -> f32 {
		let mut total_length = 0.0;
		for i in 0..self.nodes.len() - 1 {
			total_length += self.nodes[i].point.distance(self.nodes[i + 1].point);
		}
		total_length
	}

	fn update_path_percents(&mut self) {
		let total_length = self.total_length();
		let mut dist_sum: f32 = 0.;
		for i in 0..self.nodes.len() - 1 {
			let distance = self.nodes[i].point.distance(self.nodes[i + 1].point);
			self.nodes[i].percent = dist_sum / total_length;
			dist_sum += distance;
		}
		let len = self.nodes.len(); // borrowck workaround
		self.nodes[len - 1].percent = 1.0;
	}

	pub fn get_point_along_path(&self, t: f32) -> Vec3 {
		for i in 0..self.nodes.len() - 1 {
			let p1 = &self.nodes[i];
			let p2 = &self.nodes[i + 1];
			if p1.percent <= t && t <= p2.percent {
				let t = (t - p1.percent) / (p2.percent - p1.percent);
				return p1.point.lerp(p2.point, t);
			}
		}
		Vec3::new(-100., 0., 0.)
	}
}

#[test]
fn test_total_length() {
	let path = Path::new(
		0,
		vec![
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(10.0, 0.0, 0.0),
		Vec3::new(30.0, 0.0, 0.0),
	]);
	assert_eq!(path.total_length(), 30.);
}

#[test]
fn test_get_point_along_path() {
	let path = Path::new(
		0,
		vec![
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(10.0, 0.0, 0.0),
		Vec3::new(30.0, 0.0, 0.0),
	]);
	assert_eq!(path.nodes[0].percent, 0.0);
	assert_eq!(path.nodes[1].percent, 1./3.);
	assert_eq!(path.nodes[2].percent, 1.0);
	assert_eq!(path.get_point_along_path(0.0), Vec3::new(0.0, 0.0, 0.0));
	assert_eq!(path.get_point_along_path(0.5), Vec3::new(15.0, 0.0, 0.0));
}
