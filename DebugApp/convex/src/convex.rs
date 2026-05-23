use debug_ui::{
    camera::Camera,
    curve::{ArcCurve, DebugCurve},
    egui::{Color32, Painter, Pos2, Rect, Shape, Stroke},
};
use i_curve::flatten::{
    convex::ToIntConvex,
    segment::{CubicSegment, LineSegment, QuadSegment},
};
use i_overlay::i_float::{adapter::FloatPointAdapter, float::rect::FloatRect};
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

const ADAPTER_SCALE: f32 = 1.0;

pub fn paint_convex_shape(painter: &Painter, rect: Rect, camera: &Camera, curve: &DebugCurve) {
    let adapter = unit_adapter();
    let shapes = curve.to_int_convex_shapes(adapter);

    for shape in shapes {
        paint_int_convex(painter, rect, camera, &shape);
    }
}

trait CurveToIntConvexShape<I: IntNumber> {
    fn to_int_convex_shapes(&self, adapter: FloatPointAdapter<[f32; 2], I>) -> Vec<Vec<IntPoint<I>>>;
}

impl<I: IntNumber> CurveToIntConvexShape<I> for DebugCurve {
    fn to_int_convex_shapes(&self, adapter: FloatPointAdapter<[f32; 2], I>) -> Vec<Vec<IntPoint<I>>> {
        match self {
            DebugCurve::Line(points) => vec![
                LineSegment {
                    control_points: points.map(float_point),
                }
                .to_int_convex(adapter)
                .as_slice()
                .to_vec(),
            ],
            DebugCurve::Quad(points) => vec![
                QuadSegment {
                    control_points: points.map(float_point),
                }
                .to_int_convex(adapter)
                .as_slice()
                .to_vec(),
            ],
            DebugCurve::Cubic(points) => vec![
                CubicSegment {
                    control_points: points.map(float_point),
                }
                .to_int_convex(adapter)
                .as_slice()
                .to_vec(),
            ],
            DebugCurve::Arc(arc) => arc_to_int_convex_shape(*arc, adapter),
        }
    }
}

fn arc_to_int_convex_shape(
    arc: ArcCurve,
    adapter: FloatPointAdapter<[f32; 2]>,
) -> Vec<Vec<IntPoint>> {
    let sample_count = 48;
    let mut points = Vec::with_capacity(sample_count + 1);

    for index in 0..=sample_count {
        let t = index as f32 / sample_count as f32;
        points.push(arc.point_at(arc.start_angle + arc.sweep_angle * t));
    }

    points
        .windows(4)
        .step_by(3)
        .map(|chunk| {
            let segment = CubicSegment {
                control_points: [
                    float_point(chunk[0]),
                    float_point(chunk[1]),
                    float_point(chunk[2]),
                    float_point(chunk[3]),
                ],
            };

            segment.to_int_convex(adapter.clone()).as_slice().to_vec()
        })
        .collect()
}

fn paint_int_convex(painter: &Painter, rect: Rect, camera: &Camera, hull: &[IntPoint]) {
    if hull.is_empty() {
        return;
    }

    let screen_points = hull
        .iter()
        .map(|point| camera.screen_from_world(rect, int_to_float(*point)))
        .collect::<Vec<_>>();

    if screen_points.len() >= 3 {
        painter.add(Shape::convex_polygon(
            screen_points,
            Color32::from_rgba_premultiplied(128, 212, 156, 28),
            Stroke::new(1.5, Color32::from_rgb(128, 212, 156)),
        ));
    } else if screen_points.len() == 2 {
        painter.line_segment(
            [screen_points[0], screen_points[1]],
            Stroke::new(2.0, Color32::from_rgb(128, 212, 156)),
        );
    } else {
        painter.circle_filled(screen_points[0], 3.0, Color32::from_rgb(128, 212, 156));
    }
}

fn unit_adapter<I: IntNumber>() -> FloatPointAdapter<[f32; 2], I> {
    FloatPointAdapter::wi {
        dir_scale: ADAPTER_SCALE,
        inv_scale: 1.0 / ADAPTER_SCALE,
        offset: [0.0, 0.0],
        rect: FloatRect {
            min_x: -1_000_000.0,
            max_x: 1_000_000.0,
            min_y: -1_000_000.0,
            max_y: 1_000_000.0,
        },
    }
}

fn float_point(point: Pos2) -> [f32; 2] {
    [point.x, point.y]
}

fn int_to_float<I: IntNumber>(point: IntPoint<I>) -> Pos2 {
    Pos2::new(
        point.x.to_f32() / ADAPTER_SCALE,
        point.y.to_f32() / ADAPTER_SCALE,
    )
}
