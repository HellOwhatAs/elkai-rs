//! elkai-rs is a rust library for solving travelling salesman problems (TSP) based on elkai (LKH 3)   
//!
//!  ----
//!  
//!  * **based on [elkai](https://github.com/fikisipi/elkai) by fikisipi** **([LKH](http://akira.ruc.dk/~keld/research/LKH/) by Keld Helsgaun)**: with proven optimal solutions up to N=315 and more accurate results than [Google's OR tools](https://developers.google.com/optimization/routing/tsp)
//!  * **asymmetric and symmetric** [travelling salesman problems](https://en.wikipedia.org/wiki/Travelling_salesman_problem) support
//!  * **clean and simple API**: get results with one line calls
//!  
//!  ## Example usage
//!  
//!  ```rust
//!  use std::collections::HashMap;
//!  use elkai_rs::Coordinates2D;
//!  
//!  fn main() {
//!      let cities = Coordinates2D::new(HashMap::from_iter([
//!          ("city1", (0.0, 0.0)),
//!          ("city2", (0.0, 4.0)),
//!          ("city3", (5.0, 0.0))
//!      ]));
//!      println!("{:?}", cities.solve(10));
//!  }
//!  ```
//!  
//!  ```rust
//!  use elkai_rs::DistanceMatrix;
//!  
//!  fn main() {
//!      let cities = DistanceMatrix::new(vec![
//!          vec![0, 4, 0],
//!          vec![0, 0, 5],
//!          vec![0, 0, 0]
//!      ]);
//!      println!("{:?}", cities.solve(10));
//!  }
//!  ```
//!  
//!  ## License
//!  
//!  The LKH native code by Helsgaun is released for non-commercial use only. Therefore the same restriction applies to elkai-rs, which is explained in the `LICENSE` file. 
//!  
//!  ## How it works internally
//!  
//!  * We link the C api of elkai to Rust with [cc-rs](https://github.com/rust-lang/cc-rs).
//!  
//!  ⚠️ elkai-rs takes a **global mutex** (just like what elkai did) during the solving phase which means two threads cannot solve problems at the same time. If you want to run other workloads at the same time, you have to run another process.

#![allow(private_bounds)]
use libc::{c_uchar, size_t, c_int};
use std::{collections::HashMap, sync::Mutex};

extern "C" {
    fn _solve_problem(paramsStr: *const c_uchar, problemStr: *const c_uchar, toursize: *mut size_t, msg_buf: *mut u8) -> *mut c_int;
}

static ELKAI_MUTEX: Mutex<[u8; 1024]> = Mutex::new([0; 1024]);

fn elkai_solve_problem(param: &str, problem: &str) -> Vec<usize> {
    assert!(param.ends_with('\0') && problem.ends_with('\0'), "input string must end with '\\0'");

    let mut toursize: usize = 0;

    let mut msg_buf = ELKAI_MUTEX.lock().unwrap();
    
    let raw_pointer = unsafe {
        _solve_problem(
            param.as_ptr(),
            problem.as_ptr(),
            &mut toursize as *mut size_t,
            msg_buf.as_mut_ptr()
        )
    };
    if toursize == 0 {
        panic!("{}", String::from_iter(msg_buf.iter().map(|x| *x as char).take_while(|c| *c != '\0')))
    }
    let res = unsafe {
        &*std::ptr::slice_from_raw_parts(raw_pointer, toursize)
    }.iter().map(|e| (*e - 1) as usize).collect();
    
    drop(msg_buf);
    
    res
}

fn is_2d_matrix<T>(m: &Vec<Vec<T>>) -> bool {
    let dim = m.len();
    m.iter().map(|e| e.len()).all(|l| l == dim)
}

fn is_symmetric_matrix<T: PartialEq>(m: &Vec<Vec<T>>) -> bool {
    let n = m.len();
    for i in 0..n {
        for j in 0..n {
            if m[i][j] != m[j][i] {
                return false;
            }
        }
    }
    return true;
}

trait Num: num_traits::Num + std::fmt::Display {}
macro_rules! num_trait_impl { ($name:ident for $($t:ty)*) => ($(impl $name for $t {})*) }
num_trait_impl!(Num for usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 f32 f64);
impl<T: num_traits::Num + std::fmt::Display> Num for std::num::Wrapping<T>
where std::num::Wrapping<T>: num_traits::NumOps {}

/// A structure representing a matrix of float/int weights/distances.
/// ## Example usage
/// 
/// ```rust
/// use elkai_rs::DistanceMatrix;
/// 
/// fn main() {
///     let cities = DistanceMatrix::new(vec![
///         vec![0, 4, 0],
///         vec![0, 0, 5],
///         vec![0, 0, 0]
///     ]);
///     println!("{:?}", cities.solve(10));
/// }
/// ```
pub struct DistanceMatrix<T: Num> {
    distances: Vec<Vec<T>>
}

impl<T: Num> DistanceMatrix<T> {
    /// Creates the matrix structure representing a list of lists (2D matrix) of floats/ints.
    pub fn new(distances: Vec<Vec<T>>) -> Self {
        assert!(is_2d_matrix(&distances), "distances must be a 2D matrix");
        DistanceMatrix {
            distances
        }
    }

    /// Returns a list of indices that represent the TSP tour. You can adjust solver iterations with the runs parameter.
    pub fn solve(&self, runs: usize) -> Vec<usize> {
        assert!(runs >= 1, "runs must be a positive integer");
        let dimension = self.distances.len();
        assert!(dimension >= 3, "dimension must be at least 3");
        let param = format!("RUNS = {runs}\nPROBLEM_FILE = :stdin:\n\0");
        let problem_type = if is_symmetric_matrix(&self.distances) {"TSP"} else {"ATSP"};
        let mut problem = format!("TYPE : {problem_type}\nDIMENSION : {dimension}\nEDGE_WEIGHT_TYPE : EXPLICIT\nEDGE_WEIGHT_FORMAT : FULL_MATRIX\nEDGE_WEIGHT_SECTION\n");
        for row in &self.distances {
            problem.push_str(&row.iter().map(T::to_string).collect::<Vec<_>>().join(" "));
            problem.push('\n');
        }
        problem.push('\0');
        elkai_solve_problem(&param, &problem)
    }
}

/// A structure representing coordinates of type {'city name': (x, y), ...}
/// ## Example usage
///  
///  ```rust
///  use std::collections::HashMap;
///  use elkai_rs::Coordinates2D;
///  
///  fn main() {
///      let cities = Coordinates2D::new(HashMap::from_iter([
///          ("city1", (0.0, 0.0)),
///          ("city2", (0.0, 4.0)),
///          ("city3", (5.0, 0.0))
///      ]));
///      println!("{:?}", cities.solve(10));
///  }
///  ```
pub struct Coordinates2D<'a, T: Num> {
    coords: HashMap<&'a str, (T, T)>
}

impl<'a, T: Num> Coordinates2D<'a, T> {
    /// Creates the structure representing coordinates of type {'city name': (x, y), ...}
    pub fn new(coords: HashMap<&'a str, (T, T)>) -> Self {
        assert!(coords.len() >= 3, "there must be at least 3 cities");
        Coordinates2D { coords }
    }

    /// Returns a list of city names in the order of the TSP tour. You can adjust solver iterations with the runs parameter.
    pub fn solve(&self, runs: usize) -> Vec<&'a str> {
        assert!(runs >= 1, "runs must be a positive integer");
        let keys: Vec<&&str> = self.coords.keys().collect();
        
        let keys_to_numbers: HashMap<&&&str, usize> = HashMap::from_iter(keys.iter().enumerate()
            .map(|(i, k)| (k, i + 1)));
        let numbers_to_keys: HashMap<usize, &&&str> = HashMap::from_iter(keys.iter().enumerate());

        let dimension = keys.len();
        let param = format!("RUNS = {runs}\nPROBLEM_FILE = :stdin:\n\0");
        let mut problem = format!("TYPE : TSP\nDIMENSION : {dimension}\nEDGE_WEIGHT_TYPE : EUC_2D\nNODE_COORD_SECTION\n");
        for (key, num) in keys_to_numbers.iter() {
            let (x1, x2) = &self.coords[***key];
            problem.push_str(&format!("{num} {x1} {x2}\n"));
        }
        problem.push('\0');

        elkai_solve_problem(&param, &problem).into_iter().map(|num| {
            **numbers_to_keys[&num]
        }).collect()
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, io::Read};
    use crate::{elkai_solve_problem, Coordinates2D, DistanceMatrix};

    #[test]
    fn elkai_str() {
        let param = "RUNS = 10\nPROBLEM_FILE = :stdin:\n\0".to_string();
        let problem = "TYPE : ATSP\nDIMENSION : 3\nEDGE_WEIGHT_TYPE : EXPLICIT\nEDGE_WEIGHT_FORMAT : FULL_MATRIX\nEDGE_WEIGHT_SECTION\n0 4 0\n0 0 5\n0 0 0\n\0".to_string();
        println!("{:?} ", elkai_solve_problem(&param, &problem));
    }

    #[test]
    fn dis_mat() {
        let s = DistanceMatrix::new(vec![
            vec![0, 4, 0],
            vec![0, 0, 5],
            vec![0, 0, 0]
        ]);
        println!("{:?}", s.solve(10));
    }

    #[test]
    fn coordinates2d() {
        let s = Coordinates2D::new(HashMap::from_iter([
            ("city1", (0.0, 0.0)),
            ("city2", (0.0, 4.0)),
            ("city3", (5.0, 0.0))
        ]));
        println!("{:?}", s.solve(10));
    }

    fn coords_result(coords: &HashMap<&str, (f64, f64)>, solution: &Vec<&str>) -> f64 {
        fn dis(a: (f64, f64), b: (f64, f64)) -> f64 {
            ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
        }
        let mut res = (1..solution.len()).into_iter().map(|i| {
            dis(coords[solution[i-1]], coords[solution[i]])
        }).sum::<f64>();
        res += dis(coords[*solution.last().unwrap()], coords[*solution.first().unwrap()]);
        res
    }

    #[test]
    fn pr2392() {
        use text_io::scan;

        let mut s = String::new();
        std::fs::File::open("LKH-3.0.8/pr2392.tsp").unwrap().read_to_string(&mut s).unwrap();
        let start = s.find("NODE_COORD_SECTION").unwrap() + "NODE_COORD_SECTION".len();
        let end = s.rfind("EOF").unwrap();

        let (mut k, mut v) = (vec![], vec![]);
        for line in s[start..end].trim().lines() {
            let (idx, x, y): (usize, f64, f64);
            scan!(line.bytes() => "{} {} {}", idx, x, y);
            k.push(idx.to_string());
            v.push((x, y));
        }

        let coords: HashMap<&str, (f64, f64)> = HashMap::from_iter(k.iter().zip(v).map(|(k, v)| (k.as_str(), v)));
        let s = Coordinates2D::new(coords.clone());
        let solution = s.solve(10);
        println!("{:?}", solution);
        println!("{:?}", coords_result(&coords, &solution))
    }

    fn distances_result<T: std::iter::Sum + Copy + std::ops::AddAssign>(distances: &Vec<Vec<T>>, solution: &Vec<usize>) -> T {
        let mut res = (1..solution.len()).into_iter().map(|i| {
            distances[solution[i - 1]][solution[i]]
        }).sum::<T>();
        res += distances[*solution.last().unwrap()][*solution.first().unwrap()];
        res
    }

    #[test]
    fn whizzkids96() {
        let mut s = String::new();
        std::fs::File::open("LKH-3.0.8/whizzkids96.atsp").unwrap().read_to_string(&mut s).unwrap();
        let start = s.find("EDGE_WEIGHT_SECTION").unwrap() + "EDGE_WEIGHT_SECTION".len();
        let distances = s[start..].trim().lines().map(|line| line.split(' ').filter_map(|e| {
            let e = e.trim();
            match e.is_empty() {
                true => None,
                false => Some(e.parse::<usize>().unwrap()),
            }
        }).collect::<Vec<_>>()).collect::<Vec<_>>();
        let s = DistanceMatrix::new(distances.clone());
        let solution = s.solve(10);
        println!("{:?}", solution);
        println!("{:?}", distances_result(&distances, &solution));
    }
}