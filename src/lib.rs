use libc;

extern "C" {
    fn _solve_problem(paramsStr: *const libc::c_uchar, problemStr: *const libc::c_uchar, toursize: *mut libc::size_t) -> *mut libc::c_int;
}

pub fn elkai_solve_problem(param: &str, problem: &str) -> Vec<i32> {

    assert!(param.ends_with('\0') && problem.ends_with('\0'), "input string must end with '\\0'");

    let mut toursize: usize = 0;
    let raw_pointer = unsafe {
        _solve_problem(
            param.as_ptr(),
            problem.as_ptr(),
            &mut toursize as *mut libc::size_t
        )
    };
    let res = unsafe {
        &*std::ptr::slice_from_raw_parts(raw_pointer, toursize)
    }.to_vec();

    res
}

#[test]
fn test() {
    for _ in 0..10000 {
        let param = "RUNS = 10\nPROBLEM_FILE = :stdin:\n\0".to_string();
        let problem = "TYPE : ATSP\nDIMENSION : 3\nEDGE_WEIGHT_TYPE : EXPLICIT\nEDGE_WEIGHT_FORMAT : FULL_MATRIX\nEDGE_WEIGHT_SECTION\n0 4 0\n0 0 5\n0 0 0\n\0".to_string();
        print!("{:?} ", elkai_solve_problem(&param, &problem));
    }
}