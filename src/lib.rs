use libc;
use std::sync::Mutex;

extern "C" {
    fn _solve_problem(paramsStr: *const libc::c_uchar, problemStr: *const libc::c_uchar, toursize: *mut libc::size_t) -> *mut libc::c_int;
}

static ELKAI_MUTEX: Mutex<()> = Mutex::new(());

pub fn elkai_solve_problem(param: &str, problem: &str) -> Vec<usize> {
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

pub trait TspSolver {
    fn solve(&self, runs: usize) -> Vec<usize>;
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
}

impl<T: PartialEq + std::fmt::Display> TspSolver for DistanceMatrix<T> {
    fn solve(&self, runs: usize) -> Vec<usize> {
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

#[cfg(test)]
mod test {
    use crate::{elkai_solve_problem, DistanceMatrix, TspSolver};

    #[test]
    fn elkai_str() {
        let param = "RUNS = 10\nPROBLEM_FILE = :stdin:\n\0".to_string();
        let problem = "TYPE : ATSP\nDIMENSION : 3\nEDGE_WEIGHT_TYPE : EXPLICIT\nEDGE_WEIGHT_FORMAT : FULL_MATRIX\nEDGE_WEIGHT_SECTION\n0 4 0\n0 0 5\n0 0 0\n\0".to_string();
        print!("{:?} ", elkai_solve_problem(&param, &problem));
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
}