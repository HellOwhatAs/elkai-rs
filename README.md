<p align="center">
 <img src="https://github.com/HellOwhatAs/elkai/assets/88815487/45999663-6687-40b9-af56-5108e6b61053" alt="" />
</p>
<p align="center">
<em>elkai-rs - a Rust library for solving TSP problems</em>
</p>
<p align="center">
<a href="https://crates.io/crates/elkai-rs"><img src="https://img.shields.io/crates/v/elkai-rs" alt="crates.io"></a>
<a href="https://docs.rs/elkai-rs/"><img src="https://docs.rs/elkai-rs/badge.svg" alt="docs"></a>
<a href="https://github.com/HellOwhatAs/elkai-rs/"><img src="https://img.shields.io/github/repo-size/HellOwhatAs/elkai-rs" alt="repo"></a>
</p>

----

* **based on [elkai](https://github.com/fikisipi/elkai) by fikisipi** **([LKH](http://akira.ruc.dk/~keld/research/LKH/) by Keld Helsgaun)**: with proven optimal solutions up to N=315 and more accurate results than [Google's OR tools](https://developers.google.com/optimization/routing/tsp)
* **asymmetric and symmetric** [travelling salesman problems](https://en.wikipedia.org/wiki/Travelling_salesman_problem) support
* **clean and simple API**: get results with one line calls

## Installation

```toml
[dependencies]
elkai-rs = "0.1.7"
```

## Example usage

```rust
use std::collections::HashMap;
use elkai_rs::Coordinates2D;

fn main() {
    let cities = Coordinates2D::new(HashMap::from_iter([
        ("city1", (0.0, 0.0)),
        ("city2", (0.0, 4.0)),
        ("city3", (5.0, 0.0))
    ]));
    println!("{:?}", cities.solve(10));
}
```

```rust
use elkai_rs::DistanceMatrix;

fn main() {
    let cities = DistanceMatrix::new(vec![
        vec![0, 4, 0],
        vec![0, 0, 5],
        vec![0, 0, 0]
    ]);
    println!("{:?}", cities.solve(10));
}
```

## License

The LKH native code by Helsgaun is released for non-commercial use only. Therefore the same restriction applies to elkai-rs, which is explained in the `LICENSE` file. 

## How it works internally

* We link the C api of elkai to Rust with [cc-rs](https://github.com/rust-lang/cc-rs).

⚠️ elkai-rs takes a **global mutex** (just like what elkai did) during the solving phase which means two threads cannot solve problems at the same time. If you want to run other workloads at the same time, you have to run another process.