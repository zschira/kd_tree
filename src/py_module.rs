extern crate pyo3;
use crate::kd_tree::{KdTree, KdError};

use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use pyo3::{PyResult, exceptions, Python};
use ndarray::{Array1, Axis};

impl From<KdError> for PyErr {
    fn from(err: KdError) -> PyErr {
        PyErr::new::<exceptions::TypeError, _>(err.to_string())
    }
}

#[pyclass]
pub struct PyTree {
    tree: KdTree<Array1<f64>, f64>,
}

#[pymethods]
impl PyTree {
    #[new]
    fn new(dimensions: usize, num_nodes: usize) -> Self {
        PyTree {
            tree: KdTree::with_capacity(dimensions, num_nodes),
        }
    }

    #[new]
    fn create_tree(points: &PyArray2<f64>) -> PyResult<Self> {
        let shape = points.shape();
        let mut tree = PyTree { tree: KdTree::with_capacity(shape[1], shape[0]) };
        match tree.add_points(points) {
            Ok(()) => { Ok(tree) },
            Err(e) => { Err(e) },
        }
    }

    fn add_point(&mut self, point: &PyArray1<f64>) -> PyResult<()> {
        match self.tree.add_point(point.to_owned_array()) {
            Ok(()) => { Ok(()) },
            Err(e) => { Err(PyErr::from(e)) },
        }
    }

    fn add_points(&mut self, points: &PyArray2<f64>) -> PyResult<()> {
        for point in points.to_owned_array().axis_iter(Axis(0)) {
            self.tree.add_point(point.to_owned())?;
        }

        Ok(())
    }

    fn find_closest(&self, query_point: &PyArray1<f64>) -> PyResult<(Py<PyArray1<f64>>, f64)> {
        match self.tree.find_closest(&query_point.as_array().to_owned()) {
            Ok((point, distance)) => {
                let gil = Python::acquire_gil();
                Ok((PyArray1::from_owned_array(gil.python(), point).to_owned(), distance)) 
            },
            Err(e) => { Err(PyErr::from(e)) },
        }
    }
}

#[pymodule]
fn kd_tree(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTree>()
}
