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
	/// How far away is this point from the start of the path
	pub(crate) distance: f32,
}

impl Path {
	pub fn new(id: u64, points: Vec<Vec3>) -> Self {
		let mut path = Path {
			id,
			nodes: points.into_iter().map(|point| PathNode { point, percent: 0.0, distance: 0.0 }).collect()
		};
		path.update_distances();
		path.update_path_percents();
		path
	}

	pub fn points(&self) -> Vec<Vec3> {
		self.nodes.iter().map(|node| node.point).collect()
	}

	/// Total length of the path
	pub fn total_length(&self) -> f32 {
		self.nodes.last().map(|node| node.distance).unwrap_or(0.0)
	}

	fn update_distances(&mut self) {
		for i in 1..self.nodes.len() {
			self.nodes[i].distance = self.nodes[i - 1].distance + self.nodes[i - 1].point.distance(self.nodes[i].point);
		}
	}

	fn update_path_percents(&mut self) {
		let total_length = self.total_length();
		let mut dist_sum: f32 = 0.;
		for i in 0..self.nodes.len() - 1 {
			let distance = self.nodes[i + 1].distance - self.nodes[i].distance;
			self.nodes[i].percent = dist_sum / total_length;
			dist_sum += distance;
		}
		self.nodes.last_mut().map(|node| node.percent = 1.0);
	}

	/// Returns the point on the path at the given percent.
	#[allow(dead_code)]
	pub fn get_point_along_path_percent(&self, t: f32) -> Vec3 {
		for i in 0..self.nodes.len() - 1 {
			let p1 = &self.nodes[i];
			let p2 = &self.nodes[i + 1];
			if p1.percent <= t && t <= p2.percent {
				let t = (t - p1.percent) / (p2.percent - p1.percent);
				return p1.point.lerp(p2.point, t);
			}
		}
		if t < 0. {
			return self.nodes.first().map(|node| node.point).unwrap_or(Vec3::new(0., 0., 0.));
		} else {
			return self.nodes.last().map(|node| node.point).unwrap_or(Vec3::new(0., 0., 0.));
		}
	}

	/// Returns the point on the path at the given distance from the start.
	pub fn get_point_along_path(&self, distance: f32) -> Vec3 {
		for i in 0..self.nodes.len() - 1 {
			let p1 = &self.nodes[i];
			let p2 = &self.nodes[i + 1];
			if p1.distance <= distance && distance <= p2.distance {
				let t = (distance - p1.distance) / (p2.distance - p1.distance);
				return p1.point.lerp(p2.point, t);
			}
		}
		self.nodes.last().map(|node| node.point).unwrap_or(Vec3::new(0., 0., 0.))
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
fn test_total_length_2() {
	let path = Path::new(
		0,
		vec![
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(10.0, 0.0, 0.0),
		Vec3::new(10.0, 30.0, 0.0),
	]);
	assert_eq!(path.total_length(), 40.);
}

#[test]
fn test_get_point_along_path_percent() {
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
	assert_eq!(path.get_point_along_path_percent(0.0), Vec3::new(0.0, 0.0, 0.0));
	assert_eq!(path.get_point_along_path_percent(0.5), Vec3::new(15.0, 0.0, 0.0));
}

#[test]
fn test_get_point_along_path_distance() {
	let path = Path::new(
		0,
		vec![
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(10.0, 0.0, 0.0),
		Vec3::new(30.0, 0.0, 0.0),
	]);
	assert_eq!(path.nodes[0].distance, 0.);
	assert_eq!(path.nodes[1].distance, 10.);
	assert_eq!(path.nodes[2].distance, 30.);
	assert_eq!(path.get_point_along_path(0.), Vec3::new(0., 0., 0.));
	assert_eq!(path.get_point_along_path(5.), Vec3::new(5., 0., 0.));
}
