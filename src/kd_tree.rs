use num_traits::Float;
use std::marker::PhantomData;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Node structure used by tree
struct Node<DataType> {
    point: DataType,                             // Point with user defined datatype
    child_type: NodeType,                        // Node type
    parent: usize,                               // Index of parent node
    left_child: usize,                           // Index of left child (0 if no left child)
    right_child: usize,                          // Index of right child (0 if no right child)
    dimension: usize,                            // Split dimension of current node
    level: usize,                                // Level in tree of current node
}

/// Tree structure with vector of nodes
pub struct KdTree<DataType, T> {
    tree: Vec<Option<Node<DataType>>>,           // Vector of nodes
    num_dimensions: usize,                       // Number of dimensions in DataType
    max_levels: usize,                           // Total levels in tree
    last_point: usize,                           // Index of last node in tree vector
    float_type: PhantomData<T>,                  // Specify what type of float the tree holds
}

/// Error types
#[derive(Debug, PartialEq)]
pub enum KdError {
    DimensionError,                              // Improper number of dimensions
    EmptyTree,                                   // No nodes in tree
    NodeMissing,                                 // Node doesn't exist
    BinaryHeapError,                             // Error associated with binary heap object
}

/// Node type used by tree to tell which direction to go in search
#[derive(Copy, Clone)]
enum NodeType {
    RootNode,                                    // First node in tree
    LeftChild,                                   // Node is left child
    RightChild,                                  // Node is right child
}

/// Return type that pairs point and distance to point
pub struct Closest<DataType, T> {
    pub point: DataType,                         // Closest point to query point
    pub distance: T,                             // Distance to closest point
}

/// Trait that must be satisfied for user defined point types (already defined for Vec types)
pub trait Point<T: Float> {
    /// Distance from one point to another
    fn distance(&self, other: &Self) -> Result<T, KdError>;
    /// Is point greater than other in current dimension
    fn greater(&self, other: &Self, cur_dimesnion: usize) -> bool;
    /// Create point that only contains value in current dimension
    fn split_plane(&self, cur_dimension: usize) -> Self;
    /// Dimensionality of point
    fn dimensions(&self) -> usize;
}

/// KdTree functions
impl<T: Float, DataType: Point<T> + Clone> KdTree<DataType, T> {
    /// Create a new tree with specified number of dimensions
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

    /// Create a new tree with specified number of dimensions and storage for specified capacity
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

    /// Add a point to the tree
    pub fn add_point(&mut self, query_point: DataType) -> Result<(), KdError> {
        // Verify point has proper number of dimensions
        if query_point.dimensions() != self.num_dimensions { return Err(KdError::DimensionError); }

        // Check if root node, if not go down to find proper place in tree
        let (parent_index, child_type) = if self.tree[1].is_none() {
            (0, NodeType::RootNode)
        } else {
            self.go_down(&query_point, 1)?
        };

        // Get level and split dimension of node
        let (current_dimension, current_level) = if self.last_point == 1 {
            (0, 0)
        } else {
            if let Some(node) = &mut self.tree[parent_index] {
                match child_type {
                    NodeType::LeftChild => { node.left_child = self.last_point; },
                    NodeType::RightChild => { node.right_child = self.last_point; },
                    NodeType::RootNode => { },
                }

                ((node.dimension + 1) % self.num_dimensions, node.level + 1)
            } else {
                return Err(KdError::DimensionError);
            }
        };

        // Update max levels
        self.max_levels = self.max_levels.max(current_level);

        // Resize vector if at capacity
        if self.last_point >= self.tree.len() - 1  {
            let capacity = self.last_point * 2;
            self.tree.reserve(capacity);
            self.tree.resize_with(capacity, Default::default);
        }

        // Add point
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

    /// Find absolute closest point to query point
    pub fn find_closest(&self, query_point: &DataType) -> Result<(DataType, T), KdError> {
        match self.find_n_closest(query_point, 1)?.pop() {
            Some(closest) => { Ok((closest.point, closest.distance)) },
            None => { Err(KdError::BinaryHeapError) },
        }
    }

    /// Find n closest points to query point
    pub fn find_n_closest(&self, query_point: &DataType, n: usize) -> Result<BinaryHeap<Closest<DataType, T>>, KdError> {
        // Create binary heap structure to store closest points
        let mut bh_closest = BinaryHeap::with_capacity(n);
        // Table to signify whether point has been searched or not
        let mut searched_table = vec![-1i64; self.max_levels + 1];
        // Go down to bin containing point
        let (mut index, mut child_type) = self.go_down(query_point, 1)?;

        // Go back up tree to see if there are any closer points
        while let Some(node) = &self.tree[index] {
            // If node has already been searched go up
            if searched_table[node.level] == index as i64 {
                child_type = node.child_type;
                index = node.parent;
                continue;
            }

            // Check node
            let distance = node.point.distance(query_point)?;
            if bh_closest.len() < n {                               // If binary heap isn't full add point
                bh_closest.push(Closest { point: index, distance: distance, });
            } else {                                                // Otherwise check that distance is less than that of the max point in heap
                if distance < self.get_max_min(&bh_closest)? {
                    bh_closest.pop();
                    bh_closest.push(Closest { point: index, distance: distance, });
                }
            }

            // Update table to avoid checking node again
            searched_table[node.level] = index as i64;

            // See if distance to split plane is less than min to see if other subtree needs to be
            // searched
            if node.point.split_plane(node.dimension).distance(&query_point.split_plane(node.dimension))? < self.get_max_min(&bh_closest)? {
                let sub_tree = match child_type {
                    NodeType::LeftChild => { node.right_child },
                    NodeType::RightChild => { node.left_child},
                    NodeType::RootNode => { 0 },
                };

                let go_down_result = self.go_down(query_point, sub_tree);
                if let Ok((cur_ind, cur_child)) = go_down_result { index = cur_ind; child_type = cur_child; }
            }
        }

        // Get actual points from indices to points in tree vec
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

    /// Brute force search for testing
    pub fn brute_force(&self, query_point: &DataType, n: usize) -> Result<BinaryHeap<Closest<DataType, T>>, KdError> {
        let mut bh_closest = BinaryHeap::with_capacity(n);
        for (cur_ind, node) in self.tree.iter().enumerate() {
            if let Some(cur_node) = node {
                let distance = cur_node.point.distance(query_point)?;
                if bh_closest.len() < n {
                    bh_closest.push(Closest { point: cur_ind, distance: distance, });
                } else {
                    if distance < self.get_max_min(&bh_closest)? {
                        bh_closest.pop();
                        bh_closest.push(Closest { point: cur_ind, distance: distance, });
                    }
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

    /// Search tree from root to leaf node
    fn go_down(&self, query_point: &DataType, root: usize) -> Result<(usize, NodeType), KdError> {
        // Verify sub tree is not empty
        if self.tree[root].is_none() {
            return Err(KdError::EmptyTree);
        }

        let mut current_index = root;               // Current index starting from root
        let mut index = current_index;              // Index to return
        let mut child_type = NodeType::RootNode;    // Type of node
        while let Some(node) = &self.tree[current_index] {
            index = current_index;
            
            // Go left if node point is greater than query in current dimension
            if node.point.greater(query_point, node.dimension) {
                current_index = node.left_child;
                child_type = NodeType::LeftChild;
            // Otherwise go right
            } else {
                current_index = node.right_child;
                child_type = NodeType::RightChild;
            }
        };

        Ok((index, child_type))
    }

    /// Get the maximum distance in binary heap of closest points
    fn get_max_min(&self, bh_closest: &BinaryHeap<Closest<usize, T>>) -> Result<T, KdError> {
        match bh_closest.peek() {
            Some(max) => { Ok(max.distance) },
            None => { Err(KdError::BinaryHeapError) },
        }
    }

    /// Getter for dimensions of tree
    pub fn get_num_dimensions(&self) -> usize { self.num_dimensions }
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
