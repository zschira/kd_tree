enum NodeType {
    RootNode,
    LeftChild,
    RightChild,
}

pub trait Point {
    fn distance(&self, other: &Self) -> Result<f64, KdError>;
    fn greater(&self, other: &Self, cur_dimesnion: usize) -> bool;
    fn split_plane(&self, cur_dimension: usize) -> Self;
}

struct Node<DataType> {
    left_child: Option<Box<Node<DataType>>>,
    right_child: Option<Box<Node<DataType>>>,
    point: DataType,
    node_type: NodeType,
    dimension: usize,
    num_dimensions: usize,
}

#[derive(Debug, PartialEq)]
pub enum KdError {
    DimensionError,
    EmptyTree,
}

pub struct KdTree<DataType> {
    tree: Option<Box<Node<DataType>>>,
    num_dimensions: usize,
}

impl<DataType: Point + Clone> KdTree<DataType> {
    pub fn new(dimensions: usize) -> Self {
        KdTree {
            tree: None,
            num_dimensions: dimensions,
        }
    }

    pub fn add_point(&mut self, new_point: DataType) {
        match &mut self.tree {
            Some(root) => {
                root.add_point(new_point);
            },
            None => {
                self.tree = Some(Box::new(Node {
                    left_child: None,
                    right_child: None,
                    point: new_point,
                    node_type: NodeType::RootNode,
                    dimension: 0,
                    num_dimensions: self.num_dimensions,
                }));
            },
        }
    }

    pub fn find_closest(&self, query_point: DataType) -> Result<(DataType, f64), KdError> {
        if let Some(root) = &self.tree {
            root.find_closest(query_point) 
        } else {
            Err(KdError::EmptyTree)
        }
    }
}

impl<DataType: Point + Clone> Node<DataType> {
    pub fn add_point(&mut self, new_point: DataType) {
            // Handle normal node
            let cur_dimension = self.dimension;
            let (child, child_type) = if self.point.greater(&new_point, cur_dimension) {
                (&mut self.left_child, NodeType::LeftChild)
            } else {
                (&mut self.right_child, NodeType::RightChild)
            };

            if let Some(node) = child {
                node.add_point(new_point);
            } else {
                *child = Some(Box::new(Node {
                    left_child: None,
                    right_child: None,
                    point: new_point,
                    node_type: child_type,
                    dimension: (self.dimension + 1) % self.num_dimensions,
                    num_dimensions: self.num_dimensions,
                }));
            }
    }

    fn find_closest(&self, query_point: DataType) -> Result<(DataType, f64), KdError> {
        self.go_down(&query_point)
    }

    fn go_down(&self, query_point: &DataType) -> Result<(DataType, f64), KdError> {
        // Check which direction to go to
        let (child, other_child) = if self.point.greater(&query_point, self.dimension) {
            (&self.left_child, &self.right_child)
        } else {
            (&self.right_child, &self.left_child)
        };

        if let Some(node) = child {
            // Go down to leaf node if it exists
            let (point, min_distance) = node.go_down(query_point)?;

            // Check other side of the tree if distance to split plane is less than min
            // distance
            if self.point.split_plane(self.dimension).distance(query_point)? < min_distance {
                if let Some(other_node) = other_child {
                    let (mut other_point, mut other_distance) = other_node.go_down(query_point)?;

                    // Check current node
                    let cur_distance = self.point.distance(&query_point)?;
                    if cur_distance < other_distance {
                        other_distance = cur_distance;
                        other_point = self.point.clone();
                    }

                    // Return minimum found
                    if other_distance < min_distance {
                        Ok((other_point, other_distance))
                    } else {
                        Ok((point, min_distance))
                    }
                } else {
                    let other_distance = self.point.distance(&query_point)?;
                    if other_distance < min_distance {
                        Ok((self.point.clone(), self.point.distance(&query_point)?))
                    } else {
                        Ok((point, min_distance))
                    }
                }
             } else {
                 Ok((point, min_distance))
             }

        } else {
            // Check the distance to the current point
            Ok((self.point.clone(), self.point.distance(&query_point)?))
        }
    }
}

impl std::error::Error for KdError {
    fn description(&self) -> &str {
        match *self {
            KdError::DimensionError => "dimension error",
            KdError::EmptyTree => "no nodes in tree",
        }
    }
}

impl std::fmt::Display for KdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "KdTree error: {}", self.description())
    }
}

