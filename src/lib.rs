use libc;
use std::{collections::HashMap, sync::Mutex};

extern "C" {
    fn _solve_problem(paramsStr: *const libc::c_uchar, problemStr: *const libc::c_uchar, toursize: *mut libc::size_t) -> *mut libc::c_int;
}

static ELKAI_MUTEX: Mutex<()> = Mutex::new(());

fn elkai_solve_problem(param: &str, problem: &str) -> Vec<usize> {
    assert!(param.ends_with('\0') && problem.ends_with('\0'), "input string must end with '\\0'");

    let mut toursize: usize = 0;

    let lock = ELKAI_MUTEX.lock().unwrap();
    
    let raw_pointer = unsafe {
        _solve_problem(
            param.as_ptr(),
            problem.as_ptr(),
            &mut toursize as *mut libc::size_t
        )
    };
    let res = unsafe {
        &*std::ptr::slice_from_raw_parts(raw_pointer, toursize)
    }.iter().map(|e| (*e - 1) as usize).collect();
    
    drop(lock);
    
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

pub struct DistanceMatrix<T: PartialEq + std::fmt::Display> {
    distances: Vec<Vec<T>>
}

impl<T: PartialEq + std::fmt::Display> DistanceMatrix<T> {
    pub fn new(distances: Vec<Vec<T>>) -> Self {
        assert!(is_2d_matrix(&distances), "distances must be a 2D matrix");
        DistanceMatrix {
            distances
        }
    }

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

pub struct Coordinates2D<'a, T: std::fmt::Display> {
    coords: HashMap<&'a str, (T, T)>
}

impl<'a, T: std::fmt::Display> Coordinates2D<'a, T> {
    pub fn new(coords: HashMap<&'a str, (T, T)>) -> Self {
        assert!(coords.len() >= 3, "there must be at least 3 cities");
        Coordinates2D { coords }
    }

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
            ("city1", (0, 0)),
            ("city2", (0, 4)),
            ("city3", (5, 0))
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