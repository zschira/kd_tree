pub mod kd_tree;
use crate::kd_tree::{Point, KdError};

impl Point for Vec<f64> {
    fn distance(&self, other: &Self) -> Result<f64, KdError> {
        if self.len() != other.len() {
            return Err(KdError::DimensionError);
        }

        let mut distance = 0f64;
        for i in 0..self.len() {
            let diff = self[i] - other[i];
            distance += diff * diff; 
        }
        Ok(distance.sqrt())
    }

    fn greater(&self, other: &Self, cur_dimension: usize) -> bool {
        self[cur_dimension] > other[cur_dimension]
    }

    fn split_plane(&self, cur_dimension: usize) -> Vec<f64> {
        let mut plane = vec![0f64; self.len()];
        plane[cur_dimension] = self[cur_dimension];
        plane
    }
}

#[cfg(test)]
mod tests {
    use super::kd_tree::KdTree;
    #[test]
    fn it_works() {
        let mut tree = KdTree::<Vec<f64>>::new(3);
        tree.add_point(vec![0.5f64, 0.2f64, 0.1f64]);
        tree.add_point(vec![1.0f64, 1.0f64, 1.0f64]);
        tree.add_point(vec![2.0f64, 2.0f64, 2.0f64]);
        tree.add_point(vec![-1.0f64, -1.0f64, -1.0f64]);
        tree.add_point(vec![-2.0f64, -2.0f64, -2.0f64]);
        tree.add_point(vec![3.0f64, 3.0f64, 3.0f64]);
        tree.add_point(vec![-3.0f64, -3.0f64, -3.0f64]);
        let search_result = tree.find_closest(vec![0.5f64, 0.2f64, 0.1f64]);
        assert_eq!(search_result.is_ok(), true);
    }
}
