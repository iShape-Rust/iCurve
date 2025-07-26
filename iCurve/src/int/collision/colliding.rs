#[derive(Debug, PartialEq)]
pub enum CollidingResult {
    Overlap,
    Touch,
    None
}

pub trait Colliding {
    fn collide(&self, other: &Self) -> CollidingResult;
    fn overlap(&self, other: &Self) -> bool;
}