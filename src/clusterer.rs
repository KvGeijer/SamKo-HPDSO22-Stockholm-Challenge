use kodama::{Method, linkage};
use num_traits::float::Float;


pub fn cluster<T: Float>(mut matrix: Vec<T>, singles: usize, returns: usize) -> Vec<usize> {
    // - matrix: The vector representation of the upper triangular dissimilarity matrix, which
    // in our case can be interpreted as the adjacency matrix for a graph of flight travels.
    // - singles: The size of of one side of the original matrix (number of airports)
    // - returns: The number of indexes we return which are the topmost nodes in the dendrogram.
    // TODO: Can we make returns better? A generic value?

    let dend = linkage(&mut matrix, singles, Method::Average);

    dend.steps()
        .iter()
        .rev()
        .flat_map(|step| [step.cluster1, step.cluster2])
        .filter(|&cluster| cluster < singles)
        .take(returns)
        .collect()
}

#[test]
fn test_simple_clustering() {
    // Create a dissimilarity matrix where 0-1 close, 2-3 close, 4 far from everyone
    // Aproximately:
    //
    // ...1..........2.....
    // ..0.............3...
    // ....................
    // ....................
    // ....................
    // ....................
    // ....................
    // ...............4....
    //
    // [0.0, 1.0, 6.1, 6.5, 10.1]
    // [---, 0.0, 5.0, 6.2, 10,0]
    // [---, ---, 0.0, 1.5, 8.0 ]
    // [---, ---, ---, 0.0, 7.0 ]
    // [---, ---, ---, ---, 0.0 ]

    let mat: Vec<f32> = vec![1.0, 6.1, 6.5, 10.1, 5.0, 6.2, 10.0, 1.5, 8.0, 7.0];

    let all_sorted_singles = cluster(mat, 5, 5);
    assert_eq!(all_sorted_singles, vec![4, 2, 3, 0, 1]);

}