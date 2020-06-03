pub mod kd_tree;
use crate::kd_tree::{Point, KdError};
extern crate num_traits;

impl Point<f64> for Vec<f64> {
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
    use std::time::{Instant};
    #[test]
    fn it_works() {
        let mut tree = KdTree::<Vec<f64>, f64>::with_capacity(3, 1_000_000);

        println!("Constructing tree...");
        let now = Instant::now();
        for _i in 1..1_000_000 {
            match tree.add_point(vec![rand::random::<f64>(), rand::random::<f64>(), rand::random::<f64>()]) {
                Ok(()) => { },
                Err(e) => {
                    println!("Failed to add point: {}", e);
                    assert!(false);
                },
            }
        }
        println!("Tree generated generated in {}us", now.elapsed().as_micros());

        for _i in 1..2 {
            let query_point = vec![rand::random::<f64>(), rand::random::<f64>(), rand::random::<f64>()];

            println!("Brute force");
            let now = Instant::now();
            let brute_result = tree.brute_force(&query_point);
            println!("Brute force finished in {}us", now.elapsed().as_micros());

            println!("KD-Search");
            let now = Instant::now();
            let search_result = tree.find_closest(&query_point);
            println!("KD-Search finished in {}us", now.elapsed().as_micros());
            
            assert_eq!(search_result.is_ok(), true);
            match search_result {
                Ok((point, distance)) => {
                    if let Ok((point_brute, dist_brute)) = brute_result {
                        println!("KD: {}, BRUTE: {}", distance, dist_brute);
                        assert!(distance == dist_brute);
                        assert!(point[0] == point_brute[0]);
                        assert!(point[1] == point_brute[1]);
                        assert!(point[2] == point_brute[2]);
                    }
                },
                Err(e) => { println!("{}", e); }
            }
        }
    }
}
