//! Guest zkVM: ejecuta el MISMO ranking que el servidor (crate `feed-core`)
//! y compromete en el journal público el algoritmo usado y los hashes de
//! entrada/salida. La prueba STARK resultante certifica esta ejecución.

use feed_core::FeedInput;
use risc0_zkvm::guest::env;

fn main() {
    let input: FeedInput = env::read();
    let feed_ids = feed_core::rank(&input);
    let journal = feed_core::make_journal(&input, &feed_ids);
    env::commit(&journal);
}
