use super::{Shape, N, P2, THICKNESS, Curve, FiniteCurve, PointOnShapeLocation};
use super::path::Path;
use super::primitives::{Circle, Segment};
use PointOnShapeLocation::*;

impl Shape for Circle {
    fn location_of(&self, point: P2) -> PointOnShapeLocation {
        let distance = (point - self.center).norm();

        if distance < self.radius - THICKNESS / 2.0 {
            Inside
        } else if distance < self.radius + THICKNESS / 2.0 {
            OnEdge
        } else {
            Outside
        }
    }
}

#[derive(Clone)]
pub struct Band<P: Path> {
    pub path: P,
    pub width: N,
}

impl<P: Path> Band<P> {
    pub fn new(path: P, width: N) -> Band<P> {
        Band { path: path, width: width }
    }

    pub fn outline(&self) -> P {
        if let (Some(left_path), Some(right_path)) =
            (
                self.path.shift_orthogonally(-self.width / 2.0),
                self.path.shift_orthogonally(self.width / 2.0).map(|p| {
                    p.reverse()
                }),
            )
        {
            let connector1 = Segment::line(left_path.end(), right_path.start()).expect(
                "Connectors should always be valid",
            );
            let connector2 = Segment::line(right_path.end(), left_path.start()).expect(
                "Connectors should always be valid",
            );
            P::new(
                left_path
                    .segments()
                    .iter()
                    .chain(&[connector1])
                    .chain(right_path.segments().iter())
                    .chain(&[connector2])
                    .cloned()
                    .collect(),
            )
        } else {
            self.path.clone()
        }
    }

    pub fn outline_distance_to_path_distance(&self, distance: N) -> N {
        if let (Some(left_path_length), Some(right_path_length)) =
            (
                self.path.shift_orthogonally(-self.width / 2.0).map(|p| {
                    p.length()
                }),
                self.path.shift_orthogonally(self.width / 2.0).map(
                    |p| p.length(),
                ),
            )
        {
            if distance > left_path_length + self.width + right_path_length {
                // on connector2
                0.0
            } else if distance > left_path_length + self.width {
                // on right side
                (1.0 - (distance - left_path_length - self.width) / right_path_length) *
                    self.path.length()
            } else if distance > left_path_length {
                // on connector1
                self.path.length()
            } else {
                // on left side
                (distance / left_path_length) * self.path.length()
            }
        } else {
            distance
        }
    }
}

impl<P: Path> Shape for Band<P> {
    fn location_of(&self, point: P2) -> PointOnShapeLocation {
        if let Some(along) = self.path.project(point) {
            let distance = (point - self.path.along(along)).norm();
            if distance < self.width / 2.0 - THICKNESS / 2.0 {
                if along < THICKNESS || along > self.path.length() - THICKNESS {
                    OnEdge
                } else {
                    Inside
                }
            } else if distance < self.width / 2.0 + THICKNESS / 2.0 {
                OnEdge
            } else {
                Outside
            }
        } else {
            Outside
        }
    }
}

pub struct SimpleShape<P: Path> {
    pub outline: P,
}

impl<P: Path> SimpleShape<P> {
    pub fn new(outline: P) -> Self {
        SimpleShape { outline }
    }
}

impl<P: Path> Shape for SimpleShape<P> {
    fn location_of(&self, point: P2) -> PointOnShapeLocation {
        if self.outline.includes(point) {
            OnEdge
        } else if self.outline.contains(point) {
            Inside
        } else {
            Outside
        }
    }
}

pub struct InvertedSimpleShape<P: Path> {
    simple: SimpleShape<P>,
}

impl<P: Path> Shape for InvertedSimpleShape<P> {
    fn location_of(&self, point: P2) -> PointOnShapeLocation {
        match self.simple.location_of(point) {
            OnEdge => OnEdge,
            Inside => Outside,
            Outside => Inside,
        }
    }
}