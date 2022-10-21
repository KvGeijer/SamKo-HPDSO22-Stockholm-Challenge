use kodama::{Method, linkage};
use num_traits::float::Float;


pub fn cluster<T: Float>(mut matrix: Vec<T>, singles: usize, returns: usize) -> Vec<usize> {
    // matrix is the vector representation of the upper triangular dissimilarity matrix, which
    // in our case can be interpreted as the adjacency matrix for a graph of flight travels.
    // singles is the size of of one side of the original matrix (number of airports)
    // returns is the number of indexes we return which are the highest nodes in the dendogram.
    // It can actually return a vector of length (returns + 1) if last step is a merge of
    // singles.
    // The returned vector is sorted with the highest element first.
    // TODO: Can we make returns better? A generic value?

    let dend = linkage(&mut matrix, singles, Method::Average);

    let mut highest_singles = vec![];

    for step in dend.steps() {
        for cluster in [step.cluster1, step.cluster2] {
            if cluster < singles {
                highest_singles.push(cluster);
            }
        }

        if highest_singles.len() >= returns {
            return highest_singles;
        }
    }

    panic!("Not enough singles found in dendogram!");
}
