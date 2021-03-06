pub mod kd_tree;
use crate::kd_tree::{Point, KdError};
extern crate num_traits;

// Include python module if feature is enabled
#[cfg(feature="default")]
pub mod py_module;
#[cfg(feature="default")]
extern crate ndarray;
#[cfg(feature="default")]
use ndarray::Array1;

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

    fn dimensions(&self) -> usize { self.len() }
}

impl Point<f32> for Vec<f32> {
    fn distance(&self, other: &Self) -> Result<f32, KdError> {
        if self.len() != other.len() {
            return Err(KdError::DimensionError);
        }

        let mut distance = 0f32;
        for i in 0..self.len() {
            let diff = self[i] - other[i];
            distance += diff * diff; 
        }
        Ok(distance.sqrt())
    }

    fn greater(&self, other: &Self, cur_dimension: usize) -> bool {
        self[cur_dimension] > other[cur_dimension]
    }

    fn split_plane(&self, cur_dimension: usize) -> Vec<f32> {
        let mut plane = vec![0f32; self.len()];
        plane[cur_dimension] = self[cur_dimension];
        plane
    }

    fn dimensions(&self) -> usize { self.len() }
}

#[cfg(feature="default")]
impl Point<f64> for Array1<f64> {
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

    fn split_plane(&self, cur_dimension: usize) -> Array1<f64> {
        let mut plane = Array1::zeros(self.len());
        plane[cur_dimension] = self[cur_dimension];
        plane
    }

    fn dimensions(&self) -> usize { self.len() }
}

#[cfg(test)]
mod tests {
    use super::kd_tree::KdTree;
    use std::time::{Instant};
    #[test]
    fn test_vecf64() {
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
            let brute_result = tree.brute_force(&query_point, 10);
            println!("Brute force finished in {}us", now.elapsed().as_micros());

            println!("KD-Search");
            let now = Instant::now();
            let search_result = tree.find_n_closest(&query_point, 10);
            println!("KD-Search finished in {}us", now.elapsed().as_micros());
            
            assert_eq!(search_result.is_ok(), true);
            if let (Ok(mut kd_search), Ok(mut brute_search)) = (search_result, brute_result) {
                for _i in 0..kd_search.len() {
                    if let (Some(kd_closest), Some(brute_closest)) = (kd_search.pop(), brute_search.pop()) {
                        assert!(kd_closest.distance == brute_closest.distance);
                        assert!(kd_closest.point[0] == brute_closest.point[0]);
                        assert!(kd_closest.point[1] == brute_closest.point[1]);
                        assert!(kd_closest.point[2] == brute_closest.point[2]);
                    }
                }
            }
        }
    }

    #[test]
    fn test_vecf32() {
        let mut tree = KdTree::<Vec<f32>, f32>::with_capacity(3, 1_000_000);

        println!("Constructing tree...");
        let now = Instant::now();
        for _i in 1..1_000_000 {
            match tree.add_point(vec![rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()]) {
                Ok(()) => { },
                Err(e) => {
                    println!("Failed to add point: {}", e);
                    assert!(false);
                },
            }
        }
        println!("Tree generated generated in {}us", now.elapsed().as_micros());

        for _i in 1..2 {
            let query_point = vec![rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()];

            println!("Brute force");
            let now = Instant::now();
            let brute_result = tree.brute_force(&query_point, 10);
            println!("Brute force finished in {}us", now.elapsed().as_micros());

            println!("KD-Search");
            let now = Instant::now();
            let search_result = tree.find_n_closest(&query_point, 10);
            println!("KD-Search finished in {}us", now.elapsed().as_micros());
            
            assert_eq!(search_result.is_ok(), true);
            if let (Ok(mut kd_search), Ok(mut brute_search)) = (search_result, brute_result) {
                for _i in 0..kd_search.len() {
                    if let (Some(kd_closest), Some(brute_closest)) = (kd_search.pop(), brute_search.pop()) {
                        assert!(kd_closest.distance == brute_closest.distance);
                        assert!(kd_closest.point[0] == brute_closest.point[0]);
                        assert!(kd_closest.point[1] == brute_closest.point[1]);
                        assert!(kd_closest.point[2] == brute_closest.point[2]);
                    }
                }
            }
        }
    }
}
