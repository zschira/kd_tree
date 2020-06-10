use num_traits::Float;
use std::marker::PhantomData;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Copy, Clone)]
enum ChildType {
    RootNode,
    LeftChild,
    RightChild,
}

pub struct Closest<DataType, T> {
    point: DataType,
    distance: T,
}

impl<DataType, T: Float> Ord for Closest<DataType, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl<DataType, T: Float> PartialOrd for Closest<DataType, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl<DataType, T: Float> Eq for Closest<DataType, T> {}

impl<DataType, T: Float> PartialEq for Closest<DataType, T> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

pub trait Point<T: Float> {
    fn distance(&self, other: &Self) -> Result<T, KdError>;
    fn greater(&self, other: &Self, cur_dimesnion: usize) -> bool;
    fn split_plane(&self, cur_dimension: usize) -> Self;
    fn dimensions(&self) -> usize;
}

struct Node<DataType> {
    point: DataType,
    child_type: ChildType,
    parent: usize,
    left_child: usize,
    right_child: usize,
    dimension: usize,
    level: usize,
}

#[derive(Debug, PartialEq)]
pub enum KdError {
    DimensionError,
    EmptyTree,
    NodeMissing,
    BinaryHeapError,
}

pub struct KdTree<DataType, T> {
    tree: Vec<Option<Node<DataType>>>,
    num_dimensions: usize,
    max_levels: usize,
    last_point: usize,
    float_type: PhantomData<T>,
}

impl<T: Float, DataType: Point<T> + Clone> KdTree<DataType, T> {
    pub fn new(dimensions: usize) -> Self {
        // Default to capacity of 100 if no capacity is given
        let mut new_tree = KdTree {
            tree: Vec::with_capacity(100),
            num_dimensions: dimensions,
            max_levels: 0,
            last_point: 1,
            float_type: PhantomData,
        };
        new_tree.tree.resize_with(100, Default::default);
        new_tree
    }

    pub fn with_capacity(dimensions: usize, capacity: usize) -> Self {
        let mut new_tree = KdTree {
            tree: Vec::with_capacity(capacity),
            num_dimensions: dimensions,
            max_levels: 0,
            last_point: 1,
            float_type: PhantomData,
        };
        new_tree.tree.resize_with(capacity, Default::default);
        new_tree
    }

    pub fn add_point(&mut self, query_point: DataType) -> Result<(), KdError> {
        if query_point.dimensions() != self.num_dimensions { return Err(KdError::DimensionError); }

        let (parent_index, child_type) = if self.tree[1].is_none() {
            (0, ChildType::RootNode)
        } else {
            self.go_down(&query_point, 1)?
        };

        let (current_dimension, current_level) = if self.last_point == 1 {
            (0, 0)
        } else {
            if let Some(node) = &mut self.tree[parent_index] {
                match child_type {
                    ChildType::LeftChild => { node.left_child = self.last_point; },
                    ChildType::RightChild => { node.right_child = self.last_point; },
                    ChildType::RootNode => { },
                }

                ((node.dimension + 1) % self.num_dimensions, node.level + 1)
            } else {
                return Err(KdError::DimensionError);
            }
        };

        self.max_levels = self.max_levels.max(current_level);

        // Resize vector if at capacity
        if self.last_point >= self.tree.len() - 1  {
            let capacity = self.last_point * 2;
            self.tree.reserve(capacity);
            self.tree.resize_with(capacity, Default::default);
        }

        self.tree[self.last_point] = Some(Node {
                                    point: query_point,
                                    child_type: child_type,
                                    parent: parent_index,
                                    left_child: 0,
                                    right_child: 0,
                                    dimension: current_dimension,
                                    level: current_level,
                                });

        self.last_point += 1;

        Ok(())
    }

    pub fn find_closest(&self, query_point: &DataType) -> Result<(DataType, T), KdError> {
        match self.find_n_closest(query_point, 1)?.pop() {
            Some(closest) => { Ok((closest.point, closest.distance)) },
            None => { Err(KdError::BinaryHeapError) },
        }
    }

    pub fn find_n_closest(&self, query_point: &DataType, n: usize) -> Result<BinaryHeap<Closest<DataType, T>>, KdError> {
        let mut bh_closest = BinaryHeap::with_capacity(n);
        let mut searched_table = vec![-1i64; self.max_levels + 1];
        let (mut index, mut child_type) = self.go_down(query_point, 1)?;

        while let Some(node) = &self.tree[index] {
            let distance = node.point.distance(query_point)?;
            if bh_closest.len() < n {
                bh_closest.push(Closest { point: index, distance: distance, });
            } else {
                if distance < self.get_max_min(&bh_closest)? {
                    bh_closest.pop();
                    bh_closest.push(Closest { point: index, distance: distance, });
                }
            }
            if searched_table[node.level] == index as i64 {
                child_type = node.child_type;
                index = node.parent;
            } else {
                searched_table[node.level] = index as i64;
                if node.point.split_plane(node.dimension).distance(&query_point.split_plane(node.dimension))? < self.get_max_min(&bh_closest)? {
                    let sub_tree = match child_type {
                        ChildType::LeftChild => { node.right_child },
                        ChildType::RightChild => { node.left_child},
                        ChildType::RootNode => { 0 },
                    };

                    let go_down_result = self.go_down(query_point, sub_tree);
                    if let Ok((cur_ind, cur_child)) = go_down_result { index = cur_ind; child_type = cur_child; }
                }
            }
        }

        let mut bh_dtype = BinaryHeap::with_capacity(n);
        for closest in bh_closest.iter() {
            if let Some(node) = &self.tree[closest.point] {
                bh_dtype.push(Closest { point: node.point.clone(), distance: closest.distance });
            } else {
                return Err(KdError::NodeMissing);
            }
        }

        Ok(bh_dtype)
    }

    pub fn brute_force(&self, query_point: &DataType) -> Result<(DataType, T), KdError> {
        let mut min_distance: T = Float::max_value();
        let mut index = 0;
        for (cur_ind, node) in self.tree.iter().enumerate() {
            if let Some(cur_node) = node {
                let distance = cur_node.point.distance(query_point)?;
                if distance < min_distance {
                    min_distance = distance;
                    index = cur_ind;
                }
            }
        }

        if let Some(node) = &self.tree[index] {
            Ok((node.point.clone(), min_distance))
        } else {
            Err(KdError::NodeMissing)
        }
    }

    fn go_down(&self, query_point: &DataType, root: usize) -> Result<(usize, ChildType), KdError> {
        if self.tree[root].is_none() {
            return Err(KdError::EmptyTree);
        }

        let mut current_index = root;
        let mut index = current_index;
        let mut child_type = ChildType::LeftChild;
        while let Some(node) = &self.tree[current_index] {
            index = current_index;
            if node.point.greater(query_point, node.dimension) {
                current_index = node.left_child;
                child_type = ChildType::LeftChild;
            } else {
                current_index = node.right_child;
                child_type = ChildType::RightChild;
            }
        };

        Ok((index, child_type))
    }

    fn get_max_min(&self, bh_closest: &BinaryHeap<Closest<usize, T>>) -> Result<T, KdError> {
        match bh_closest.peek() {
            Some(max) => { Ok(max.distance) },
            None => { Err(KdError::BinaryHeapError) },
        }
    }
}


impl std::fmt::Display for KdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let description = match *self {
            KdError::DimensionError => "dimension error",
            KdError::EmptyTree => "no nodes in tree",
            KdError::NodeMissing => "Cant access current node",
            KdError::BinaryHeapError => "Error accessing binary heap",
        };
        write!(f, "KdTree error: {}", description)
    }
}

