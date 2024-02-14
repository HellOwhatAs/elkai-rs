#include "math.h"
#include "gb_string.h"
#include "setjmp.h"

// // These are implemented in the LKH-3.0.8/SRC directory.

extern void ElkaiSolveProblem(gbString params, gbString problem, int *tourSize, int **tourPtr);


// Our copy of LKH is highly modified and does not correspond to the upstream. In the future,
// we should ship the original LKH folder and then apply a patch *at build time*.

int* _solve_problem(const char *paramsStr, const char *problemStr, size_t* toursize) {
    gbString params = gb_make_string(paramsStr);
    gbString problem = gb_make_string(problemStr);
    
    int tourSize = 0;
    int *tourPtr;
    ElkaiSolveProblem(params, problem, &tourSize, &tourPtr);

    gb_free_string(params);
    gb_free_string(problem);

    *toursize = tourSize;
    return tourPtr;
}